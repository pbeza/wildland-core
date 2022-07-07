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

use crate::{error::CryptoError, signature::Signature};
use ed25519_dalek::Signer;

use super::bytes_key_from_str;

#[derive(Debug)]
pub struct SigningKeypair(ed25519_dalek::Keypair);

impl SigningKeypair {
    pub fn try_from_bytes_slices(pubkey: [u8; 32], seckey: [u8; 32]) -> Result<Self, CryptoError> {
        Ok(Self(
            ed25519_dalek::Keypair::from_bytes([seckey, pubkey].concat().as_slice())
                .map_err(|e| CryptoError::InvalidSignatureBytesError(e.to_string()))?,
        ))
    }

    pub fn try_from_str(public_key: &str, secret_key: &str) -> Result<Self, CryptoError> {
        let pubkey = bytes_key_from_str(public_key)?;
        let seckey = bytes_key_from_str(secret_key)?;
        Self::try_from_bytes_slices(pubkey, seckey)
    }

    pub fn try_from_secret_bytes(secret_key_bytes: &[u8; 32]) -> Result<Self, CryptoError> {
        let sec_key = ed25519_dalek::SecretKey::from_bytes(secret_key_bytes)
            .map_err(|e| CryptoError::InvalidSignatureBytesError(e.to_string()))?;
        let pub_key = ed25519_dalek::PublicKey::from(&sec_key);
        Ok(Self(ed25519_dalek::Keypair {
            secret: sec_key,
            public: pub_key,
        }))
    }

    pub fn public(&self) -> [u8; 32] {
        self.0.public.to_bytes()
    }

    pub fn secret(&self) -> [u8; 32] {
        self.0.secret.to_bytes()
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        Signature(self.0.sign(msg))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::test_utilities::{SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY};
    use crate::identity::signing_keypair::SigningKeypair;

    #[test]
    fn should_create_keypair_when_keys_have_proper_length() {
        // when
        let keypair = SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY);

        // then
        assert!(keypair.is_ok());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_short() {
        // when
        let keypair = SigningKeypair::try_from_str("", SIGNING_SECRET_KEY);

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_pub_key_is_too_long() {
        // when
        let keypair = SigningKeypair::try_from_str(
            "1234567890123456789012345678901234567890123456789012345678901234567890",
            SIGNING_SECRET_KEY,
        );

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_short() {
        // when
        let keypair = SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, "");

        // then
        assert!(keypair.is_err());
    }

    #[test]
    fn should_not_create_keypair_when_sec_key_is_too_long() {
        // when
        let keypair = SigningKeypair::try_from_str(
            SIGNING_PUBLIC_KEY,
            "1234567890123456789012345678901234567890123456789012345678901234567890",
        );

        // then
        assert!(keypair.is_err());
    }
}