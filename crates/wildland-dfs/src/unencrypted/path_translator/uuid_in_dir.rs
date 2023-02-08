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

use std::path::{Path, PathBuf};
use std::str::FromStr;

use itertools::Itertools;
use uuid::Uuid;

use super::PathConflictResolver;
use crate::unencrypted::NodeDescriptor;

pub struct UuidInDirTranslator {}

impl UuidInDirTranslator {
    pub fn new() -> Self {
        Self {}
    }
}

impl PathConflictResolver for UuidInDirTranslator {
    // Caller must provide all nodes that may collide with each other in context of operation (like `readdir`).
    // PathTranslator is not able to retrieve this data, cause PathResolver has no information about files in
    // containers' storages.
    fn solve_conflicts<'a>(
        &self,
        nodes: Vec<&'a NodeDescriptor>,
    ) -> Vec<(&'a NodeDescriptor, PathBuf)> {
        let counted_abs_paths = nodes.iter().counts_by(|node| node.abs_path());
        nodes
            .iter()
            .map(|node| {
                let abs_path = node.abs_path();

                // If another file tries to claim the same path
                if counted_abs_paths.get(&abs_path).unwrap() > &1
                // or a physical node has the same path as some virtual node
                || (node.is_physical() && conflicts_with_virtual_path(node.abs_path(), &nodes))
                {
                    // then append uuid to avoid conflict
                    let exposed_path = abs_path.join(node.storages().map_or(PathBuf::new(), |s| {
                        PathBuf::from_str(s.uuid().to_string().as_str()).unwrap()
                    }));
                    (*node, exposed_path)
                } else {
                    // otherwise, in case there is no conflicts, expose as the same path
                    (*node, abs_path.into())
                }
            })
            .collect_vec()
    }

    fn exposed_to_absolute_path(&self, path: &Path) -> PathBuf {
        pop_uuid_from_path(path)
    }
}

fn conflicts_with_virtual_path(physical_node_abs_path: &Path, nodes: &[&NodeDescriptor]) -> bool {
    nodes
        .iter()
        .filter(|node| node.is_virtual())
        .any(|node| node.abs_path().starts_with(physical_node_abs_path))
}

fn pop_uuid_from_path(path: &Path) -> PathBuf {
    match path.file_name() {
        Some(file_name) if Uuid::parse_str(&file_name.to_string_lossy()).is_ok() => {
            let mut path = PathBuf::from(path);
            path.pop();
            path
        }
        _ => path.into(),
    }
}

#[cfg(test)]
mod unit_tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::unencrypted::node_descriptor::NodeStorages;
    use crate::unencrypted::tests::new_mufs_storage;

    #[test]
    fn test_pop_uuid_from_path() {
        let expected = PathBuf::from_str("/a/b/file").unwrap();
        let result =
            pop_uuid_from_path(Path::new("/a/b/file/00000000-0000-0000-0000-000000000002"));
        assert_eq!(expected, result);
    }

    #[test]
    fn test_assigning_single_physical_path() {
        let mufs_storage = new_mufs_storage("/");

        let translator = UuidInDirTranslator::new();

        let node = NodeDescriptor::Physical {
            storages: NodeStorages::new(vec![mufs_storage], "/file".into(), Uuid::from_u128(1)),
            absolute_path: "/file".into(),
        };
        let nodes = vec![&node];
        let exposed_paths = translator.solve_conflicts(nodes);

        assert_eq!(exposed_paths, vec![(&node, "/file".into())])
    }

    #[test]
    fn test_assigning_colliding_physical_paths() {
        let storage1 = new_mufs_storage("/storage1/");
        let storage2 = new_mufs_storage("/storage2/");

        let translator = UuidInDirTranslator::new();

        let node1 = NodeDescriptor::Physical {
            storages: NodeStorages::new(vec![storage1], "/file".into(), Uuid::from_u128(1)),
            absolute_path: "/file".into(),
        };
        let node2 = NodeDescriptor::Physical {
            storages: NodeStorages::new(vec![storage2], "/file".into(), Uuid::from_u128(2)),
            absolute_path: "/file".into(),
        };
        let nodes = vec![&node1, &node2];
        let exposed_paths = translator.solve_conflicts(nodes);

        assert_eq!(
            exposed_paths,
            vec![
                (&node1, "/file/00000000-0000-0000-0000-000000000001".into()),
                (&node2, "/file/00000000-0000-0000-0000-000000000002".into())
            ]
        )
    }

    #[test]
    fn test_assigning_colliding_physical_and_virtual_paths() {
        let storage1 = new_mufs_storage("/storage1/");

        let translator = UuidInDirTranslator::new();

        let node1 = NodeDescriptor::Physical {
            storages: NodeStorages::new(vec![storage1], "/a".into(), Uuid::from_u128(1)),
            absolute_path: "/a".into(),
        };
        let node2 = NodeDescriptor::Virtual {
            absolute_path: "/a".into(),
        };
        let nodes = vec![&node1, &node2];
        let exposed_paths = translator.solve_conflicts(nodes);

        assert_eq!(
            exposed_paths,
            vec![
                (&node1, "/a/00000000-0000-0000-0000-000000000001".into()),
                (&node2, "/a/".into())
            ]
        )
    }
}
