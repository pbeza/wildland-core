use crate::api::{AdminManagerError, AdminManagerResult};
use std::fmt::Debug;

#[derive(Debug)]
pub struct Res<T>(AdminManagerResult<T>);

impl<T> From<AdminManagerResult<T>> for Res<T> {
    fn from(res: AdminManagerResult<T>) -> Self {
        Res(res)
    }
}

impl<T: Debug + Clone> Res<T> {
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    // SWIG treat all references as mutable so there is no need to provide many unwrap methods
    // like e.g. unwrap for &ref and unwrap_mut for &mut ref
    // In C++ though, there is no possibility to obtain mutable reference without additional method
    #[cfg(feature = "bindings")]
    pub fn unwrap(&self) -> Box<T> {
        Box::new(self.inner_unwrap())
    }
    #[cfg(feature = "swift-bridge")]
    pub fn unwrap(&self) -> T {
        self.inner_unwrap()
    }

    fn inner_unwrap(&self) -> T {
        self.0.as_ref().unwrap().clone()
    }

    #[cfg(feature = "bindings")]
    pub fn unwrap_err(&self) -> Box<AdminManagerError> {
        Box::new(self.inner_unwrap_err())
    }
    #[cfg(feature = "swift-bridge")]
    pub fn unwrap_err(&self) -> AdminManagerError {
        self.inner_unwrap_err()
    }

    fn inner_unwrap_err(&self) -> AdminManagerError {
        self.0.as_ref().unwrap_err().clone()
    }
}
