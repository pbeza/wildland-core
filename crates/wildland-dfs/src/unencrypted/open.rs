use std::path::Path;

use wildland_corex::dfs::interface::{DfsFrontendError, FileDescriptor};

use super::utils::{fetch_data_from_backends, get_related_nodes};
use super::{NodeDescriptor, UnencryptedDfs};

pub fn open(
    dfs_front: &mut UnencryptedDfs,
    input_exposed_path: String,
) -> Result<FileDescriptor, DfsFrontendError> {
    let input_exposed_path = Path::new(&input_exposed_path);

    let nodes = get_related_nodes(dfs_front, input_exposed_path)?;

    let stats: Vec<(&NodeDescriptor, FileDescriptor)> =
        fetch_data_from_backends(&nodes, dfs_front, |backend, path| backend.open(path)).collect();

    todo!()
}
