use crate::{
    api::Identity,
    ffi::{option::Opt, result::Res},
};

pub type DynIdentity = Box<dyn Identity>;
pub type IdentityResult<'a> = Res<CxxDynIdentity<'a>>;
pub type OptionalIdentity<'a> = Opt<CxxDynIdentity<'a>>;

#[derive(Debug)]
pub struct CxxDynIdentity<'a>(pub &'a mut DynIdentity);
impl CxxDynIdentity<'_> {
    pub fn set_name(&mut self, name: String) {
        self.0.set_name(name);
    }

    pub fn get_name(&self) -> String {
        self.0.get_name()
    }
}
