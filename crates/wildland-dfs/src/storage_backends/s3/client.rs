use std::path::{Path, PathBuf};
use std::rc::Rc;

use aws_sdk_s3::model::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::output::{
    CompleteMultipartUploadOutput,
    CreateMultipartUploadOutput,
    GetObjectOutput,
    HeadObjectOutput,
    UploadPartCopyOutput,
    UploadPartOutput,
};
use aws_sdk_s3::{Client, Credentials, Region};
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;
use wildland_corex::dfs::interface::{NodeType, Stat, UnixTimestamp};

use super::connector::build_s3_client;
use super::error::S3Error;
use super::helpers::{
    add_trailling_slash,
    execute_by_step,
    remove_trailling_slash,
    to_completed_part,
};
use super::models::{ObjectAttributes, WriteResp};

const NODE_TYPE_META_KEY: &str = "nodeType";

// S3 multipart upload limits
const MINIMUM_PART_SIZE: usize = 5 * 1024 * 1024;
const MAXIMUM_PART_SIZE: usize = 5 * 1024 * 1024 * 1024;

#[cfg_attr(test, mockall::automock)]
pub trait S3Client {
    fn list_files(&self, path: &Path, bucket_name: &str) -> Result<Vec<PathBuf>, S3Error>;
    fn get_object_attributes(
        &self,
        path: &Path,
        bucket_name: &str,
    ) -> Result<ObjectAttributes, S3Error>;
    fn read_object(
        &self,
        path: &Path,
        bucket_name: &str,
        position: usize,
        number_of_bytes: usize,
        etag: Option<String>,
    ) -> Result<Vec<u8>, S3Error>;
    fn write_buffer(
        &self,
        path: &Path,
        bucket_name: &str,
        position: usize,
        buf: &[u8],
        file_size: usize,
        etag: Option<String>,
    ) -> Result<WriteResp, S3Error>;
    fn create_dir(&self, path: &Path, bucket_name: &str) -> Result<(), S3Error>;
    fn remove_object(&self, path: &Path, bucket_name: &str) -> Result<(), S3Error>;
}

pub struct WildlandS3Client {
    rt: Rc<Runtime>,
    client: Client,
}

impl WildlandS3Client {
    pub fn new(rt: Rc<Runtime>, credentials: Credentials, region: Region) -> Self {
        WildlandS3Client {
            rt,
            client: build_s3_client(credentials, region),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn multipart_copy(
        &self,
        begin: usize,
        end: usize,
        bucket_name: &str,
        copy_source: &str,
        etag: &Option<String>,
        key: &Option<String>,
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
                    .set_copy_source_if_match(etag.clone())
                    .copy_source_range(format!("bytes={begin}-{}", end - 1))
                    .set_key(key.clone())
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
    fn list_files(&self, path: &Path, bucket_name: &str) -> Result<Vec<PathBuf>, S3Error> {
        let object_key = add_trailling_slash(path);

        let result = self.rt.block_on(async {
            self.client
                .list_objects_v2()
                .bucket(bucket_name)
                .prefix(object_key.clone())
                .into_paginator()
                .send()
                .collect::<Result<Vec<_>, _>>()
                .await
        })?;

        Ok(result
            .into_iter()
            .filter_map(|item| item.contents)
            .flatten()
            .filter_map(|object| object.key)
            .filter(|key| key != &object_key)
            .filter(|key| {
                key.strip_prefix(&object_key).map(|tail| tail.contains('/')) == Some(false)
            })
            .map(Into::into)
            .collect())
    }

    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn get_object_attributes(
        &self,
        path: &Path,
        bucket_name: &str,
    ) -> Result<ObjectAttributes, S3Error> {
        let path = remove_trailling_slash(path);

        let HeadObjectOutput {
            last_modified,
            content_length,
            e_tag,
            metadata,
            ..
        } = self.rt.block_on(async {
            self.client
                .head_object()
                .bucket(bucket_name)
                .key(path)
                .send()
                .await
        })?;

        Ok(ObjectAttributes {
            stat: Stat {
                node_type: metadata
                    .as_ref()
                    .and_then(|metadata| metadata.get(NODE_TYPE_META_KEY))
                    .map(|node_type| match node_type.as_str() {
                        "File" => NodeType::File,
                        "Dir" => NodeType::Dir,
                        "Symlink" => NodeType::Symlink,
                        "Other" => NodeType::Other,
                        _ => NodeType::Other,
                    })
                    .unwrap_or(NodeType::Other),
                size: content_length as _,
                access_time: None,
                modification_time: last_modified.map(|time| UnixTimestamp {
                    sec: time.secs() as _,
                    nano_sec: time.subsec_nanos(),
                }),
                change_time: None,
            },
            etag: e_tag,
        })
    }

    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn read_object(
        &self,
        path: &Path,
        bucket_name: &str,
        position: usize,
        number_of_bytes: usize,
        etag: Option<String>,
    ) -> Result<Vec<u8>, S3Error> {
        let path = remove_trailling_slash(path);

        let GetObjectOutput { body, .. } = self.rt.block_on(async {
            self.client
                .get_object()
                .bucket(bucket_name)
                .range(format!(
                    "bytes={begin}-{end}",
                    begin = position,
                    end = position + number_of_bytes - 1
                ))
                .key(path)
                .set_if_match(etag)
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
        path: &Path,
        bucket_name: &str,
        mut position: usize,
        buf: &[u8],
        file_size: usize,
        etag: Option<String>,
    ) -> Result<WriteResp, S3Error> {
        let path = remove_trailling_slash(path);

        let CreateMultipartUploadOutput { upload_id, key, .. } = self.rt.block_on(async {
            self.client
                .create_multipart_upload()
                .bucket(bucket_name)
                .key(path.clone())
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
                        .set_key(key.clone())
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
        let copy_source = format!("{bucket_name}/{path}");

        // Copy range [0, position) from the original file
        match position {
            // Nothing to copy
            0 => (),
            // Not enough bytes to use multipart copy. We need to copy it manually
            p if p < MINIMUM_PART_SIZE => {
                buffer = self.read_object(path.as_ref(), bucket_name, 0, position, etag.clone())?;
                position = 0;
            }
            _ => {
                let parts = self.multipart_copy(
                    0,
                    position,
                    bucket_name,
                    &copy_source,
                    &etag,
                    &key,
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
        // It means that we are uploading the last part of a file and MINIMUM_PART_SIZE restriction doesn't need to be fullfiled.
        if buffer.len() < MINIMUM_PART_SIZE && position_after_write < file_size {
            let remaining_bytes = self.read_object(
                path.as_ref(),
                bucket_name,
                position + buffer.len(),
                MINIMUM_PART_SIZE - buffer.len(),
                etag.clone(),
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
                    .set_key(key.clone())
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
                &etag,
                &key,
                &upload_id,
                &mut part_number,
            )?;
            completed_parts.extend(parts);
        }

        let CompleteMultipartUploadOutput { e_tag, .. } = self.rt.block_on(async {
            self.client
                .complete_multipart_upload()
                .bucket(bucket_name)
                .set_key(key.clone())
                .multipart_upload(
                    CompletedMultipartUpload::builder()
                        .set_parts(Some(completed_parts))
                        .build(),
                )
                .set_upload_id(upload_id.clone())
                .send()
                .await
        })?;

        scopeguard::ScopeGuard::into_inner(aboart_upload_on_exit); // defuse

        Ok(WriteResp {
            bytes_count: buf.len(),
            etag: e_tag,
        })
    }

    fn create_dir(&self, path: &Path, bucket_name: &str) -> Result<(), S3Error> {
        let path = remove_trailling_slash(path);

        self.rt.block_on(async {
            self.client
                .put_object()
                .bucket(bucket_name)
                .body(Vec::new().into())
                .key(path)
                .metadata(NODE_TYPE_META_KEY, "Dir")
                .send()
                .await
        })?;
        Ok(())
    }

    fn remove_object(&self, path: &Path, bucket_name: &str) -> Result<(), S3Error> {
        let path = remove_trailling_slash(path);

        self.rt.block_on(async {
            self.client
                .delete_object()
                .bucket(bucket_name)
                .key(path)
                .send()
                .await
        })?;
        Ok(())
    }
}
