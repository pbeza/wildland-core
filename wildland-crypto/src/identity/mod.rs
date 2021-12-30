//
// Wildland Project
// 
// Copyright © 2021 Golem Foundation,
// 	    	     Piotr K. Isajew <piotr@wildland.io>
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

use crate::error::{CargoError, CargoErrorRepresentable};
use std::fmt;

use bip39::{Mnemonic,Language,};
use hkdf::Hkdf;
use ed25519_bip32::{XPrv};

use sha2::Sha256;
use hex_literal::hex;

#[derive(Copy,Clone,PartialEq,Debug)]
pub enum IdentityError {
    InvalidWordVector = 1,
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

#[derive(Debug,PartialEq)]
pub struct Identity {
    xprv: XPrv,
    words: [String; 12]
}

pub fn from_random_seed() -> Box<Identity> {
    // rust-bip39
    let mnemonic = Mnemonic::generate(12).unwrap();
    let mut vec: Vec<String> = Vec::new();
    for word in mnemonic.word_iter() {
        vec.push(word.to_string());
    }
    from_mnemonic(&vec).unwrap()
}


pub fn from_mnemonic(phrase: &Vec<String>) -> Result<Box<Identity>, CargoError> {
    if phrase.len() != 12 {
        return Err(IdentityError::InvalidWordVector.into())
    }
    // Passphrases are great for plausible deniability in case of a cryptocurrency wallet.
    // We don't need them here.
    let passphrase = "";
    let mnemonic_string: String = phrase.join(" ");
    match Mnemonic::parse_in_normalized(Language::English, &mnemonic_string) {
        Err(_error) => Err(IdentityError::InvalidWordVector.into()),
        Ok(mnemonic) => {
            let seed = mnemonic.to_seed_normalized(passphrase);
            // Seed here is randomness of high quality (it is hard to guess).
            // But we only have 64 bytes of it, and we need extra 32 bytes for
            // BIP32's "chain code", which should satisfy following requirements:
            // 1. be deterministic
            // 2. look like good randomness
            // 3. be public, since it will be used as a part of both XPrv and XPub!
            // To achieve this, we use key derivation function (KDF).
            // A very standard variant of that is HKDF.
            let mut output_key_material = [0u8; 96];
            extend_seed(seed, &mut output_key_material);

            // Now we can use this randomness as bip32-ed25519 extended private key
            let root_xprv = XPrv::normalize_bytes_ed25519(output_key_material);
            let mut words: [String; 12] = Default::default();
            for (i, word) in phrase.iter().enumerate() {
                words[i] = word.to_string();
            }
            Ok(Box::new(Identity {xprv: root_xprv, words: words}))
        }
    }
}

fn extend_seed(seed: [u8; 64], target: &mut [u8; 96]) {
    let initial_key_material = seed;
    let info = hex!("57696c646c616e64"); // b'Wildland'.hex()
    let hk = Hkdf::<Sha256>::new(None, &initial_key_material);
    hk.expand(&info, target).expect("Should return 96 bytes of randomness");
}

impl Identity {
    pub fn mnemonic(&self) -> Vec<String> {
        let mut result: Vec<String> = vec!["".to_string(); 12];
        for (i, word) in self.words.iter().enumerate() {
            result[i] = word.to_string();
        }
        result
    }

    pub fn signing_key(&self, index: u64) -> Box<KeyPair> {
        todo!();
    }

    pub fn encryption_key(&self, index: u64) -> Box<KeyPair> {
        todo!();
    }

    pub fn single_use_encryption_key(&self, index: u64) -> Box<KeyPair> {
        todo!();
    }
}

pub struct KeyPair {
    pubkey: Vec<u8>,
    seckey: Vec<u8>
}

impl KeyPair {
    pub fn pubkey_str(&self) -> &String {
        todo!()
    }

    pub fn seckey_str(&self) -> &String {
        todo!()
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn can_generate_seed_for_phrase() {
        let user = from_random_seed();
        assert_eq!(user.mnemonic().len(), 12);
    }

    #[test]
    fn expanding_the_seed() {
        // vectors constructed using 'hkdf' python package
        let ikm = hex!("
            0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b
            0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b
            0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b
            0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b
        ");
        let mut output_key_material = [0u8; 96];
        extend_seed(ikm, &mut output_key_material);
        let (secret_key, chain_code) = output_key_material.split_at(64);
        let expected_secret = hex!("
            540d175899e60c3fae2e80592a19ef98
            3b26186b5b4be4bbb9cf590ab401d689
            7e293e76ac281196ec04b7bc68d2e8a0
            36ef6b6171f6fcde3836fdaacbd1a661
        ");
        assert_eq!(secret_key, expected_secret);
        let expected_chain_code = hex!("
            d4d1716dc1a50023fc97267109d4e4e7
            b1ff0ba00e5404d7127b48bfd4900e79
        ");
        assert_eq!(chain_code, expected_chain_code);
    }

    #[test]
    fn can_recover_seed() {
        let mnemonic_string = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic_vec: Vec<String> =
            mnemonic_string
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let user = from_mnemonic(&mnemonic_vec).unwrap();
        assert_eq!(user.mnemonic().len(), 12);
    }
}
