//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use super::bytes_key_from_str;
use crate::error::CryptoError;
use crate::signature::Signature;

pub type PubKey = [u8; 32];
pub type SecKey = [u8; 32];

#[derive(Debug)]
pub struct SigningKeypair(ed25519_dalek::Keypair);

impl<'de> Deserialize<'de> for SigningKeypair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex_encoded_str = String::deserialize(deserializer)?;
        let bytes = hex::decode(hex_encoded_str).map_err(serde::de::Error::custom)?;
        Ok(Self(
            ed25519_dalek::Keypair::from_bytes(&bytes).map_err(serde::de::Error::custom)?,
        ))
    }
}

impl Serialize for SigningKeypair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hex = hex::encode(self.0.to_bytes());
        String::serialize(&hex, serializer)
    }
}

impl TryFrom<Vec<u8>> for SigningKeypair {
    type Error = CryptoError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(
            ed25519_dalek::Keypair::from_bytes(value.as_slice())
                .map_err(|e| CryptoError::InvalidSignatureBytesError(e.to_string()))?,
        ))
    }
}

impl PartialEq for SigningKeypair {
    fn eq(&self, other: &Self) -> bool {
        self.public() == other.public() && self.secret() == other.secret()
    }
}

impl From<&SigningKeypair> for SigningKeypair {
    fn from(other: &SigningKeypair) -> Self {
        Self(ed25519_dalek::Keypair {
            public: PublicKey::from_bytes(&other.public()).unwrap(),
            secret: SecretKey::from_bytes(&other.secret()).unwrap(),
        })
    }
}

impl SigningKeypair {
    pub fn generate<R>(csprng: &mut R) -> Self
    where
        R: CryptoRng + RngCore,
    {
        //TODO: WILX-366 use ed25519_dalek::Keypair::generate(csprng) when ed25519_dalek will support rand 0.8

        let mut bytes = [0u8; 32];
        csprng.fill_bytes(&mut bytes);

        let sk = SecretKey::from_bytes(bytes.as_ref()).unwrap();

        Self(Keypair {
            public: (&sk).into(),
            secret: sk,
        })
    }

    pub fn try_from_bytes_slices(pubkey: PubKey, seckey: SecKey) -> Result<Self, CryptoError> {
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

    pub fn try_from_secret_bytes(secret_key_bytes: &SecKey) -> Result<Self, CryptoError> {
        let sec_key = ed25519_dalek::SecretKey::from_bytes(secret_key_bytes)
            .map_err(|e| CryptoError::InvalidSignatureBytesError(e.to_string()))?;
        let pub_key = ed25519_dalek::PublicKey::from(&sec_key);
        Ok(Self(ed25519_dalek::Keypair {
            secret: sec_key,
            public: pub_key,
        }))
    }

    pub fn public(&self) -> PubKey {
        self.0.public.to_bytes()
    }

    pub fn secret(&self) -> SecKey {
        self.0.secret.to_bytes()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        Vec::from(self.0.to_bytes())
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
