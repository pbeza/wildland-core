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
use super::helpers::{execute_by_step, to_completed_part};
use super::models::{ObjectAttributes, WriteResp};

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
}

impl S3Client for WildlandS3Client {
    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn list_files(&self, path: &Path, bucket_name: &str) -> Result<Vec<PathBuf>, S3Error> {
        let path = path.to_string_lossy();

        let result = self.rt.block_on(async {
            self.client
                .list_objects_v2()
                .bucket(bucket_name)
                .prefix(path)
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
            .map(Into::into)
            .collect())
    }

    #[tracing::instrument(err(Debug), level = "debug", skip_all)]
    fn get_object_attributes(
        &self,
        path: &Path,
        bucket_name: &str,
    ) -> Result<ObjectAttributes, S3Error> {
        let HeadObjectOutput {
            last_modified,
            content_length,
            e_tag,
            ..
        } = self.rt.block_on(async {
            self.client
                .head_object()
                .bucket(bucket_name)
                .key(path.to_string_lossy())
                .send()
                .await
        })?;

        Ok(ObjectAttributes {
            stat: Stat {
                node_type: if path.ends_with("/") {
                    NodeType::Dir
                } else {
                    NodeType::File
                },
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
        let GetObjectOutput { body, .. } = self.rt.block_on(async {
            self.client
                .get_object()
                .bucket(bucket_name)
                .range(format!(
                    "bytes={begin}-{end}",
                    begin = position,
                    end = position + number_of_bytes - 1
                ))
                .key(path.to_string_lossy())
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
        let mut bytes = vec![];

        if position != 0 && position < MINIMUM_PART_SIZE {
            // S3 doesn't allow copying chunks smaller then MINIMUM_PART_SIZE so we need to copy those bytes manually
            bytes = self.read_object(path, bucket_name, 0, position, etag.clone())?;
            position = 0;
        }

        bytes.extend_from_slice(buf);

        let position_after_write = position + bytes.len();
        if bytes.len() < MINIMUM_PART_SIZE && (position_after_write < file_size) {
            // Only the last chunk can be smaller then MINIMUM_PART_SIZE
            // so we need to copy some data from the original file
            let remaining_bytes = self.read_object(
                path,
                bucket_name,
                position + bytes.len(),
                MINIMUM_PART_SIZE - bytes.len(),
                etag.clone(),
            )?;

            bytes.extend(remaining_bytes.into_iter())
        }

        let CreateMultipartUploadOutput { upload_id, key, .. } = self.rt.block_on(async {
            self.client
                .create_multipart_upload()
                .bucket(bucket_name)
                .key(path.to_string_lossy())
                .send()
                .await
        })?;

        let guard = scopeguard::guard((), |_| {
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

        let mut part_number = 1;
        let mut completed_parts = vec![];
        let copy_source = format!("{bucket_name}/{}", path.to_string_lossy());

        if position != 0 {
            execute_by_step(0, position, MAXIMUM_PART_SIZE, |begin, end| {
                let UploadPartCopyOutput {
                    copy_part_result, ..
                } = self.rt.block_on(async {
                    self.client
                        .upload_part_copy()
                        .bucket(bucket_name)
                        .copy_source(&copy_source)
                        .set_copy_source_if_match(etag.clone())
                        .copy_source_range(format!("bytes={begin}-{}", end - 1))
                        .set_key(key.clone())
                        .part_number(part_number)
                        .set_upload_id(upload_id.clone())
                        .send()
                        .await
                })?;

                completed_parts.push(to_completed_part(copy_part_result, part_number));
                part_number += 1;
                Ok(())
            })?;
        }

        let position_after_write = position + bytes.len();

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
                .body(bytes.into())
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

        if position_after_write < file_size {
            execute_by_step(
                position_after_write,
                file_size,
                MAXIMUM_PART_SIZE,
                |begin, end| {
                    let UploadPartCopyOutput {
                        copy_part_result, ..
                    } = self.rt.block_on(async {
                        self.client
                            .upload_part_copy()
                            .bucket(bucket_name)
                            .copy_source(copy_source.clone())
                            .set_copy_source_if_match(etag.clone())
                            .copy_source_range(format!("bytes={begin}-{}", end - 1))
                            .set_key(key.clone())
                            .part_number(part_number)
                            .set_upload_id(upload_id.clone())
                            .send()
                            .await
                    })?;

                    completed_parts.push(to_completed_part(copy_part_result, part_number));
                    part_number += 1;
                    Ok(())
                },
            )?;
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

        scopeguard::ScopeGuard::into_inner(guard); // defuse

        Ok(WriteResp {
            bytes_count: buf.len(),
            etag: e_tag,
        })
    }
}
