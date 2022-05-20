use crate::api::{AdminManagerError, AdminManagerResult};
use std::fmt::Debug;

#[derive(Debug)]
pub struct CxxResult<T: Debug>(AdminManagerResult<T>);

impl<T: Debug> From<AdminManagerResult<T>> for CxxResult<T> {
    fn from(res: AdminManagerResult<T>) -> Self {
        CxxResult(res)
    }
}

impl<T: Clone + std::fmt::Debug> CxxResult<T> {
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    pub fn unwrap(&self) -> &T {
        self.0.as_ref().unwrap()
    }

    pub fn unwrap_err(&self) -> &AdminManagerError {
        self.0.as_ref().unwrap_err()
    }
}
