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

use bip39::{Language::English, Mnemonic, MnemonicType, Seed};
use hkdf::Hkdf;
use sha2::Sha256;
use MnemonicType::Words12;

use crate::error::CryptoError;

pub const SEED_PHRASE_LEN: usize = 12;
pub type SeedPhraseWords = [String; SEED_PHRASE_LEN];

/// Create a new random seed phrase
pub fn generate_random_seed_phrase() -> Result<SeedPhraseWords, CryptoError> {
    Mnemonic::new(Words12, English)
        .phrase()
        .split(' ')
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

pub fn extend_seed(seed: Seed, target: &mut [u8; 96]) {
    let input_key_material = seed.as_bytes();
    let info = [87, 105, 108, 100, 108, 97, 110, 100]; // list(b'Wildland')
    let hk = Hkdf::<Sha256>::new(None, input_key_material);
    hk.expand(&info, target)
        .expect("Should return 96 bytes of randomness");
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::common::test_utilities::MNEMONIC_PHRASE;

    use super::*;

    #[test]
    fn expanding_the_seed() {
        let mnemonic = Mnemonic::from_phrase(MNEMONIC_PHRASE, English).unwrap();
        let seed = Seed::new(&mnemonic, "");
        let mut output_key_material = [0u8; 96];
        extend_seed(seed, &mut output_key_material);
        let (secret_key, chain_code) = output_key_material.split_at(64);

        let expected_secret = hex!(
            "
            18617dff4efef20450dd5eafc060fd85
            faacca13d95ace3bda0be32e4694fcd7
            a1b2c4786672c20ccdda8a97f4a6d623
            838cc246eccd7b4846d72c24b60f199e
            "
        );
        assert_eq!(secret_key, expected_secret);
        let expected_chain_code = hex!(
            "
            75a1d31d7dc30cec8a9bce03100b368f
            d1df07fa09fc8e574fd6d34502939f3f
            "
        );
        assert_eq!(chain_code, expected_chain_code);
    }
}
