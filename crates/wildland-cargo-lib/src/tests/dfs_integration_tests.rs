use std::collections::HashSet;

use pretty_assertions::assert_eq;
use rstest::rstest;
use wildland_corex::{LocalSecureStorage, StorageTemplate};
use wildland_dfs::{DfsFrontendError, NodeType};
use wildland_lfs::template::LocalFilesystemStorageTemplate;

use crate::api::cargo_lib::create_cargo_lib;
use crate::api::CargoConfig;
use crate::utils::test::lss_stub;

#[rstest]
fn dfs_integration_test_with_containers_with_lfs_storages(
    lss_stub: &'static dyn LocalSecureStorage,
) {
    //
    // Given containers with data
    //
    let tmpdir = tempfile::tempdir().unwrap().into_path();

    let config_str = r#"{
        "log_level": "trace",
        "log_use_ansi": false,
        "log_file_enabled": true,
        "log_file_path": "cargo_lib_log",
        "log_file_rotate_directory": ".",
        "evs_url": "some_url",
        "sc_url": "some_url"
    }"#;
    let cfg: CargoConfig = serde_json::from_str(config_str).unwrap();

    let cargo_lib = create_cargo_lib(lss_stub, cfg).unwrap();
    let cargo_lib = cargo_lib.lock().unwrap();
    let user_api = cargo_lib.user_api();
    let mnemonic = user_api.generate_mnemonic().unwrap();
    let user = user_api
        .create_user_from_mnemonic(&mnemonic, "device_name".to_string())
        .unwrap();

    let template = LocalFilesystemStorageTemplate {
        local_dir: tmpdir.clone(),
        container_prefix: "{{ CONTAINER_NAME }}".to_owned(),
    };
    let container1 = user
        .create_container(
            "C1".to_owned(),
            &StorageTemplate::try_new("LocalFilesystem", template.clone()).unwrap(),
            "/some/path/".to_owned(),
        )
        .unwrap();
    let container2 = user
        .create_container(
            "C2".to_owned(),
            &StorageTemplate::try_new("LocalFilesystem", template.clone()).unwrap(),
            "/some/path/dir".to_owned(),
        )
        .unwrap();
    let container3 = user
        .create_container(
            "C3".to_owned(),
            &StorageTemplate::try_new("LocalFilesystem", template).unwrap(),
            "/some/path/other_dir".to_owned(),
        )
        .unwrap();

    std::fs::create_dir(tmpdir.join("C1")).unwrap();
    std::fs::create_dir(tmpdir.join("C1/dir")).unwrap();
    std::fs::File::create(tmpdir.join("C1/dir/c1_file")).unwrap();

    std::fs::create_dir(tmpdir.join("C2")).unwrap();
    std::fs::File::create(tmpdir.join("C2/c2_file")).unwrap();

    std::fs::create_dir(tmpdir.join("C3")).unwrap();
    std::fs::File::create(tmpdir.join("C3/c3_file")).unwrap();

    let dfs = cargo_lib.dfs_api();
    let mut dfs = dfs.lock().unwrap();

    let entries = dfs.readdir("/some/path/".to_string()).unwrap();
    assert_eq!(entries, Vec::<String>::new());

    //
    // When containers are mounted
    //
    user.mount(&container1).unwrap();
    user.mount(&container2).unwrap();
    user.mount(&container3).unwrap();

    //
    // Then data is accessible via DFS
    //
    let entries: HashSet<String> = dfs
        .readdir("/some/path/dir".to_string())
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(
        entries,
        HashSet::from([
            "/some/path/dir/c2_file".to_owned(),
            "/some/path/dir/c1_file".to_owned()
        ])
    );
    let entries: HashSet<String> = dfs
        .readdir("/some/path/".to_string())
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(
        entries,
        HashSet::from([
            "/some/path/dir/".to_owned(),
            "/some/path/other_dir".to_owned()
        ])
    );
    let c1_file_stat = dfs.getattr("/some/path/dir/c1_file".to_owned()).unwrap();
    assert_eq!(c1_file_stat.node_type, NodeType::File);
    let c1_file_stat = dfs.getattr("/some/path/dir/".to_owned()).unwrap();
    assert_eq!(c1_file_stat.node_type, NodeType::Dir);

    let file = dfs.open("/some/path/dir/c1_file".to_owned()).unwrap();

    dfs.write(&file, vec![1, 2, 3, 4, 5]).unwrap();
    dfs.seek_from_start(&file, 1).unwrap();
    let read_buf = dfs.read(&file, 3).unwrap();
    assert_eq!(read_buf, vec![2, 3, 4]);

    dfs.close(&file).unwrap();

    //
    // And when one container is unmounted
    //
    user.unmount(&container1).unwrap();

    //
    // Then its data is inaccessible
    //
    let entries: HashSet<String> = dfs
        .readdir("/some/path/dir".to_string())
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(
        entries,
        HashSet::from(["/some/path/dir/c2_file".to_owned(),])
    );
    let c1_file_stat_err = dfs
        .getattr("/some/path/dir/c1_file".to_owned())
        .unwrap_err();
    assert_eq!(c1_file_stat_err, DfsFrontendError::NoSuchPath);
}