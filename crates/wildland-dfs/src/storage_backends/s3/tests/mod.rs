//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod backend;
mod descriptor;

use std::path::Path;
use std::rc::Rc;

use aws_sdk_s3::{Client, Credentials, Region};
use rstest::fixture;
use tokio::runtime::Runtime;
use uuid::Uuid;
use wildland_corex::dfs::interface::NodeType;

use super::backend::S3Backend;
use super::client::WildlandS3Client;
use super::connector::build_s3_client;
use crate::storage_backends::models::{CreateFileResponse, MetadataResponse, OpenResponse};
use crate::storage_backends::{CloseOnDropDescriptor, OpenedFileDescriptor, StorageBackend};

struct MinioClient {
    rt: Rc<Runtime>,
    client: Client,
}

#[fixture]
fn minio_url() -> String {
    std::env::var("MINIO_URL").unwrap_or_else(|_| "http://127.0.0.1:9000".into())
}

#[fixture]
fn minio_credentials() -> Credentials {
    Credentials::new("minioadmin", "minioadmin", None, None, "test_client")
}

#[fixture]
#[once]
fn minio_client(minio_url: String, minio_credentials: Credentials) -> MinioClient {
    let region = Region::new("us-east-1");

    MinioClient {
        rt: Rc::new(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        ),
        client: build_s3_client(minio_credentials, region, minio_url.into()),
    }
}

#[fixture]
fn empty_bucket(minio_client: &MinioClient) -> String {
    let bucket_name = Uuid::new_v4().to_string();

    minio_client
        .rt
        .block_on(async {
            minio_client
                .client
                .create_bucket()
                .bucket(bucket_name.clone())
                .send()
                .await
        })
        .unwrap();

    bucket_name
}

#[fixture]
pub fn s3_backend(
    empty_bucket: String,
    minio_url: String,
    minio_credentials: Credentials,
) -> S3Backend {
    let region = Region::new("us-east-1");
    let client = Rc::new(WildlandS3Client::new(
        Rc::new(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        ),
        minio_credentials,
        region,
        minio_url.into(),
    ));

    S3Backend::new(client, empty_bucket)
}

#[fixture]
pub fn empty_file_setup(s3_backend: S3Backend) -> (S3Backend, CloseOnDropDescriptor) {
    let descriptor = match s3_backend.create_file(Path::new("file.txt")).unwrap() {
        CreateFileResponse::Created(descriptor) => descriptor,
        _ => panic!(),
    };
    (s3_backend, descriptor)
}

#[fixture]
pub fn small_file_setup(s3_backend: S3Backend) -> (S3Backend, CloseOnDropDescriptor) {
    let mut descriptor = match s3_backend.create_file(Path::new("file.txt")).unwrap() {
        CreateFileResponse::Created(descriptor) => descriptor,
        _ => panic!(),
    };

    let len = descriptor.write(&[10; 10]).unwrap();
    assert_eq!(len, 10);

    let descriptor = match s3_backend.open(Path::new("file.txt")).unwrap() {
        OpenResponse::Found(descriptor) => descriptor,
        _ => panic!(),
    };

    (s3_backend, descriptor)
}

pub fn get_file_length(s3_backend: &S3Backend, path: &str) -> usize {
    match s3_backend.metadata(Path::new(path)).unwrap() {
        MetadataResponse::Found(stat) => stat.size,
        _ => panic!(),
    }
}

pub fn open(s3_backend: &S3Backend, path: &str) -> CloseOnDropDescriptor {
    match s3_backend.open(Path::new(path)).unwrap() {
        OpenResponse::Found(d) => d,
        _ => panic!(),
    }
}

pub fn assert_is_file(s3_backend: &S3Backend, path: &str) {
    match s3_backend.metadata(Path::new(path)).unwrap() {
        MetadataResponse::Found(stat) => assert_eq!(stat.node_type, NodeType::File),
        _ => panic!(),
    };
}

pub fn assert_is_dir(s3_backend: &S3Backend, path: &str) {
    match s3_backend.metadata(Path::new(path)).unwrap() {
        MetadataResponse::Found(stat) => assert_eq!(stat.node_type, NodeType::Dir),
        _ => panic!(),
    };
}

pub fn assert_not_found(s3_backend: &S3Backend, path: &str) {
    match s3_backend.metadata(Path::new(path)).unwrap() {
        MetadataResponse::NotFound => (),
        _ => panic!(),
    };
}
