#[derive(Clone, Copy)]
pub enum IdentityType {
    Master,
    Device,
}

pub trait Identity {
    fn set_identity(&mut self, identity: IdentityType);
    fn get_identity(&self) -> IdentityType;
    fn set_name(&mut self, name: String);
    fn get_name(&self) -> String;
    fn set_pubkey(&mut self, pubkey: Vec<u8>);
    fn get_pubkey(&self) -> Vec<u8>;
    fn set_fingerprint(&mut self, fingerprint: Vec<u8>);
    fn get_fingerprint(&self) -> Vec<u8>;
}
