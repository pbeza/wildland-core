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

use crate::error::CryptoError;
use crate::identity::{MnemonicPhrase, MNEMONIC_LEN};
use bip39::Language::English;
use bip39::{Mnemonic, MnemonicType};
use hkdf::Hkdf;
use sha2::Sha256;

/// Generate a new random mnemonic phrase
#[tracing::instrument(level="debug", skip_all)]
pub fn generate_random_mnemonic() -> Result<MnemonicPhrase, CryptoError> {
    Mnemonic::new(
        MnemonicType::for_word_count(MNEMONIC_LEN)
            .map_err(|e| CryptoError::MnemonicGenerationError(e.to_string()))?,
        English,
    )
    .phrase()
    .split(' ')
    .map(|word| word.to_owned())
    .collect::<Vec<String>>()
    .try_into()
    .map_err(|e: Vec<_>| {
        CryptoError::MnemonicGenerationError(format!(
            "Invalid mnemonic phrase length: {} - expected {}",
            e.len(),
            MNEMONIC_LEN
        ))
    })
}

pub(crate) fn extend_seed(seed: &[u8], target: &mut [u8; 96]) {
    let info = [87, 105, 108, 100, 108, 97, 110, 100]; // list(b'Wildland')
    let hk = Hkdf::<Sha256>::new(None, seed);
    hk.expand(&info, target)
        .expect("Should return 96 bytes of randomness");
}

#[cfg(test)]
mod tests {
    use bip39::Language::English;
    use bip39::{Mnemonic, Seed};
    use hex_literal::hex;

    use crate::common::test_utilities::MNEMONIC_PHRASE;

    use super::*;

    #[test]
    fn expanding_the_seed_from_vector() {
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
        extend_seed(&ikm, &mut output_key_material);
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

    #[test]
    fn seed_should_be_deterministic() {
        let mnemonic = Mnemonic::from_phrase(MNEMONIC_PHRASE, English).unwrap();
        let seed = Seed::new(&mnemonic, "");
        let mut output_key_material = [0u8; 96];
        extend_seed(seed.as_bytes(), &mut output_key_material);
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
