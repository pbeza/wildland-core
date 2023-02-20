use std::path::{Component, Components, Path};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use wildland_corex::dfs::interface::UnixTimestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FileSystemNode {
    Directory {
        name: String,
        children: Vec<FileSystemNode>,
        modification_time: UnixTimestamp,
    },
    File {
        name: String,
        object_name: String,
        size: usize,
        e_tag: String,
        modification_time: UnixTimestamp,
    },
}

impl FileSystemNode {
    pub fn name(&self) -> &str {
        match self {
            FileSystemNode::Directory { name, .. } => name,
            FileSystemNode::File { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystem {
    root_node: FileSystemNode,
}

impl Default for FileSystem {
    fn default() -> Self {
        Self {
            root_node: FileSystemNode::Directory {
                name: "root".into(),
                children: Vec::new(),
                modification_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|duration| UnixTimestamp {
                        sec: duration.as_secs(),
                        nano_sec: duration.subsec_nanos(),
                    })
                    .unwrap(),
            },
        }
    }
}

impl FileSystem {
    pub fn get_node(&mut self, path: &Path) -> Option<&mut FileSystemNode> {
        let mut components = path.components();

        fn visit_node<'a>(
            node: &'a mut FileSystemNode,
            components: &mut Components,
        ) -> Option<&'a mut FileSystemNode> {
            match components.next() {
                Some(Component::RootDir) => None,
                Some(Component::CurDir) => visit_node(node, components),
                Some(Component::Normal(node_name)) => match node {
                    FileSystemNode::Directory { children, .. } => children
                        .iter_mut()
                        .find(|node| node.name() == node_name)
                        .and_then(|node| visit_node(node, components)),
                    FileSystemNode::File { .. } => None,
                },
                Some(Component::ParentDir) => None,
                Some(Component::Prefix(_)) => None,
                None => Some(node),
            }
        }

        loop {
            match components.next() {
                Some(Component::RootDir | Component::CurDir) => continue,
                Some(Component::Normal(node_name)) => {
                    return match &mut self.root_node {
                        FileSystemNode::Directory { children, .. } => children
                            .iter_mut()
                            .find(|node| node.name() == node_name)
                            .and_then(|node| visit_node(node, &mut components)),
                        FileSystemNode::File { .. } => None,
                    }
                }
                Some(Component::ParentDir) => return None,
                Some(Component::Prefix(_)) => return None,
                None => return Some(&mut self.root_node),
            }
        }
    }
}
