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

use crate::error::CryptoError;
use bip39::Mnemonic;
use hkdf::Hkdf;
use sha2::Sha256;

pub const SEED_PHRASE_LEN: usize = 12;
pub type SeedPhraseWords = [String; SEED_PHRASE_LEN];

/// Create a new random seed phrase
pub fn generate_random_seed_phrase() -> Result<SeedPhraseWords, CryptoError> {
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

pub fn extend_seed(seed: [u8; 64], target: &mut [u8; 96]) {
    let initial_key_material = seed;
    let info = [87, 105, 108, 100, 108, 97, 110, 100]; // list(b'Wildland')
    let hk = Hkdf::<Sha256>::new(None, &initial_key_material);
    hk.expand(&info, target)
        .expect("Should return 96 bytes of randomness");
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn expanding_the_seed() {
        // vectors constructed using 'hkdf' python package
        let ikm = hex!(
            "
            0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b
            0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b
            0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b
            0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b
        "
        );
        let mut output_key_material = [0u8; 96];
        extend_seed(ikm, &mut output_key_material);
        let (secret_key, chain_code) = output_key_material.split_at(64);
        let expected_secret = hex!(
            "
            540d175899e60c3fae2e80592a19ef98
            3b26186b5b4be4bbb9cf590ab401d689
            7e293e76ac281196ec04b7bc68d2e8a0
            36ef6b6171f6fcde3836fdaacbd1a661
        "
        );
        assert_eq!(secret_key, expected_secret);
        let expected_chain_code = hex!(
            "
            d4d1716dc1a50023fc97267109d4e4e7
            b1ff0ba00e5404d7127b48bfd4900e79
        "
        );
        assert_eq!(chain_code, expected_chain_code);
    }
}
