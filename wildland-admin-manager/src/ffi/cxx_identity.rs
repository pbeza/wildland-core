use super::{cxx_option::CxxOption, cxx_result::CxxResult};
use crate::api::Identity;
use std::sync::Arc;

pub type IdentityResult = CxxResult<DynIdentity>;
pub type OptionalIdentity = CxxOption<DynIdentity>;

// TODO derive macro
#[derive(Debug)]
pub struct DynIdentity(pub Arc<dyn Identity>);
impl DynIdentity {
    pub fn set_name(&mut self, name: String) {
        let inner = unsafe { Arc::get_mut_unchecked(&mut self.0) };
        inner.set_name(name)
    }

    pub fn get_name(&self) -> String {
        self.0.get_name()
    }
}
