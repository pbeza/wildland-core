//
// Wildland Project
//
// Copyright © 2021 Golem Foundation,
// 	    	     Lukasz Kujawski <leon@wildland.io>
// 	    	     Pawel Peregud <pepesza@wildland.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use cryptoxide::ed25519;
use cryptoxide::ed25519::SIGNATURE_LENGTH;
use crate::identity::error::CryptoError;
use crate::identity::error::CryptoError::CannotCreateKeyPairError;
use hex::FromHex;

pub trait SigningKeyPair {
    fn pubkey(&self) -> [u8; 32];
    fn seckey(&self) -> [u8; 32];
    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_LENGTH];
    fn verify(&self, message: &[u8], signature: &[u8; SIGNATURE_LENGTH]) -> bool;
    }

pub trait EncryptionKeyPair {
    fn pubkey(&self) -> [u8; 32];
    fn seckey(&self) -> [u8; 32];
}

/// KeyPair type.
///
/// Represents a keypair derived from seed. Can be used to sign or to encrypt,
/// depending on the way it was derived.
pub struct KeyPair {
    seckey: [u8; 32],
    pubkey: [u8; 32],
}

impl KeyPair {
    pub fn from_bytes(seckey: [u8; 32], pubkey: [u8; 32]) -> Self {
        Self { seckey, pubkey }
    }

    pub fn from_str<'a>(public_key: &'a str, secret_key: &'a str) -> Result<Self, CryptoError> {
        let pubkey = <[u8; 32]>::from_hex(public_key)
            .map_err(|_| CannotCreateKeyPairError(public_key.into()))?;
        let seckey: [u8; 32] = <[u8; 32]>::from_hex(secret_key)
            .map_err(|_| CannotCreateKeyPairError(secret_key.into()))?;

        Ok(Self { pubkey, seckey })
    }

    fn packed(&self) -> [u8; 64] {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[..32].copy_from_slice(&self.seckey[..32]);
        bytes[32..64].copy_from_slice(&self.pubkey[..32]);
        bytes
    }

}

impl SigningKeyPair for KeyPair {
    fn pubkey(&self) -> [u8; 32] {
        self.pubkey
    }

    fn seckey(&self) -> [u8; 32] {
        self.seckey
    }

    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_LENGTH] {
        ed25519::signature(message, &self.packed())
    }

    fn verify(
        &self, message: &[u8],
        signature: &[u8; SIGNATURE_LENGTH],
    ) -> bool {
        ed25519::verify(message, &self.pubkey, signature)
    }
}

impl EncryptionKeyPair for KeyPair {
    fn pubkey(&self) -> [u8; 32] {
        self.pubkey
    }

    fn seckey(&self) -> [u8; 32] {
        self.seckey
    }
}

#[cfg(test)]
mod tests {
    use serde::{Serialize, Deserialize};
    use serde_json::Value;
    use crate::constants::test_utilities::{PUBLIC_KEY, SECRET_KEY, TIMESTAMP};
    use crate::identity::KeyPair;
    use crate::identity::keys::SigningKeyPair;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(rename(serialize = "credentialID"))]
        pub credential_id: String,
        pub timestamp: String,
    }

    #[test]
    fn should_sign_custom_struct() {
        // given
        let keypair = KeyPair::from_str(PUBLIC_KEY, SECRET_KEY).unwrap();
        let request = TestStruct {
            credential_id: PUBLIC_KEY.into(),
            timestamp: TIMESTAMP.into(),
        };
        let message = serde_json::to_vec(&request).unwrap();

        // when
        let signature = &keypair.sign(&message);
        let expected_json_str = r#"
        {
            "credentialID":"1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f",
            "timestamp":"1648541699814"
        }
        "#;
        let expected_json: Value = serde_json::from_str(expected_json_str).unwrap();
        let expected_message = serde_json::to_vec(&expected_json).unwrap();

        // then
        assert!(&keypair.verify(&expected_message, signature));
    }

    #[test]
    fn should_create_keypair_when_keys_have_proper_length() {
        // when
        let keypair = KeyPair::from_str(PUBLIC_KEY, SECRET_KEY);

        // then
        assert!(keypair.is_ok());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_short() {
        // when
        let keypair = KeyPair::from_str("", SECRET_KEY);

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_long() {
        // when
        let keypair = KeyPair::from_str(
            "1234567890123456789012345678901234567890123456789012345678901234567890",
            SECRET_KEY,
        );

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_short() {
        // when
        let keypair = KeyPair::from_str(PUBLIC_KEY, "");

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_long() {
        // when
        let keypair = KeyPair::from_str(
            PUBLIC_KEY,
            "1234567890123456789012345678901234567890123456789012345678901234567890",
        );

        // then
        assert!(keypair.is_err());
    }
}
