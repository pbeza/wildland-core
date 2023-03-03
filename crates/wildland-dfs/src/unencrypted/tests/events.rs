use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

use mockall::predicate;
use rstest::rstest;
use uuid::Uuid;
use wildland_corex::dfs::interface::{Cause, DfsFrontend, Event, EventReceiver, Operation};
use wildland_corex::{MockPathResolver, ResolvedPath};

use crate::unencrypted::tests::{
    dfs_with_unresponsive_and_mu_fs,
    dfs_with_unresponsive_fs,
    new_mufs_storage,
    new_unresponsive_fs_storage,
};

fn get_all_events(receiver: Arc<Mutex<dyn EventReceiver>>) -> impl Iterator<Item = Event> {
    std::iter::from_fn(move || receiver.lock().unwrap().recv())
}

fn assert_events_eq(receiver: Arc<Mutex<dyn EventReceiver>>, mut expected_events: Vec<Event>) {
    for event in get_all_events(receiver) {
        let index = expected_events
            .iter()
            .position(|expected| expected == &event);
        assert!(index.is_some(), "Unexpected event: {:?}", event);
        expected_events.remove(index.unwrap());
    }

    assert!(
        expected_events.is_empty(),
        "Events not matched: {:?}",
        expected_events
    );
}

#[rstest]
fn test_read_dir_unresponsive_event() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/dir")))
        .times(1)
        .returning(|_path| {
            Ok(HashSet::from([ResolvedPath::PathWithStorages {
                path_within_storage: "/dir".into(),
                storages_id: Uuid::from_u128(1),
                storages: vec![new_unresponsive_fs_storage(), new_mufs_storage("/")],
            }]))
        });

    let path_resolver = Box::new(path_resolver);
    let mut dfs = dfs_with_unresponsive_and_mu_fs(path_resolver);
    let rec = dfs.get_receiver();

    assert!(dfs.read_dir("/dir".to_string()).is_err());
    drop(dfs);

    assert_events_eq(
        rec,
        vec![Event {
            cause: Cause::UnresponsiveBackend,
            operation: Some(Operation::ReadDir),
            operation_path: Some("/dir".into()),
            backend_type: Some("UnresponsiveFs".into()),
        }],
    );
}

#[rstest]
fn test_read_dir_all_backends_unresponsive_event() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/dir")))
        .times(1)
        .returning(|_path| {
            Ok(HashSet::from([ResolvedPath::PathWithStorages {
                path_within_storage: "/dir".into(),
                storages_id: Uuid::from_u128(1),
                storages: vec![new_unresponsive_fs_storage()],
            }]))
        });

    let path_resolver = Box::new(path_resolver);
    let mut dfs = dfs_with_unresponsive_fs(path_resolver);
    let rec = dfs.get_receiver();

    assert!(dfs.read_dir("/dir".to_string()).is_err());
    drop(dfs);

    assert_events_eq(
        rec,
        vec![
            Event {
                cause: Cause::UnresponsiveBackend,
                operation: Some(Operation::ReadDir),
                operation_path: Some("/dir".into()),
                backend_type: Some("UnresponsiveFs".into()),
            },
            Event {
                cause: Cause::AllBackendsUnresponsive,
                operation: Some(Operation::ReadDir),
                operation_path: Some("/dir".into()),
                backend_type: None,
            },
        ],
    );
}

#[rstest]
fn test_read_dir_unsupported_backend_event() {
    let mut path_resolver = MockPathResolver::new();

    path_resolver
        .expect_resolve()
        .with(predicate::eq(Path::new("/dir")))
        .times(1)
        .returning(|_path| {
            Ok(HashSet::from([ResolvedPath::PathWithStorages {
                path_within_storage: "/dir".into(),
                storages_id: Uuid::from_u128(1),
                storages: vec![new_mufs_storage("/")],
            }]))
        });

    let path_resolver = Box::new(path_resolver);
    let mut dfs = dfs_with_unresponsive_fs(path_resolver);
    let rec = dfs.get_receiver();

    assert!(dfs.read_dir("/dir".to_string()).is_err());
    drop(dfs);

    assert_events_eq(
        rec,
        vec![
            Event {
                cause: Cause::UnsupportedBackendType,
                operation: Some(Operation::ReadDir),
                operation_path: Some("/dir".into()),
                backend_type: Some("MUFS".into()),
            },
            Event {
                cause: Cause::AllBackendsUnresponsive,
                operation: Some(Operation::ReadDir),
                operation_path: Some("/dir".into()),
                backend_type: None,
            },
        ],
    );
}
