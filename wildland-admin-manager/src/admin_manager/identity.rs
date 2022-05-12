use crate::api;

pub struct Identity {
    identity_type: api::IdentityType,
    name: String,
    pubkey: Vec<u8>,
    fingerprint: Vec<u8>,
}

impl Identity {
    pub fn new_master_identity(name: String, pubkey: Vec<u8>, fingerprint: Vec<u8>) -> Self {
        Self {
            identity_type: api::IdentityType::Master,
            name,
            pubkey,
            fingerprint,
        }
    }

    pub fn new_device_identity(name: String, pubkey: Vec<u8>, fingerprint: Vec<u8>) -> Self {
        Self {
            identity_type: api::IdentityType::Device,
            name,
            pubkey,
            fingerprint,
        }
    }
}

impl api::Identity for Identity {
    fn set_name(&mut self, name: String) {
        self.name = name
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_pubkey(&mut self, pubkey: Vec<u8>) {
        self.pubkey = pubkey
    }

    fn get_pubkey(&self) -> Vec<u8> {
        self.pubkey.clone()
    }

    fn set_fingerprint(&mut self, fingerprint: Vec<u8>) {
        self.fingerprint = fingerprint;
    }

    fn get_fingerprint(&self) -> Vec<u8> {
        self.fingerprint.clone()
    }

    fn set_identity(&mut self, identity: api::IdentityType) {
        self.identity_type = identity
    }

    fn get_identity(&self) -> api::IdentityType {
        self.identity_type
    }
}
