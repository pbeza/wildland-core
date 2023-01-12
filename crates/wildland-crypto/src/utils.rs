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

//! General crypto utilities and wrappers

// Please mind that udnerlying feature related to choosing the language is not
// exposed, as we are always using the default one (English).

use bip39::Mnemonic;

pub use crate::error::CryptoError;

/// Derivate mnemonic from a given phrase.
/// the phrase must be a valid english phrase.
///
/// # Errors
/// If the phrase is not a valid english phrase, a
/// `CryptoError::MnemonicGeneratioError` is returned.
///
/// # Examples
/// ```
/// use wildland_crypto::utils;
/// let mnemonic = utils::new_mnemonic_from_phrase("abandon abandon abandon abandon abandon abandon
/// abandon abandon abandon abandon abandon about").unwrap();
/// ```
pub fn new_mnemonic_from_phrase(phrase: &str) -> Result<Mnemonic, CryptoError> {
    Mnemonic::from_phrase(phrase, bip39::Language::English)
        .map_err(|e| CryptoError::MnemonicGenerationError(e.to_string()))
}

/// Derivate mnemonic from provided arbitrary length bytes.
///
/// # Errors
/// In case of internal error, a `CryptoError::MnemonicGeneratioError`
/// is returned.
///
/// # Examples
/// ```
/// use wildland_crypto::utils;
/// let mnemonic = utils::new_mnemonic_from_entropy(&[0; 32]).unwrap();
/// ```
pub fn new_mnemonic_from_entropy(entropy: &[u8]) -> Result<Mnemonic, CryptoError> {
    Mnemonic::from_entropy(entropy, bip39::Language::English)
        .map_err(|e| CryptoError::MnemonicGenerationError(e.to_string()))
}
