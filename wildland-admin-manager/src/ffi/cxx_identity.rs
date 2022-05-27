use super::{cxx_option::CxxOption, cxx_result::CxxResult, rcref::RcRef};
use crate::api::Identity;

pub type IdentityResult = CxxResult<CxxDynIdentity>;
pub type OptionalIdentity = CxxOption<CxxDynIdentity>;

#[derive(Debug)]
pub struct CxxDynIdentity(pub RcRef<dyn Identity>);
impl CxxDynIdentity {
    pub fn set_name(&mut self, name: String) {
        self.0.get_mut().set_name(name);
    }

    pub fn get_name(&self) -> String {
        self.0.deref().get_name()
    }
}
