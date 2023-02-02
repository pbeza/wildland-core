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

use wildland_corex::dfs::interface::{DfsFrontendError, NodeType, Stat};

use super::utils::*;
use super::{NodeDescriptor, UnencryptedDfs};
use crate::storage_backends::GetattrResponse;

pub fn getattr(
    dfs_front: &mut UnencryptedDfs,
    input_exposed_path: String,
) -> Result<Stat, DfsFrontendError> {
    let input_exposed_path = Path::new(&input_exposed_path);

    let nodes = get_related_nodes(dfs_front, input_exposed_path)?;

    let mut stats: Vec<(&NodeDescriptor, Stat)> =
        fetch_data_from_containers(&nodes, dfs_front, |backend, path| backend.getattr(path))
            .filter_map(|(node, opt_result)| match opt_result {
                GetattrResponse::Found(stat) => Some((node, stat)),
                GetattrResponse::NotFound => None,
            })
            .chain(nodes.iter().filter_map(|node| {
                if node.is_virtual() {
                    Some((
                        node,
                        Stat {
                            node_type: NodeType::Dir,
                            size: 0,
                            access_time: None,
                            modification_time: None,
                            change_time: None,
                        },
                    ))
                } else {
                    None
                }
            }))
            .collect();

    match stats.len() {
        0 => Err(DfsFrontendError::NoSuchPath),
        1 => Ok(stats.pop().unwrap().1),
        _ => {
            let nodes: Vec<&NodeDescriptor> = stats.iter().map(|(n, _)| *n).collect();
            let exposed_paths = dfs_front.path_translator.solve_conflicts(nodes);
            let node = find_node_matching_requested_path(input_exposed_path, &exposed_paths);
            match node {
                // Physical node
                Some(node @ NodeDescriptor::Physical { .. }) => stats
                    .into_iter()
                    .find_map(|(n, stat)| if n == node { Some(stat) } else { None })
                    .ok_or(DfsFrontendError::NoSuchPath),
                // Virtual node
                Some(_) | None if !exposed_paths.is_empty() => Ok(Stat {
                    node_type: NodeType::Dir,
                    size: 0,
                    access_time: None,
                    modification_time: None,
                    change_time: None,
                }),
                _ => Err(DfsFrontendError::NoSuchPath),
            }
        }
    }
}
