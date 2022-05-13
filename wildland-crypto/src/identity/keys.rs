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

use crate::identity::error::CryptoError::{
    self, CannotCreateKeyPairError, CannotDecryptMessageError, CannotEncryptMessageError,
};
use crypto_box::{aead::Aead, PublicKey, SecretKey};
use cryptoxide::ed25519::{signature, verify, SIGNATURE_LENGTH};
use hex::{encode, FromHex};
use salsa20::XNonce;

pub trait SigningKeyPair {
    fn pubkey_as_bytes(&self) -> [u8; 32];
    fn seckey_as_bytes(&self) -> [u8; 32];
    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_LENGTH];
    fn verify(&self, message: &[u8], signature: &[u8; SIGNATURE_LENGTH]) -> bool;
}

pub trait EncryptionKeyPair {
    fn pubkey(&self) -> PublicKey;
    fn seckey(&self) -> SecretKey;
    fn encrypt(
        &self,
        message: &[u8],
        nonce: &XNonce,
        recipient_pubkey: &PublicKey,
    ) -> Result<Vec<u8>, CryptoError>;

    fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &XNonce,
        sender_pubkey: &PublicKey,
    ) -> Result<Vec<u8>, CryptoError>;
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
    pub fn signing_keypair_from_str(
        public_key: &str,
        secret_key: &str,
    ) -> Result<impl SigningKeyPair, CryptoError> {
        KeyPair::from_str(public_key, secret_key)
    }

    pub(crate) fn from_bytes(seckey: [u8; 32], pubkey: [u8; 32]) -> Self {
        Self { seckey, pubkey }
    }

    pub(crate) fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
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
    fn pubkey_as_bytes(&self) -> [u8; 32] {
        self.pubkey
    }

    fn seckey_as_bytes(&self) -> [u8; 32] {
        self.seckey
    }

    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_LENGTH] {
        signature(message, &self.packed())
    }

    fn verify(&self, message: &[u8], signature: &[u8; SIGNATURE_LENGTH]) -> bool {
        verify(message, &self.pubkey, signature)
    }
}

impl EncryptionKeyPair for KeyPair {
    fn pubkey(&self) -> PublicKey {
        crypto_box::PublicKey::from(self.pubkey)
    }

    fn seckey(&self) -> SecretKey {
        crypto_box::SecretKey::from(self.seckey)
    }

    fn encrypt(
        &self,
        message: &[u8],
        nonce: &XNonce,
        recipient_pubkey: &PublicKey,
    ) -> Result<Vec<u8>, CryptoError> {
        let salsa_box = crypto_box::Box::new(recipient_pubkey, &self.seckey());

        salsa_box
            .encrypt(nonce, message)
            .map_err(|_| CannotEncryptMessageError(encode(message)))
    }

    fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &XNonce,
        sender_pubkey: &PublicKey,
    ) -> Result<Vec<u8>, CryptoError> {
        let salsa_box = crypto_box::Box::new(sender_pubkey, &self.seckey());

        salsa_box
            .decrypt(nonce, ciphertext)
            .map_err(|_| CannotDecryptMessageError(encode(ciphertext)))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::test_utilities::{
        generate_random_nonce, ENCRYPTION_PUBLIC_KEY_1, ENCRYPTION_PUBLIC_KEY_2,
        ENCRYPTION_SECRET_KEY_1, ENCRYPTION_SECRET_KEY_2, SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY,
        TIMESTAMP,
    };
    use crate::identity::keys::{EncryptionKeyPair, SigningKeyPair};
    use crate::identity::KeyPair;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(rename(serialize = "credentialID"))]
        pub credential_id: String,
        pub timestamp: String,
    }

    #[test]
    fn should_sign_custom_struct() {
        // given
        let keypair = KeyPair::from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap();
        let message_to_sign = generate_message();
        let expected_message = get_expected_message();

        // when
        let signature = &keypair.sign(&message_to_sign);

        // then
        assert!(&keypair.verify(&expected_message, signature));
    }

    #[test]
    fn can_encrypt_custom_struct() {
        let alice_keypair =
            KeyPair::from_str(ENCRYPTION_PUBLIC_KEY_1, ENCRYPTION_SECRET_KEY_1).unwrap();
        let bob_keypair =
            KeyPair::from_str(ENCRYPTION_PUBLIC_KEY_2, ENCRYPTION_SECRET_KEY_2).unwrap();
        let nonce = generate_random_nonce();
        let message_to_encrypt = generate_message();
        let expected_message = get_expected_message();

        let ciphertext = alice_keypair
            .encrypt(message_to_encrypt.as_ref(), &nonce, &bob_keypair.pubkey())
            .unwrap();
        let result = bob_keypair.decrypt(ciphertext.as_slice(), &nonce, &alice_keypair.pubkey());

        assert_eq!(expected_message, result.unwrap().as_slice())
    }

    #[test]
    fn should_create_keypair_when_keys_have_proper_length() {
        // when
        let keypair = KeyPair::from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY);

        // then
        assert!(keypair.is_ok());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_short() {
        // when
        let keypair = KeyPair::from_str("", SIGNING_SECRET_KEY);

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_long() {
        // when
        let keypair = KeyPair::from_str(
            "1234567890123456789012345678901234567890123456789012345678901234567890",
            SIGNING_SECRET_KEY,
        );

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_short() {
        // when
        let keypair = KeyPair::from_str(SIGNING_PUBLIC_KEY, "");

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_long() {
        // when
        let keypair = KeyPair::from_str(
            SIGNING_PUBLIC_KEY,
            "1234567890123456789012345678901234567890123456789012345678901234567890",
        );

        // then
        assert!(keypair.is_err());
    }

    fn generate_message() -> Vec<u8> {
        let request = TestStruct {
            credential_id: SIGNING_PUBLIC_KEY.into(),
            timestamp: TIMESTAMP.into(),
        };
        serde_json::to_vec(&request).unwrap()
    }

    fn get_expected_message() -> Vec<u8> {
        let expected_json_str = r#"
        {
            "credentialID":"1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f",
            "timestamp":"1648541699814"
        }
        "#;
        let expected_json: Value = serde_json::from_str(expected_json_str).unwrap();
        serde_json::to_vec(&expected_json).unwrap()
    }
}
