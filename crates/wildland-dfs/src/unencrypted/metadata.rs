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

use wildland_corex::dfs::interface::{DfsFrontendError, NodeType, Operation, Stat, WlPermissions};

use super::utils::*;
use super::{NodeDescriptor, UnencryptedDfs};
use crate::events::EventBuilder;
use crate::storage_backends::models::MetadataResponse;

pub fn metadata(
    dfs: &mut UnencryptedDfs,
    input_exposed_path: String,
) -> Result<Stat, DfsFrontendError> {
    let event_builder = EventBuilder::new(dfs.event_system.clone())
        .operation(Operation::Metadata)
        .operation_path(input_exposed_path.clone());

    let get_metadata = |dfs: &mut UnencryptedDfs, node: &NodeDescriptor| match node {
        NodeDescriptor::Physical { storages, .. } => execute_container_operation(
            dfs,
            storages,
            |backend| backend.metadata(storages.path_within_storage()),
            &event_builder,
        )
        .and_then(|resp| match resp {
            MetadataResponse::Found(stat) => Ok(stat),
            MetadataResponse::NotFound => Err(DfsFrontendError::NoSuchPath),
        }),
        NodeDescriptor::Virtual { .. } => Ok(Stat {
            node_type: NodeType::Dir,
            size: 0,
            access_time: None,
            modification_time: None,
            change_time: None,
            permissions: WlPermissions::readonly(),
        }),
    };

    let input_exposed_path = Path::new(&input_exposed_path);
    let nodes = get_related_nodes(dfs, input_exposed_path)?;

    match nodes.as_slice() {
        [] => Err(DfsFrontendError::NoSuchPath),
        [node] => get_metadata(dfs, node),
        _ => {
            let existent_paths: Vec<_> =
                filter_existent_nodes(&nodes, dfs, &event_builder)?.collect();

            match existent_paths.as_slice() {
                [] => Err(DfsFrontendError::NoSuchPath),
                [node] => get_metadata(dfs, node),
                _ => {
                    let exposed_paths = dfs.path_translator.solve_conflicts(existent_paths);
                    let node =
                        find_node_matching_requested_path(input_exposed_path, &exposed_paths);

                    match &node {
                        Some(node) => get_metadata(dfs, node),
                        // Aggregating dir for conflicting files
                        // This behavior does not abstract from conflict resolution methods, so it should be changed when conflict resolution changes
                        None if !exposed_paths.is_empty() => Ok(Stat {
                            node_type: NodeType::Dir,
                            size: 0,
                            access_time: None,
                            modification_time: None,
                            change_time: None,
                            permissions: WlPermissions::readonly(),
                        }),
                        None => Err(DfsFrontendError::NoSuchPath),
                    }
                }
            }
        }
    }
}
