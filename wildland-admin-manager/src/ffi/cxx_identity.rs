use super::{cxx_option::CxxOption, cxx_result::CxxResult};
use crate::api::Identity;

pub type DynIdentity = Box<dyn Identity>;
pub type IdentityResult = CxxResult<DynIdentity>;
pub type OptionalIdentity<'a> = CxxOption<CxxDynIdentity<'a>>;

pub struct CxxDynIdentity<'a>(pub &'a mut DynIdentity);
impl CxxDynIdentity<'_> {
    pub fn set_name(&mut self, name: String) {
        self.0.set_name(name);
    }

    pub fn get_name(&self) -> String {
        self.0.get_name()
    }
}
