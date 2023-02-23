use std::ops::RangeInclusive;
use std::rc::Rc;

use aws_sdk_s3::model::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::output::{
    CompleteMultipartUploadOutput,
    CreateMultipartUploadOutput,
    GetObjectOutput,
    PutObjectOutput,
    UploadPartCopyOutput,
    UploadPartOutput,
};
use aws_sdk_s3::{Client, Credentials, Region};
use tokio::runtime::Runtime;
use uuid::Uuid;

use super::connector::build_s3_client;
use super::error::S3Error;
use super::helpers::{defuse, execute_by_step, to_completed_part};
use super::models::{CreateNewEmptyResp, WriteResp};

// S3 multipart upload limits
const MINIMUM_PART_SIZE: usize = 5 * 1024 * 1024;
const MAXIMUM_PART_SIZE: usize = 5 * 1024 * 1024 * 1024;

#[cfg_attr(test, mockall::automock)]
pub trait S3Client {
    fn read_object(
        &self,
        object_name: &str,
        bucket_name: &str,
        range: Option<RangeInclusive<usize>>,
        e_tag: Option<String>,
    ) -> Result<Vec<u8>, S3Error>;
    fn write_buffer(
        &self,
        object_name: &str,
        bucket_name: &str,
        position: usize,
        buf: &[u8],
        file_size: usize,
        e_tag: Option<String>,
    ) -> Result<WriteResp, S3Error>;
    fn remove_object(&self, object_name: &str, bucket_name: &str) -> Result<(), S3Error>;
    fn save_object(
        &self,
        object_name: &str,
        bucket_name: &str,
        buf: Vec<u8>,
    ) -> Result<(), S3Error>;
    fn create_new_empty(&self, bucket_name: &str) -> Result<CreateNewEmptyResp, S3Error>;
}

pub struct WildlandS3Client {
    rt: Rc<Runtime>,
    client: Client,
}

impl WildlandS3Client {
    pub fn new(
        rt: Rc<Runtime>,
        credentials: Credentials,
        region: Region,
        endpoint_url: Option<String>,
    ) -> Self {
        WildlandS3Client {
            rt,
            client: build_s3_client(credentials, region, endpoint_url),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn multipart_copy(
        &self,
        begin: usize,
        end: usize,
        bucket_name: &str,
        copy_source: &str,
        e_tag: &Option<String>,
        new_object_name: &str,
        upload_id: &Option<String>,
        part_number: &mut i32,
    ) -> Result<Vec<CompletedPart>, S3Error> {
        let mut completed_parts = vec![];
        execute_by_step(begin, end, MAXIMUM_PART_SIZE, |begin, end| {
            let UploadPartCopyOutput {
                copy_part_result, ..
            } = self.rt.block_on(async {
                self.client
                    .upload_part_copy()
                    .bucket(bucket_name)
                    .copy_source(copy_source)
                    .set_copy_source_if_match(e_tag.clone())
                    .copy_source_range(format!("bytes={begin}-{}", end - 1))
                    .key(new_object_name)
                    .part_number(*part_number)
                    .set_upload_id(upload_id.clone())
                    .send()
                    .await
            })?;

            completed_parts.push(to_completed_part(copy_part_result, *part_number));
            *part_number += 1;
            Ok(())
        })?;
        Ok(completed_parts)
    }
}

impl S3Client for WildlandS3Client {
    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn read_object(
        &self,
        object_name: &str,
        bucket_name: &str,
        range: Option<RangeInclusive<usize>>,
        e_tag: Option<String>,
    ) -> Result<Vec<u8>, S3Error> {
        let range_bytes = range.map(|range| format!("bytes={}-{}", range.start(), range.end()));

        let GetObjectOutput { body, .. } = self.rt.block_on(async {
            self.client
                .get_object()
                .bucket(bucket_name)
                .set_range(range_bytes)
                .key(object_name)
                .set_if_match(e_tag)
                .send()
                .await
        })?;

        self.rt
            .block_on(async { body.collect().await })
            .map(|v| v.to_vec())
            .map_err(|e| S3Error::Generic(e.into()))
    }

    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn write_buffer(
        &self,
        object_name: &str,
        bucket_name: &str,
        mut position: usize,
        buf: &[u8],
        file_size: usize,
        e_tag: Option<String>,
    ) -> Result<WriteResp, S3Error> {
        let new_object_name = Uuid::new_v4().to_string();

        let CreateMultipartUploadOutput { upload_id, .. } = self.rt.block_on(async {
            self.client
                .create_multipart_upload()
                .bucket(bucket_name)
                .key(&new_object_name)
                .send()
                .await
        })?;

        let aboart_upload_on_exit = scopeguard::guard((), |_| {
            tracing::error!("Failed to perform multipart upload. Aborting.");
            if self
                .rt
                .block_on(async {
                    self.client
                        .abort_multipart_upload()
                        .bucket(bucket_name)
                        .key(&new_object_name)
                        .set_upload_id(upload_id.clone())
                        .send()
                        .await
                })
                .is_err()
            {
                tracing::error!("Failed to abort multipart upload");
            }
        });

        let mut buffer = vec![];
        let mut part_number = 1;
        let mut completed_parts = vec![];
        let copy_source = format!("{bucket_name}/{object_name}");

        // Copy range [0, position) from the original file
        match position {
            // Nothing to copy
            0 => (),
            // Not enough bytes to use multipart copy. We need to copy it manually
            p if p < MINIMUM_PART_SIZE => {
                buffer = self.read_object(
                    object_name.as_ref(),
                    bucket_name,
                    Some(RangeInclusive::new(0, position - 1)),
                    e_tag.clone(),
                )?;
                position = 0;
            }
            _ => {
                let parts = self.multipart_copy(
                    0,
                    position,
                    bucket_name,
                    &copy_source,
                    &e_tag,
                    &new_object_name,
                    &upload_id,
                    &mut part_number,
                )?;
                completed_parts.extend(parts);
            }
        }

        buffer.extend_from_slice(buf);
        let mut position_after_write = position + buffer.len();

        // We might not have enough bytes to perform multipart upload so wee need to take some from the original file.
        // Buffer size may not reach MINIMUM_PART_SIZE after that "if" statement but it not a problem.
        // It means that we are uploading the last part of a file and MINIMUM_PART_SIZE restriction doesn't need to be fulfilled.
        if buffer.len() < MINIMUM_PART_SIZE && position_after_write < file_size {
            let remaining_bytes = self.read_object(
                object_name.as_ref(),
                bucket_name,
                Some(RangeInclusive::new(
                    position + buffer.len(),
                    MINIMUM_PART_SIZE - buffer.len() - 1,
                )),
                e_tag.clone(),
            )?;
            buffer.extend(remaining_bytes);
        }

        buffer.chunks(MINIMUM_PART_SIZE).try_for_each(|chunk| {
            let UploadPartOutput {
                e_tag,
                checksum_crc32,
                checksum_crc32_c,
                checksum_sha1,
                checksum_sha256,
                ..
            } = self.rt.block_on(async {
                self.client
                    .upload_part()
                    .body(chunk.to_vec().into())
                    .bucket(bucket_name)
                    .key(&new_object_name)
                    .part_number(part_number)
                    .set_upload_id(upload_id.clone())
                    .send()
                    .await
            })?;

            completed_parts.push(
                CompletedPart::builder()
                    .set_e_tag(e_tag)
                    .set_checksum_crc32(checksum_crc32)
                    .set_checksum_crc32_c(checksum_crc32_c)
                    .set_checksum_sha1(checksum_sha1)
                    .set_checksum_sha256(checksum_sha256)
                    .part_number(part_number)
                    .build(),
            );
            part_number += 1;
            Ok::<_, S3Error>(())
        })?;

        position_after_write = position + buffer.len();
        if position_after_write < file_size {
            let parts = self.multipart_copy(
                position_after_write,
                file_size,
                bucket_name,
                &copy_source,
                &e_tag,
                &new_object_name,
                &upload_id,
                &mut part_number,
            )?;
            completed_parts.extend(parts);
        }

        let CompleteMultipartUploadOutput { e_tag, .. } = self.rt.block_on(async {
            self.client
                .complete_multipart_upload()
                .bucket(bucket_name)
                .key(&new_object_name)
                .multipart_upload(
                    CompletedMultipartUpload::builder()
                        .set_parts(Some(completed_parts))
                        .build(),
                )
                .set_upload_id(upload_id.clone())
                .send()
                .await
        })?;

        defuse(aboart_upload_on_exit);

        Ok(WriteResp {
            bytes_count: buf.len(),
            new_object_name,
            new_e_tag: e_tag.unwrap(),
        })
    }

    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn remove_object(&self, object_name: &str, bucket_name: &str) -> Result<(), S3Error> {
        self.rt.block_on(async {
            self.client
                .delete_object()
                .bucket(bucket_name)
                .key(object_name)
                .send()
                .await
        })?;
        Ok(())
    }

    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn save_object(
        &self,
        object_name: &str,
        bucket_name: &str,
        buf: Vec<u8>,
    ) -> Result<(), S3Error> {
        self.rt.block_on(async {
            self.client
                .put_object()
                .bucket(bucket_name)
                .key(object_name)
                .body(buf.into())
                .send()
                .await
        })?;
        Ok(())
    }

    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn create_new_empty(&self, bucket_name: &str) -> Result<CreateNewEmptyResp, S3Error> {
        let object_name = Uuid::new_v4().to_string();

        let PutObjectOutput { e_tag, .. } = self.rt.block_on(async {
            self.client
                .put_object()
                .bucket(bucket_name)
                .key(&object_name)
                .body(Vec::new().into())
                .send()
                .await
        })?;

        Ok(CreateNewEmptyResp {
            object_name,
            e_tag: e_tag.unwrap(),
        })
    }
}
