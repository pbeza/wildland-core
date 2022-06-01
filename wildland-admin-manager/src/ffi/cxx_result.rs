use crate::api::{AdminManagerError, AdminManagerResult};
use std::fmt::Debug;

pub struct CxxResult<T>(AdminManagerResult<T>);

impl<T> From<AdminManagerResult<T>> for CxxResult<T> {
    fn from(res: AdminManagerResult<T>) -> Self {
        CxxResult(res)
    }
}

impl<T: Debug> CxxResult<T> {
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    // SWIG treat all references as mutable so there is no need to provide many unwrap methods
    // like e.g. unwrap for &ref and unwrap_mut for &mut ref
    // In C++ though, there is no possibility to obtain mutable reference without additional method
    pub fn unwrap(&self) -> &T {
        self.0.as_ref().unwrap()
    }

    pub fn unwrap_err(&self) -> &AdminManagerError {
        self.0.as_ref().unwrap_err()
    }
}