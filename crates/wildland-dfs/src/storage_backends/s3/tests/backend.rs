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

use std::path::Path;

use rstest::rstest;
use wildland_corex::dfs::interface::{NodeType, Stat};

use super::{assert_is_dir, assert_is_file, assert_not_found, s3_backend};
use crate::storage_backends::models::{
    CreateDirResponse,
    CreateFileResponse,
    MetadataResponse,
    OpenResponse,
    ReadDirResponse,
    RemoveDirResponse,
    RemoveFileResponse,
};
use crate::storage_backends::s3::backend::S3Backend;
use crate::storage_backends::StorageBackend;

mod read_dir {
    use super::*;

    #[rstest]
    fn test_read_dir_read_root(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.read_dir(Path::new("/")).unwrap();
        assert_eq!(resp, ReadDirResponse::Entries(vec!["foo".into()]));
    }

    #[rstest]
    fn test_read_dir_read_dir(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();
        s3_backend.create_file(Path::new("/foo/bar")).unwrap();

        let resp = s3_backend.read_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, ReadDirResponse::Entries(vec!["bar".into()]));
    }

    #[rstest]
    fn test_read_dir_content_of_nested_dirs_is_not_visible(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();
        s3_backend.create_file(Path::new("/foo/bar")).unwrap();

        let resp = s3_backend.read_dir(Path::new("/")).unwrap();
        assert_eq!(resp, ReadDirResponse::Entries(vec!["foo".into()]));
    }

    #[rstest]
    fn test_read_dir_content_of_upper_dirs_is_not_visible(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();
        s3_backend.create_file(Path::new("/bar")).unwrap();

        let resp = s3_backend.read_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, ReadDirResponse::Entries(Vec::new()));
    }

    #[rstest]
    fn test_fail_to_read_dir_unknown_path(s3_backend: S3Backend) {
        let resp = s3_backend.read_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, ReadDirResponse::NoSuchPath);
    }

    #[rstest]
    fn test_fail_to_read_dir_path_is_file(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.read_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, ReadDirResponse::NotADirectory);
    }
}

mod metadata {
    use super::*;

    #[rstest]
    fn test_get_metadata_of_file(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.metadata(Path::new("/foo")).unwrap();
        assert!(matches!(
            resp,
            MetadataResponse::Found(Stat {
                node_type: NodeType::File,
                size: 0,
                ..
            })
        ));
    }

    #[rstest]
    fn test_get_metadata_of_dir(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();

        let resp = s3_backend.metadata(Path::new("/foo")).unwrap();
        assert!(matches!(
            resp,
            MetadataResponse::Found(Stat {
                node_type: NodeType::Dir,
                ..
            })
        ));
    }

    #[rstest]
    fn test_get_metadata_of_root(s3_backend: S3Backend) {
        let resp = s3_backend.metadata(Path::new("/")).unwrap();
        assert!(matches!(
            resp,
            MetadataResponse::Found(Stat {
                node_type: NodeType::Dir,
                ..
            })
        ));
    }

    #[rstest]
    fn test_fail_to_get_metadata_unknown_path(s3_backend: S3Backend) {
        let resp = s3_backend.metadata(Path::new("/foo")).unwrap();
        assert_eq!(resp, MetadataResponse::NotFound);
    }
}

mod open {
    use super::*;

    #[rstest]
    fn test_open(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();
        let resp = s3_backend.open(Path::new("/foo")).unwrap();
        assert!(matches!(resp, OpenResponse::Found(_)));
    }

    #[rstest]
    fn test_fail_to_open_dir(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();
        let resp = s3_backend.open(Path::new("/foo")).unwrap();
        assert!(matches!(resp, OpenResponse::NotAFile));
    }

    #[rstest]
    fn test_fail_to_open_file_not_found(s3_backend: S3Backend) {
        let resp = s3_backend.open(Path::new("/foo")).unwrap();
        assert!(matches!(resp, OpenResponse::NotFound));
    }
}

mod create_dir {
    use super::*;

    #[rstest]
    fn test_create_dir(s3_backend: S3Backend) {
        let resp = s3_backend.create_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, CreateDirResponse::Created);
        assert_is_dir(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_fail_to_create_dir_cant_create_root(s3_backend: S3Backend) {
        let resp = s3_backend.create_dir(Path::new("/")).unwrap();
        assert_eq!(resp, CreateDirResponse::InvalidParent);
    }

    #[rstest]
    fn test_fail_to_create_dir_path_taken_by_other_dir(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();

        let resp = s3_backend.create_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, CreateDirResponse::PathAlreadyExists);
        assert_is_dir(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_fail_to_create_dir_path_taken_by_other_file(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.create_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, CreateDirResponse::PathAlreadyExists);
        assert_is_file(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_fail_to_create_dir_parent_does_not_exist(s3_backend: S3Backend) {
        let resp = s3_backend.create_dir(Path::new("/foo/bar")).unwrap();
        assert_eq!(resp, CreateDirResponse::InvalidParent);
        assert_not_found(&s3_backend, "/foo");
        assert_not_found(&s3_backend, "/foo/bar");
    }
}

mod remove_dir {
    use super::*;

    #[rstest]
    fn test_remove_dir(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();

        let resp = s3_backend.remove_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, RemoveDirResponse::Removed);
        assert_not_found(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_fail_to_remove_dir_cant_remove_root(s3_backend: S3Backend) {
        let resp = s3_backend.remove_dir(Path::new("/")).unwrap();
        assert_eq!(resp, RemoveDirResponse::RootRemovalNotAllowed);
        assert_is_dir(&s3_backend, "/");
    }

    #[rstest]
    fn test_fail_to_remove_dir_dir_not_empty(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();
        s3_backend.create_file(Path::new("/foo/bar")).unwrap();

        let resp = s3_backend.remove_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, RemoveDirResponse::DirNotEmpty);
        assert_is_dir(&s3_backend, "/foo");
        assert_is_file(&s3_backend, "/foo/bar");
    }

    #[rstest]
    fn test_fail_to_remove_dir_path_is_file(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.remove_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, RemoveDirResponse::NotADirectory);
        assert_is_file(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_fail_to_remove_dir_not_found(s3_backend: S3Backend) {
        let resp = s3_backend.remove_dir(Path::new("/foo")).unwrap();
        assert_eq!(resp, RemoveDirResponse::NotFound);
    }
}

mod path_exists {
    use super::*;

    #[rstest]
    fn test_path_exists_file(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.path_exists(Path::new("/foo")).unwrap();
        assert!(resp);
    }

    #[rstest]
    fn test_path_exists_dir(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();

        let resp = s3_backend.path_exists(Path::new("/foo")).unwrap();
        assert!(resp);
    }

    #[rstest]
    fn test_path_exists_root(s3_backend: S3Backend) {
        let resp = s3_backend.path_exists(Path::new("/")).unwrap();
        assert!(resp);
    }

    #[rstest]
    fn test_path_exists_no_entry(s3_backend: S3Backend) {
        let resp = s3_backend.path_exists(Path::new("/foo")).unwrap();
        assert!(!resp);
    }
}

mod remove_file {
    use super::*;

    #[rstest]
    fn test_remove_file(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.remove_file(Path::new("/foo")).unwrap();
        assert_eq!(resp, RemoveFileResponse::Removed);
        assert_not_found(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_fail_to_remove_file_not_found(s3_backend: S3Backend) {
        let resp = s3_backend.remove_file(Path::new("/foo")).unwrap();
        assert_eq!(resp, RemoveFileResponse::NotFound);
    }

    #[rstest]
    fn test_fail_to_remove_file_path_is_dir(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();

        let resp = s3_backend.remove_file(Path::new("/foo")).unwrap();
        assert_eq!(resp, RemoveFileResponse::NotAFile);
        assert_is_dir(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_fail_to_remove_file_parent_does_not_exist(s3_backend: S3Backend) {
        let resp = s3_backend.remove_file(Path::new("/foo/bar")).unwrap();
        assert_eq!(resp, RemoveFileResponse::NotFound);
    }
}

mod create_file {
    use super::*;

    #[rstest]
    fn test_create_file(s3_backend: S3Backend) {
        let resp = s3_backend.create_file(Path::new("/foo")).unwrap();
        assert!(matches!(resp, CreateFileResponse::Created(_)));
        assert_is_file(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_create_file_by_replacing_old_file(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.create_file(Path::new("/foo")).unwrap();
        assert!(matches!(resp, CreateFileResponse::Created(_)));
        assert_is_file(&s3_backend, "/foo");
    }

    #[rstest]
    fn test_fail_to_create_file_no_parent_dir(s3_backend: S3Backend) {
        let resp = s3_backend.create_file(Path::new("/foo/bar")).unwrap();
        assert!(matches!(resp, CreateFileResponse::InvalidParent));
        assert_not_found(&s3_backend, "/foo");
        assert_not_found(&s3_backend, "/foo/bar");
    }

    #[rstest]
    fn test_fail_to_create_file_parent_is_file(s3_backend: S3Backend) {
        s3_backend.create_file(Path::new("/foo")).unwrap();

        let resp = s3_backend.create_file(Path::new("/foo/bar")).unwrap();
        assert!(matches!(resp, CreateFileResponse::InvalidParent));
        assert_not_found(&s3_backend, "/foo/bar");
    }

    #[rstest]
    fn test_fail_to_create_file_path_is_dir(s3_backend: S3Backend) {
        s3_backend.create_dir(Path::new("/foo")).unwrap();

        let resp = s3_backend.create_file(Path::new("/foo")).unwrap();
        assert!(matches!(resp, CreateFileResponse::PathTakenByDir));
        assert_is_dir(&s3_backend, "/foo");
    }
}
