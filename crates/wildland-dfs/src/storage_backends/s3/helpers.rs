use std::path::Path;

use aws_sdk_s3::model::{CompletedPart, CopyPartResult};

use super::error::S3Error;

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

pub fn remove_trailing_slash(path: &Path) -> String {
    let path = path.to_string_lossy().to_string();
    path.strip_suffix('/')
        .map(ToOwned::to_owned)
        .unwrap_or(path)
}

pub fn add_trailing_slash(path: &Path) -> String {
    let path = path.to_string_lossy().to_string();
    if !path.ends_with('/') {
        format!("{path}/")
    } else {
        path
    }
}
