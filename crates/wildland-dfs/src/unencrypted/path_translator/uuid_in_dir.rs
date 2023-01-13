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
use std::rc::Rc;
use std::str::FromStr;

use itertools::Itertools;
use uuid::Uuid;
use wildland_corex::PathResolver;

use super::PathTranslator;
use crate::unencrypted::NodeDescriptor;

pub struct UuidInDirTranslator {
    path_resolver: Rc<dyn PathResolver>,
}

impl UuidInDirTranslator {
    pub fn new(path_resolver: Rc<dyn PathResolver>) -> Self {
        Self { path_resolver }
    }
}

impl PathTranslator for UuidInDirTranslator {
    fn assign_exposed_paths(
        &self,
        nodes: Vec<NodeDescriptor>,
    ) -> Vec<(NodeDescriptor, Option<PathBuf>)> {
        let counted_abs_paths = nodes.iter().counts_by(|node| node.absolute_path.clone());
        nodes
            .into_iter()
            .map(move |node| {
                let abs_path = node.absolute_path.clone();

                // If another file tries to claim the same path
                if counted_abs_paths.get(&abs_path).unwrap() > &1
                // or any physical node has the same path as some virtual node
                || self.path_resolver.is_virtual_nodes(&abs_path)
                {
                    // then append uuid to avoid conflict
                    let exposed_path =
                        abs_path.join(node.storages.as_ref().map_or(PathBuf::new(), |s| {
                            PathBuf::from_str(s.uuid.to_string().as_str()).unwrap()
                        }));
                    (node, Some(exposed_path))
                } else {
                    // otherwise, in case there is no conflicts, expose as the same path
                    (node, Some(abs_path))
                }
            })
            .collect_vec()
    }

    fn exposed_to_absolute_path(&self, path: &Path) -> PathBuf {
        pop_uuid_from_path(path)
    }
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

    #[test]
    fn test_pop_uuid_from_path() {
        let expected = PathBuf::from_str("/a/b/file").unwrap();
        let result =
            pop_uuid_from_path(Path::new("/a/b/file/00000000-0000-0000-0000-000000000002"));
        assert_eq!(expected, result);
    }
}
