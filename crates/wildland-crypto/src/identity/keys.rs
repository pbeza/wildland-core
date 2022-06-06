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
    self, CannotCreateKeypairError, CannotDecryptMessageError, CannotEncryptMessageError,
};
use hex::{FromHex};
// use crypto_box::aead::Aead;
use ed25519_dalek::Keypair as SigningKeypairA;
use ed25519_dalek::PublicKey as VerifyingKeyA;
use ed25519_dalek::SecretKey as SigningKeyA;

use crypto_box::PublicKey as PublicKeyA;
use crypto_box::SecretKey as SecretKeyA;

pub type SigningKeypair = SigningKeypairA;
pub type SigningKey = SigningKeyA;
pub type VerifyingKey = VerifyingKeyA;

pub type PublicKey = PublicKeyA;
pub type SecretKey = SecretKeyA;
pub struct EncryptingKeypair {
    pub secret: SecretKey,
    pub public: PublicKey,
}

// impl EncryptingKeypair {
//     fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
//         let pubkey = <[u8; 32]>::from_hex(public_key)
//             .map_err(|_| CannotCreateKeypairError(public_key.into()))?;
//         let seckey: [u8; 32] = <[u8; 32]>::from_hex(secret_key)
//             .map_err(|_| CannotCreateKeypairError(secret_key.into()))?;
//         Ok( Self::from_bytes(pubkey, seckey))
//     }
// }

// impl SigningKeypair {
//     fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
//         let pubkey = <[u8; 32]>::from_hex(public_key)
//             .map_err(|_| CannotCreateKeypairError(public_key.into()))?;
//         let seckey: [u8; 32] = <[u8; 32]>::from_hex(secret_key)
//             .map_err(|_| CannotCreateKeypairError(secret_key.into()))?;
//         Ok( Self::from_bytes(pubkey, seckey))
//     }
// }

pub trait Keypair {
    fn from_bytes(seckey: [u8; 32], pubkey: [u8; 32]) -> Self;
    fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> where Self: Sized;
}

impl Keypair for EncryptingKeypair {
    fn from_bytes(seckey: [u8; 32], pubkey: [u8; 32]) -> Self {
        Self { secret: SecretKey::from(seckey), public: PublicKey::from(pubkey) }
    }
    fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
        let pubkey = <[u8; 32]>::from_hex(public_key)
            .map_err(|_| CannotCreateKeypairError(public_key.into()))?;
        let seckey: [u8; 32] = <[u8; 32]>::from_hex(secret_key)
            .map_err(|_| CannotCreateKeypairError(secret_key.into()))?;
        Ok( Self::from_bytes(pubkey, seckey))
    }
}

impl Keypair for SigningKeypair {
    fn from_bytes(seckey: [u8; 32], pubkey: [u8; 32]) -> Self {
        let secret: SigningKey = SigningKey::from_bytes(&seckey).unwrap();
        let public: VerifyingKey = VerifyingKey::from_bytes(&pubkey).unwrap();

        SigningKeypair{
            secret: secret,
            public: public
        }
    }

    fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
        let pubkey = <[u8; 32]>::from_hex(public_key)
            .map_err(|_| CannotCreateKeypairError(public_key.into()))?;
        let seckey: [u8; 32] = <[u8; 32]>::from_hex(secret_key)
            .map_err(|_| CannotCreateKeypairError(secret_key.into()))?;

        let secret_key: SigningKey = SigningKey::from_bytes(&seckey).unwrap();
        let public_key: VerifyingKey = VerifyingKey::from_bytes(&pubkey).unwrap();

        Ok(SigningKeypair{
            secret: secret_key,
            public: public_key
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::common::test_utilities::{
        generate_random_nonce, ENCRYPTION_PUBLIC_KEY_1, ENCRYPTION_PUBLIC_KEY_2,
        ENCRYPTION_SECRET_KEY_1, ENCRYPTION_SECRET_KEY_2, SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY,
        TIMESTAMP,
    };
    use crate::identity::keys::{EncryptingKeypair, SigningKeypair, Keypair};
    use crate::identity::keys::{PublicKey, SecretKey};
    use ed25519_dalek::{Signer};
    use salsa20::XNonce;
    use crypto_box::{aead::Aead};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(rename(serialize = "credentialID"))]
        pub credential_id: String,
        pub timestamp: String,
    }

    fn encrypt(msg: &[u8], sender: &SecretKey, recipient: &PublicKey, nonce: &XNonce) -> Vec<u8> {
        let sbox = crypto_box::Box::new(recipient, sender);
        let ciphertext = sbox.encrypt(&nonce, msg).unwrap();
        ciphertext
    }

    fn decrypt(ciphertext: &[u8], sender: &PublicKey, recipient: &SecretKey, nonce: &XNonce) -> Vec<u8> {
        let sbox = crypto_box::Box::new(sender, recipient);
        let msg = sbox.decrypt(&nonce, ciphertext).unwrap();
        msg
    }

    #[test]
    fn should_sign_custom_struct() {
        // given
        let keypair = SigningKeypair::from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap();
        let message_to_sign = generate_message();
        let expected_message = get_expected_message();

        // when
        let signature = &keypair.sign(&message_to_sign);

        // then
        keypair.verify(&expected_message, signature).expect("OK");
    }

    #[test]
    fn can_encrypt_custom_struct() {
        let alice_keypair =
            EncryptingKeypair::from_str(ENCRYPTION_PUBLIC_KEY_2, ENCRYPTION_SECRET_KEY_2).unwrap();
        let bob_keypair =
            EncryptingKeypair::from_str(ENCRYPTION_PUBLIC_KEY_2, ENCRYPTION_SECRET_KEY_2).unwrap();
        let nonce = generate_random_nonce();
        let message_to_encrypt = generate_message();
        let expected_message = get_expected_message();

        let ciphertext = encrypt(message_to_encrypt.as_ref(),
                                 &alice_keypair.secret,
                                 &bob_keypair.public,
                                 &nonce);
        let result = decrypt(ciphertext.as_slice(),
                             &alice_keypair.public,
                             &bob_keypair.secret,
                             &nonce);

        assert_eq!(expected_message, result.as_slice())
    }

    #[test]
    fn should_create_keypair_when_keys_have_proper_length() {
        // when
        let keypair = SigningKeypair::from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY);

        // then
        assert!(keypair.is_ok());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_short() {
        // when
        let keypair = SigningKeypair::from_str("", SIGNING_SECRET_KEY);

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_long() {
        // when
        let keypair = SigningKeypair::from_str(
            "1234567890123456789012345678901234567890123456789012345678901234567890",
            SIGNING_SECRET_KEY,
        );

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_short() {
        // when
        let keypair = SigningKeypair::from_str(SIGNING_PUBLIC_KEY, "");

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_long() {
        // when
        let keypair = SigningKeypair::from_str(
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
