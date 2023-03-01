use anyhow::Context;
use aws_sdk_s3::model::{CompletedPart, CopyPartResult};
use scopeguard::ScopeGuard;
use wildland_corex::dfs::interface::DfsFrontendError;

use super::client::S3Client;
use super::error::S3Error;
use super::file_system::FileSystem;
use crate::storage_backends::models::StorageBackendError;

const FILE_SYSTEM_FILE: &str = "filesystem.wildland";

pub fn to_completed_part(mut result: Option<CopyPartResult>, part_number: i32) -> CompletedPart {
    CompletedPart::builder()
        .set_e_tag(result.as_mut().and_then(|v| v.e_tag.take()))
        .set_checksum_crc32(result.as_mut().and_then(|v| v.checksum_crc32.take()))
        .set_checksum_crc32_c(result.as_mut().and_then(|v| v.checksum_crc32_c.take()))
        .set_checksum_sha1(result.as_mut().and_then(|v| v.checksum_sha1.take()))
        .set_checksum_sha256(result.and_then(|v| v.checksum_sha256))
        .part_number(part_number)
        .build()
}

pub fn execute_by_step<T>(begin: usize, end: usize, step: usize, mut op: T) -> Result<(), S3Error>
where
    T: FnMut(usize, usize) -> Result<(), S3Error>,
{
    if begin >= end {
        return Ok(());
    }

    op(begin, std::cmp::min(begin + step, end))?;
    execute_by_step(begin + step, end, step, op)
}

pub fn load_file_system(
    client: &dyn S3Client,
    bucket_name: &str,
) -> Result<FileSystem, StorageBackendError> {
    match client.read_object(FILE_SYSTEM_FILE, bucket_name, None, None) {
        Ok(body) => serde_json::from_slice(&body)
            .context("Filesystem metadata deserialization error")
            .map_err(StorageBackendError::Generic),
        Err(S3Error::NotFound) => Ok(Default::default()),
        Err(err @ (S3Error::ETagMistmach | S3Error::Generic(_))) => {
            Err(StorageBackendError::Generic(err.into()))
        }
    }
}

pub fn commit_file_system(
    client: &dyn S3Client,
    bucket_name: &str,
    file_system: FileSystem,
) -> Result<(), StorageBackendError> {
    // Prone to "lost write" anomaly
    client
        .save_object(
            FILE_SYSTEM_FILE,
            bucket_name,
            serde_json::to_vec(&file_system)
                .map_err(|err| StorageBackendError::Generic(err.into()))?,
        )
        .map_err(|err| StorageBackendError::Generic(err.into()))
}

pub fn defuse<T, F, S>(guard: ScopeGuard<T, F, S>)
where
    F: FnOnce(T),
    S: scopeguard::Strategy,
{
    scopeguard::ScopeGuard::into_inner(guard);
}

pub fn map_conccurent_operation_error(err: S3Error) -> DfsFrontendError {
    match err {
        S3Error::NotFound => DfsFrontendError::ConcurrentIssue,
        S3Error::ETagMistmach => DfsFrontendError::ConcurrentIssue,
        S3Error::Generic(err) => DfsFrontendError::Generic(format!("{err:?}")),
    }
}
