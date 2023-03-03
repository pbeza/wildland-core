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
use wildland_corex::dfs::interface::DfsFrontendError;

use super::{
    assert_is_dir,
    assert_not_found,
    empty_file_setup,
    get_file_length,
    open,
    small_file_setup,
};
use crate::storage_backends::models::SeekFrom;
use crate::storage_backends::s3::backend::S3Backend;
use crate::storage_backends::{CloseOnDropDescriptor, OpenedFileDescriptor, StorageBackend};

mod close {
    use super::*;

    #[rstest]
    fn test_close(empty_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, descriptor) = empty_file_setup;
        descriptor.close().unwrap()
    }
}

mod read {
    use super::*;

    #[rstest]
    fn test_read_empty_file(empty_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = empty_file_setup;
        let resp = descriptor.read(10).unwrap();
        assert_eq!(resp, Vec::<u8>::new());
    }

    #[rstest]
    fn test_read_full_file(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (s3_backend, mut descriptor) = small_file_setup;
        let length = get_file_length(&s3_backend, "file.txt");

        let resp = descriptor.read(length).unwrap();
        assert_eq!(resp, vec![10; length]);
    }

    #[rstest]
    fn test_read_more_then_file_length_should_return_full_file(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = small_file_setup;
        let length = get_file_length(&s3_backend, "file.txt");

        let resp = descriptor.read(length + 100).unwrap();
        assert_eq!(resp, vec![10; length]);
    }

    #[rstest]
    fn test_read_file_in_chunks(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (s3_backend, mut descriptor) = small_file_setup;
        let length = get_file_length(&s3_backend, "file.txt");

        let chunk1 = descriptor.read(length / 2).unwrap();
        let chunk2 = descriptor.read(length / 2).unwrap();
        assert_eq!([chunk1, chunk2].concat(), vec![10; length]);
    }

    #[rstest]
    fn test_read_file_after_reading_full_file_returns_empty_vec(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = small_file_setup;
        let length = get_file_length(&s3_backend, "file.txt");
        descriptor.read(length).unwrap();

        let resp = descriptor.read(5).unwrap();
        assert_eq!(resp, Vec::<u8>::new());
    }

    #[rstest]
    fn test_fail_to_read_file_cant_read_removed_file(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = small_file_setup;
        s3_backend.remove_file(Path::new("file.txt")).unwrap();

        let resp = descriptor.read(5);
        assert_eq!(resp, Err(DfsFrontendError::ConcurrentIssue));
    }

    #[rstest]
    fn test_fail_to_read_file_file_was_replaced_by_dir(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = small_file_setup;
        s3_backend.remove_file(Path::new("file.txt")).unwrap();
        s3_backend.create_dir(Path::new("file.txt")).unwrap();

        let resp = descriptor.read(5);
        assert_eq!(resp, Err(DfsFrontendError::ConcurrentIssue));
    }

    #[rstest]
    fn test_fail_to_read_file_file_was_overridden(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = small_file_setup;
        s3_backend.create_file(Path::new("file.txt")).unwrap();

        let resp = descriptor.read(5);
        assert_eq!(resp, Err(DfsFrontendError::ConcurrentIssue));
    }

    #[rstest]
    fn test_fail_to_read_file_file_was_updated_by_other_descriptor(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = small_file_setup;
        let mut descriptor2 = open(&s3_backend, "file.txt");
        descriptor2.write(&[1, 2, 3]).unwrap();

        let resp = descriptor.read(5);
        assert_eq!(resp, Err(DfsFrontendError::ConcurrentIssue));
    }
}

mod write {
    use super::*;

    #[rstest]
    fn test_write_empty_file(empty_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (s3_backend, mut descriptor) = empty_file_setup;
        let resp = descriptor.write(&[1, 2, 3]).unwrap();
        assert_eq!(resp, 3);

        let mut descriptor2 = open(&s3_backend, "file.txt");
        let resp = descriptor2.read(5).unwrap();
        assert_eq!(resp, vec![1, 2, 3]);
    }

    #[rstest]
    fn test_write_beggining(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (s3_backend, mut descriptor) = small_file_setup;
        let resp = descriptor.write(&[1, 2, 3]).unwrap();
        assert_eq!(resp, 3);

        let mut descriptor2 = open(&s3_backend, "file.txt");
        let resp = descriptor2.read(20).unwrap();
        assert_eq!(resp, vec![1, 2, 3, 10, 10, 10, 10, 10, 10, 10]);
    }

    #[rstest]
    fn test_write_middle(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (s3_backend, mut descriptor) = small_file_setup;
        descriptor.seek(SeekFrom::Start { offset: 5 }).unwrap();

        let resp = descriptor.write(&[1, 2, 3]).unwrap();
        assert_eq!(resp, 3);

        let mut descriptor2 = open(&s3_backend, "file.txt");
        let resp = descriptor2.read(20).unwrap();
        assert_eq!(resp, vec![10, 10, 10, 10, 10, 1, 2, 3, 10, 10]);
    }

    #[rstest]
    fn test_write_end(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (s3_backend, mut descriptor) = small_file_setup;
        descriptor.seek(SeekFrom::Start { offset: 7 }).unwrap();

        let resp = descriptor.write(&[1, 2, 3]).unwrap();
        assert_eq!(resp, 3);

        let mut descriptor2 = open(&s3_backend, "file.txt");
        let resp = descriptor2.read(20).unwrap();
        assert_eq!(resp, vec![10, 10, 10, 10, 10, 10, 10, 1, 2, 3]);
    }

    #[rstest]
    fn test_write_append_data(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (s3_backend, mut descriptor) = small_file_setup;
        descriptor.seek(SeekFrom::End { offset: 0 }).unwrap();

        let resp = descriptor.write(&[1, 2, 3]).unwrap();
        assert_eq!(resp, 3);

        let mut descriptor2 = open(&s3_backend, "file.txt");
        let resp = descriptor2.read(20).unwrap();
        assert_eq!(resp, vec![10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 1, 2, 3]);
    }

    #[rstest]
    fn test_fail_to_write_file_was_updated_by_other_descriptor(
        empty_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = empty_file_setup;
        let mut descriptor2 = open(&s3_backend, "file.txt");
        descriptor2.write(&[1, 2, 3]).unwrap();

        let resp = descriptor.write(&[10, 10, 10]);
        assert_eq!(resp, Err(DfsFrontendError::ConcurrentIssue));

        let mut descriptor2 = open(&s3_backend, "file.txt");
        let resp = descriptor2.read(5).unwrap();
        assert_eq!(resp, vec![1, 2, 3]);
    }

    #[rstest]
    fn test_fail_to_write_file_was_removed(empty_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (s3_backend, mut descriptor) = empty_file_setup;
        s3_backend.remove_file(Path::new("file.txt")).unwrap();

        let resp = descriptor.write(&[10, 10, 10]);
        assert_eq!(resp, Err(DfsFrontendError::ConcurrentIssue));
        assert_not_found(&s3_backend, "file.txt");
    }

    #[rstest]
    fn test_fail_to_write_file_was_replaced_by_dir(
        empty_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = empty_file_setup;
        s3_backend.remove_file(Path::new("file.txt")).unwrap();
        s3_backend.create_dir(Path::new("file.txt")).unwrap();

        let resp = descriptor.write(&[10, 10, 10]);
        assert_eq!(resp, Err(DfsFrontendError::ConcurrentIssue));
        assert_is_dir(&s3_backend, "file.txt");
    }

    #[rstest]
    fn test_fail_to_write_file_was_overridden(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (s3_backend, mut descriptor) = small_file_setup;
        s3_backend.create_file(Path::new("file.txt")).unwrap();

        let resp = descriptor.write(&[10, 10, 10]);
        assert_eq!(resp, Err(DfsFrontendError::ConcurrentIssue));

        let mut descriptor2 = open(&s3_backend, "file.txt");
        let resp = descriptor2.read(5).unwrap();
        assert_eq!(resp, Vec::<u8>::new());
    }
}

mod seek {
    use super::*;

    #[rstest]
    fn test_seek_start(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = small_file_setup;
        let resp = descriptor.seek(SeekFrom::Start { offset: 5 }).unwrap();
        assert_eq!(resp, 5);

        let resp = descriptor.read(10).unwrap();
        assert_eq!(resp.len(), 5);
    }

    #[rstest]
    fn test_seek_current_forward(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = small_file_setup;
        let resp = descriptor.seek(SeekFrom::Current { offset: 5 }).unwrap();
        assert_eq!(resp, 5);

        let resp = descriptor.read(10).unwrap();
        assert_eq!(resp.len(), 5);
    }

    #[rstest]
    fn test_seek_current_back(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = small_file_setup;
        descriptor.seek(SeekFrom::Start { offset: 5 }).unwrap();
        let resp = descriptor.seek(SeekFrom::Current { offset: -5 }).unwrap();
        assert_eq!(resp, 0);

        let resp = descriptor.read(10).unwrap();
        assert_eq!(resp.len(), 10);
    }

    #[rstest]
    fn test_seek_end(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = small_file_setup;
        descriptor.seek(SeekFrom::Start { offset: 5 }).unwrap();
        let resp = descriptor.seek(SeekFrom::End { offset: -2 }).unwrap();
        assert_eq!(resp, 8);

        let resp = descriptor.read(10).unwrap();
        assert_eq!(resp.len(), 2);
    }

    #[rstest]
    fn test_seek_to_end(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = small_file_setup;
        descriptor.seek(SeekFrom::Start { offset: 5 }).unwrap();
        let resp = descriptor.seek(SeekFrom::End { offset: 0 }).unwrap();
        assert_eq!(resp, 10);

        let resp = descriptor.read(10).unwrap();
        assert_eq!(resp.len(), 0);
    }

    #[rstest]
    fn test_fail_to_seek_start_after_eof(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = small_file_setup;
        let resp = descriptor.seek(SeekFrom::Start { offset: 50 });
        assert_eq!(resp, Err(DfsFrontendError::SeekError));
    }

    #[rstest]
    fn test_fail_to_seek_current_after_eof(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = small_file_setup;
        let resp = descriptor.seek(SeekFrom::Current { offset: 50 });
        assert_eq!(resp, Err(DfsFrontendError::SeekError));
    }

    #[rstest]
    fn test_fail_to_seek_end_after_eof(small_file_setup: (S3Backend, CloseOnDropDescriptor)) {
        let (_, mut descriptor) = small_file_setup;
        let resp = descriptor.seek(SeekFrom::End { offset: 50 });
        assert_eq!(resp, Err(DfsFrontendError::SeekError));
    }

    #[rstest]
    fn test_fail_to_seek_current_before_file_begin(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (_, mut descriptor) = small_file_setup;
        let resp = descriptor.seek(SeekFrom::Current { offset: -50 });
        assert_eq!(resp, Err(DfsFrontendError::SeekError));
    }

    #[rstest]
    fn test_fail_to_seek_end_before_file_begin(
        small_file_setup: (S3Backend, CloseOnDropDescriptor),
    ) {
        let (_, mut descriptor) = small_file_setup;
        let resp = descriptor.seek(SeekFrom::End { offset: -50 });
        assert_eq!(resp, Err(DfsFrontendError::SeekError));
    }
}
