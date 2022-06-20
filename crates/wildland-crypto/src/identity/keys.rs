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

use crypto_box::{PublicKey as EncryptionPublicKey, SecretKey as EncryptionSecretKey};
use hex::FromHex;

use crate::identity::error::CryptoError::{self, CannotCreateKeyError};

pub type SigningKeypair = ed25519_dalek::Keypair;

pub struct EncryptingKeypair {
    pub secret: EncryptionSecretKey,
    pub public: EncryptionPublicKey,
}

pub trait Keypair {
    fn from_bytes_slices(seckey: [u8; 32], pubkey: [u8; 32]) -> Self;
    fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError>
    where
        Self: Sized;
}

impl Keypair for EncryptingKeypair {
    fn from_bytes_slices(seckey: [u8; 32], pubkey: [u8; 32]) -> Self {
        Self {
            secret: EncryptionSecretKey::from(seckey),
            public: EncryptionPublicKey::from(pubkey),
        }
    }
    fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
        let pubkey = bytes_key_from_str(public_key)?;
        let seckey = bytes_key_from_str(secret_key)?;
        Ok(Self::from_bytes_slices(pubkey, seckey))
    }
}

impl Keypair for SigningKeypair {
    fn from_bytes_slices(seckey: [u8; 32], pubkey: [u8; 32]) -> Self {
        SigningKeypair::from_bytes([seckey, pubkey].concat().as_slice()).unwrap()
    }

    fn from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
        let pubkey = bytes_key_from_str(public_key)?;
        let seckey = bytes_key_from_str(secret_key)?;
        Ok(Self::from_bytes_slices(seckey, pubkey))
    }
}

pub fn bytes_key_from_str(key: &str) -> Result<[u8; 32], CryptoError> {
    let key = <[u8; 32]>::from_hex(key).map_err(|_| CannotCreateKeyError(key.len()))?;
    Ok(key)
}

#[cfg(test)]
mod tests {

    use ed25519_dalek::Signer;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::common::test_utilities::{
        generate_message, generate_random_nonce, get_expected_message, ENCRYPTION_PUBLIC_KEY_2,
        ENCRYPTION_SECRET_KEY_2, SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY, TIMESTAMP,
    };
    use crate::identity::keys::{EncryptingKeypair, Keypair, SigningKeypair};

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
}
