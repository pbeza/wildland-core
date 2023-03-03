use std::collections::HashMap;
use std::rc::Rc;

use rsfs::mem::FS;
use wildland_corex::MockPathResolver;

use super::{MufsFactory, UnresponsiveFsFactory};
use crate::storage_backends::StorageBackendFactory;
use crate::unencrypted::UnencryptedDfs;

pub mod mufs;
pub mod unresponsive_fs;

pub fn dfs_with_unresponsive_fs(path_resolver: Box<MockPathResolver>) -> UnencryptedDfs {
    let factory = UnresponsiveFsFactory {};
    let mut backend_factories: HashMap<String, Box<dyn StorageBackendFactory>> = HashMap::new();
    backend_factories.insert("UnresponsiveFs".to_string(), Box::new(factory));
    UnencryptedDfs::new(path_resolver, backend_factories)
}

type DfsFixture = (UnencryptedDfs, Rc<FS>);
pub fn dfs_with_mu_fs(path_resolver: Box<MockPathResolver>) -> DfsFixture {
    let fs = Rc::new(FS::new());
    let factory = MufsFactory::new(fs.clone());
    let mut backend_factories: HashMap<String, Box<dyn StorageBackendFactory>> = HashMap::new();
    backend_factories.insert("MUFS".to_string(), Box::new(factory));

    let dfs = UnencryptedDfs::new(path_resolver, backend_factories);
    (dfs, fs)
}

pub fn dfs_with_unresponsive_and_mu_fs(path_resolver: Box<MockPathResolver>) -> UnencryptedDfs {
    let factory = UnresponsiveFsFactory {};
    let mut backend_factories: HashMap<String, Box<dyn StorageBackendFactory>> = HashMap::new();
    backend_factories.insert("UnresponsiveFs".to_string(), Box::new(factory));

    let factory = MufsFactory::new(Rc::default());
    backend_factories.insert("MUFS".to_string(), Box::new(factory));

    UnencryptedDfs::new(path_resolver, backend_factories)
}
