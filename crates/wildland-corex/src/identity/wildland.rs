use std::fmt;
use std::fmt::{Display, Formatter};
use wildland_crypto::identity::SigningKeypair;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WildlandIdentityType {
    Forest,
    Device,
}

impl Display for WildlandIdentityType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct WildlandIdentity {
    identity_type: WildlandIdentityType,
    keypair: SigningKeypair,
    name: String,
}

impl WildlandIdentity {
    pub(crate) fn new(
        identity_type: WildlandIdentityType,
        keypair: SigningKeypair,
        name: String,
    ) -> Self {
        Self {
            identity_type,
            keypair,
            name,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.keypair.public().into()
    }

    pub fn get_private_key(&self) -> Vec<u8> {
        self.keypair.secret().into()
    }

    pub fn get_keypair_bytes(&self) -> Vec<u8> {
        self.keypair.to_bytes()
    }

    pub fn get_fingerprint(&self) -> String {
        format!("wildland.{}.{}", self.identity_type, self.name)
    }

    pub fn get_type(&self) -> WildlandIdentityType {
        self.identity_type
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utilities::{SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY};
    use crate::WildlandIdentity;
    use crate::WildlandIdentityType::Device;
    use wildland_crypto::identity::SigningKeypair;

    #[test]
    fn should_get_correct_fingerprint() {
        let keypair = SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap();
        let wildland_identity = WildlandIdentity::new(Device, keypair, "Device 1".to_string());

        assert_eq!(
            wildland_identity.get_fingerprint(),
            "wildland.Device.Device 1".to_string()
        )
    }
}
