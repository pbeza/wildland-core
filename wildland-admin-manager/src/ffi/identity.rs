use crate::{
    api::AdminManagerIdentity,
    ffi::{option::Opt, result::Res},
};

pub type IdentityResult = Res<DynIdentity>;
pub type OptionalIdentity = Opt<DynIdentity>;

// TODO derive macro
#[derive(Debug)]
pub struct DynIdentity(pub AdminManagerIdentity);
impl DynIdentity {
    pub fn set_name(&self, name: String) {
        self.0.lock().unwrap().set_name(name)
    }

    pub fn get_name(&self) -> String {
        self.0.lock().unwrap().get_name()
    }
}
