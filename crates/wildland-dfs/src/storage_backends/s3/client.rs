use std::path::{Path, PathBuf};
use std::rc::Rc;

use aws_sdk_s3::output::HeadObjectOutput;
use aws_sdk_s3::{Client, Credentials, Region};
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;
use wildland_corex::dfs::interface::{NodeType, Stat, UnixTimestamp};

use super::connector::build_s3_client;
use super::error::S3Error;

#[cfg_attr(test, mockall::automock)]
pub trait S3Client {
    fn list_files(&self, path: &Path, bucket_name: &str) -> Result<Vec<PathBuf>, S3Error>;
    fn get_object_attributes(&self, path: &Path, bucket_name: &str) -> Result<Stat, S3Error>;
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
    fn list_files(&self, path: &Path, bucket_name: &str) -> Result<Vec<PathBuf>, S3Error> {
        let path = path.to_string_lossy().to_string();

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

    fn get_object_attributes(&self, path: &Path, bucket_name: &str) -> Result<Stat, S3Error> {
        let HeadObjectOutput {
            last_modified,
            content_length,
            ..
        } = self.rt.block_on(async {
            self.client
                .head_object()
                .bucket(bucket_name)
                .key(path.to_string_lossy())
                .send()
                .await
        })?;

        Ok(Stat {
            node_type: NodeType::File, // TODO: COR-23 implement with write
            size: content_length as _,
            access_time: None, // TODO: COR-23 implement with read
            modification_time: last_modified.map(|time| UnixTimestamp {
                sec: time.secs() as _,
                nano_sec: time.subsec_nanos(),
            }),
            change_time: None,
        })
    }
}
