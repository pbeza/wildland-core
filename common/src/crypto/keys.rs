use crate::error::CorexCommonError;
use crate::error::CorexCommonError::CannotCreateKeyPairError;
use hex::FromHex;

pub struct KeyPair<T> {
    public_key: T,
    secret_key: T,
}

impl<'a> KeyPair<&'a str> {
    pub fn new(public_key: &'a str, secret_key: &'a str) -> Result<Self, CorexCommonError> {
        if public_key.len() != 64 || secret_key.len() != 64 {
            return Err(CannotCreateKeyPairError(public_key.into(), secret_key.into()));
        }
        Ok(Self {
            public_key,
            secret_key,
        })
    }

    pub fn packed(&self) -> [u8; 64] {
        let mut keypair: [u8; 64] = [0; 64];
        let seckey_bytes =
            <[u8; 32]>::from_hex(&self.secret_key).expect("Decoding secret key failed");
        let pubkey_bytes =
            <[u8; 32]>::from_hex(&self.public_key).expect("Decoding public key failed");
        keypair[..32].copy_from_slice(&seckey_bytes);
        keypair[32..64].copy_from_slice(&pubkey_bytes);

        keypair
    }

    pub fn pubkey_array(&self) -> [u8; 32] {
        <[u8; 32]>::from_hex(&self.public_key).expect("Decoding public key failed")
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::keys::KeyPair;
    use crate::utils::constants::{PUBLIC_KEY, SECRET_KEY};

    #[test]
    fn should_create_keypair_when_keys_have_proper_length() {
        // when
        let keypair = KeyPair::new(PUBLIC_KEY, SECRET_KEY);

        // then
        assert!(keypair.is_ok());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_short() {
        // when
        let keypair = KeyPair::new("", SECRET_KEY);

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_long() {
        // when
        let keypair = KeyPair::new(
            "1234567890123456789012345678901234567890123456789012345678901234567890",
            SECRET_KEY,
        );

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_short() {
        // when
        let keypair = KeyPair::new(PUBLIC_KEY, "");

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_long() {
        // when
        let keypair = KeyPair::new(
            PUBLIC_KEY,
            "1234567890123456789012345678901234567890123456789012345678901234567890",
        );

        // then
        assert!(keypair.is_err());
    }
}
