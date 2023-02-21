use std::path::{Component, Components, Path};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use wildland_corex::dfs::interface::UnixTimestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub name: String,
    pub children: Vec<FileSystemNode>,
    pub modification_time: UnixTimestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub object_name: String,
    pub size: usize,
    pub e_tag: String,
    pub modification_time: UnixTimestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FileSystemNode {
    Directory(Directory),
    File(File),
}

impl From<Directory> for FileSystemNode {
    fn from(value: Directory) -> Self {
        Self::Directory(value)
    }
}

impl From<File> for FileSystemNode {
    fn from(value: File) -> Self {
        Self::File(value)
    }
}

impl FileSystemNode {
    pub fn name(&self) -> &str {
        match self {
            FileSystemNode::Directory(dir) => &dir.name,
            FileSystemNode::File(file) => &file.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystem {
    root_node: Directory,
}

#[derive(Debug)]
pub enum Node<'a> {
    Directory(&'a mut Directory),
    File(&'a mut File),
}

impl<'a> From<&'a mut FileSystemNode> for Node<'a> {
    fn from(value: &'a mut FileSystemNode) -> Self {
        match value {
            FileSystemNode::Directory(dir) => Self::Directory(dir),
            FileSystemNode::File(file) => Self::File(file),
        }
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self {
            root_node: Directory {
                name: "/".into(),
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
    pub fn get_node(&mut self, path: &Path) -> Option<Node> {
        let mut components = path.components();

        fn visit_node<'a>(
            node: &'a mut FileSystemNode,
            components: &mut Components,
        ) -> Option<Node<'a>> {
            match components.next() {
                Some(Component::RootDir) => None,
                Some(Component::CurDir) => visit_node(node, components),
                Some(Component::Normal(node_name)) => match node {
                    FileSystemNode::Directory(dir) => dir
                        .children
                        .iter_mut()
                        .find(|node| node.name() == node_name)
                        .and_then(|node| visit_node(node, components)),
                    FileSystemNode::File { .. } => None,
                },
                Some(Component::ParentDir) => None,
                Some(Component::Prefix(_)) => None,
                None => Some(node.into()),
            }
        }

        loop {
            match components.next() {
                Some(Component::RootDir | Component::CurDir) => continue,
                Some(Component::Normal(node_name)) => {
                    return self
                        .root_node
                        .children
                        .iter_mut()
                        .find(|node| node.name() == node_name)
                        .and_then(|node| visit_node(node, &mut components))
                }
                Some(Component::ParentDir) => return None,
                Some(Component::Prefix(_)) => return None,
                None => return Some(Node::Directory(&mut self.root_node)),
            }
        }
    }
}
