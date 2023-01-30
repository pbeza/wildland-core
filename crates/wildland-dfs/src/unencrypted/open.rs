use std::path::Path;
use std::rc::Rc;

use wildland_corex::dfs::interface::{DfsFrontendError, FileHandle, OpenedFileDescriptor};

use super::utils::{fetch_data_from_backends, get_related_nodes};
use super::{NodeDescriptor, UnencryptedDfs};

pub fn open(
    dfs_front: &mut UnencryptedDfs,
    input_exposed_path: String,
) -> Result<FileHandle, DfsFrontendError> {
    let input_exposed_path = Path::new(&input_exposed_path);

    let nodes = get_related_nodes(dfs_front, input_exposed_path)?;

    let mut descriptors: Vec<(&NodeDescriptor, Rc<dyn OpenedFileDescriptor>)> =
        fetch_data_from_backends(&nodes, dfs_front, |backend, path| backend.open(path)).collect();

    match descriptors.len() {
        0 => Err(DfsFrontendError::NoSuchPath),
        1 => Ok(dfs_front.insert_opened_file(descriptors.pop().unwrap().1)),
        _ => todo!(),
    }
}
