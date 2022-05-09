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

use crate::identity::error::CryptoError;
use crate::identity::error::CryptoError::CannotCreateKeyPairError;
use hex::FromHex;

pub trait SigningKeyPair {
    fn pubkey(&self) -> [u8; 32];
    fn seckey(&self) -> [u8; 32];
    fn packed(&self) -> [u8; 64];
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
}

impl SigningKeyPair for KeyPair {
    fn pubkey(&self) -> [u8; 32] {
        self.pubkey
    }

    fn seckey(&self) -> [u8; 32] {
        self.seckey
    }

    fn packed(&self) -> [u8; 64] {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[..32].copy_from_slice(&self.seckey[..32]);
        bytes[32..64].copy_from_slice(&self.pubkey[..32]);
        bytes
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
    use crate::identity::KeyPair;

    pub const PUBLIC_KEY: &str = "1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f";
    pub const SECRET_KEY: &str = "e02cdfa23ad7d94508108ad41410e556c5b0737e9c264d4a2304a7a45894fc57";

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
