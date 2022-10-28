use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ContainerMountError {}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ContainerUnmountError {}
