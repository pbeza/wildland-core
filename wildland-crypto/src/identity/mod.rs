//
// Wildland Project
//
// Copyright © 2021 Golem Foundation,
// 	    	     Pawel Peregud <pepesza@wildland.io>
// 	    	     Piotr K. Isajew <piotr@wildland.io>
// 	    	     Lukasz Kujawski <leon@wildland.io>
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

use bip39::{Language, Mnemonic};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;
use wildland_admin_manager_api::CryptoError;

use crate::error::{CargoError, CargoErrorRepresentable};
pub use crate::identity::{derivation::Identity, keys::KeyPair};

pub mod derivation;
pub mod error;
pub mod keys;
mod seed;

pub const SEED_PHRASE_LEN: usize = 12;
type SeedPhrase = [String; SEED_PHRASE_LEN];

// TODO move these errors to identity/error.rs - WAP-86
#[derive(Copy, Clone, PartialEq, Debug, Error)]
pub enum IdentityError {
    InvalidWordVector = 1,
    EntropyTooLow = 2,
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl CargoErrorRepresentable for IdentityError {
    const CARGO_ERROR_TYPE: &'static str = "IdentityError";

    fn error_code(&self) -> String {
        self.to_string()
    }
}

/// Deterministically derive Wildland identity from Ethereum
/// signature (or any random bits). Assumes high quality entropy
/// and does not perform any checks.
#[allow(clippy::ptr_arg)]
pub fn from_entropy(entropy: &Vec<u8>) -> Result<Identity, CargoError> {
    // assume high quality entropy of arbitrary length (>= 32 bytes)
    if (entropy.len() * 8) < 128 {
        return Err(IdentityError::EntropyTooLow.into());
    }
    let mut hasher = Sha256::new();
    hasher.update(entropy);
    let hashed_entropy = hasher.finalize();
    let mnemonic = Mnemonic::from_entropy(&hashed_entropy[0..16]).unwrap();
    let words = mnemonic
        .word_iter()
        .map(|word| word.to_owned())
        .collect::<Vec<_>>();
    from_mnemonic(&words)
}

/// Create a new, random Wildland identity.
/// Will return new identity each time it is called.
pub fn from_random_seed() -> Result<Identity, CargoError> {
    let mnemonic = Mnemonic::generate(SEED_PHRASE_LEN).unwrap();
    let words = mnemonic
        .word_iter()
        .map(|word| word.to_owned())
        .collect::<Vec<_>>();
    from_mnemonic(&words)
}

/// Create a new random seed phrase
pub fn generate_random_seed_phrase() -> Result<SeedPhrase, CryptoError> {
    let mnemonic = Mnemonic::generate(SEED_PHRASE_LEN)
        .map_err(|e| CryptoError::SeedPhraseGenerationError(e.to_string()))?;
    mnemonic
        .word_iter()
        .map(|word| word.to_owned())
        .collect::<Vec<String>>()
        .try_into()
        .map_err(|e: Vec<_>| {
            CryptoError::SeedPhraseGenerationError(format!(
                "Invalid seed phrase length: {} - expected {}",
                e.len(),
                SEED_PHRASE_LEN
            ))
        })
}

/// Derive Wildland identity from mnemonic (12 dictionary words).
#[allow(clippy::ptr_arg)]
pub fn from_mnemonic(phrase: &[String]) -> Result<Identity, CargoError> {
    if phrase.len() != SEED_PHRASE_LEN {
        return Err(IdentityError::InvalidWordVector.into());
    }
    let mnemonic_string: String = phrase.join(" ");
    Mnemonic::parse_in_normalized(Language::English, &mnemonic_string)
        .map_err(|_error| IdentityError::InvalidWordVector.into())
        .map(Identity::from_mnemonic)
}

#[cfg(test)]
mod tests {
    use ed25519_bip32::XPrv;
    use hex_literal::hex;

    use super::*;

    const TEST_MNEMONIC_6: &str = "abandon abandon abandon abandon abandon about";
    const TEST_MNEMONIC_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const TEST_MNEMONIC_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
    abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const TEST_MNEMONIC_ITALIAN: &str =
        "abaco abaco abaco abaco abaco abaco abaco abaco abaco abaco abaco abbaglio";

    // expected extended root private key bytes generated from TEST_MNEMONIC_12
    const ROOT_XPRV: [u8; 96] = [
        24, 97, 125, 255, 78, 254, 242, 4, 80, 221, 94, 175, 192, 96, 253, 133, 250, 172, 202, 19,
        217, 90, 206, 59, 218, 11, 227, 46, 70, 148, 252, 215, 161, 178, 196, 120, 102, 114, 194,
        12, 205, 218, 138, 151, 244, 166, 214, 35, 131, 140, 194, 70, 236, 205, 123, 72, 70, 215,
        44, 36, 182, 15, 25, 158, 117, 161, 211, 29, 125, 195, 12, 236, 138, 155, 206, 3, 16, 11,
        54, 143, 209, 223, 7, 250, 9, 252, 142, 87, 79, 214, 211, 69, 2, 147, 159, 63,
    ];

    #[test]
    fn can_generate_seed_for_phrase() {
        let user = from_random_seed().unwrap();
        assert_eq!(user.mnemonic().len(), SEED_PHRASE_LEN);
    }

    #[test]
    fn can_generate_from_entropy() {
        let entropy = hex!(
            "
            65426aa1176159d1929caea10514cddd
            d11235741001f125922f258a58716b58
            da63e3060fe461fe37e4ed201d76b132
            e35830929b0f4764e577d3da09ecb6d2
            12
        "
        );
        let user = from_entropy(&entropy.to_vec()).ok().unwrap();
        assert_eq!(
            vec!(
                "expect", "cruel", "stadium", "sand", "couch", "garden", "nothing", "wool",
                "grocery", "shop", "noise", "voice"
            ),
            user.mnemonic()
        );
    }

    #[test]
    fn will_crash_on_low_entropy_source() {
        let entropy = hex!(
            "
            65426aa1176159d1929caea10514
        "
        );
        assert!(from_entropy(&entropy.to_vec()).is_err());
    }

    #[test]
    fn can_generate_from_mnemonic() {
        let mnemonic_vec: Vec<String> = TEST_MNEMONIC_12
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let user = from_mnemonic(&mnemonic_vec).ok().unwrap();

        assert_eq!(user.get_xprv(), &XPrv::normalize_bytes_ed25519(ROOT_XPRV))
    }

    #[test]
    fn should_fail_on_too_long_mnemonic() {
        let mnemonic_vec: Vec<String> = TEST_MNEMONIC_24
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert!(from_mnemonic(&mnemonic_vec).is_err());
    }

    #[test]
    fn should_fail_on_too_short_mnemonic() {
        let mnemonic_vec: Vec<String> = TEST_MNEMONIC_6
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert!(from_mnemonic(&mnemonic_vec).is_err());
    }

    #[test]
    fn should_fail_on_not_english_mnemonic() {
        let mnemonic_vec: Vec<String> = TEST_MNEMONIC_ITALIAN
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        assert!(from_mnemonic(&mnemonic_vec).is_err());
    }

    #[test]
    fn can_recover_seed() {
        let mnemonic_vec: Vec<String> = TEST_MNEMONIC_12
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let user = from_mnemonic(&mnemonic_vec).unwrap();
        assert_eq!(user.mnemonic().join(" "), TEST_MNEMONIC_12);
    }
}
