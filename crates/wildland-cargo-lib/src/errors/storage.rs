use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GetStoragesError {}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DeleteStorageError {}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AddStorageError {}
