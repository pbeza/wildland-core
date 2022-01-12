//
// Wildland Project
//
// Copyright © 2021 Golem Foundation,
// 	    	     Pawel Peregud <pepesza@wildland.io>
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

use std::fmt;

use bip39::{Language, Mnemonic};
use cryptoxide::ed25519::to_public;
use ed25519_bip32::{DerivationScheme, XPrv};
use hex::encode;
use hkdf::Hkdf;
use sha2::{Sha256, Digest};

use crate::error::{CargoError, CargoErrorRepresentable};

#[derive(Copy, Clone, PartialEq, Debug)]
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

#[derive(Debug, PartialEq)]
pub struct Identity {
    xprv: XPrv,
    words: [String; 12],
}

pub fn from_entropy(entropy: &Vec<u8>) -> Result<Box<Identity>, CargoError> {
    // assume high quality entropy of arbitrary length (>= 32 bytes)
    if (entropy.len() * 8) < 128 {
        return Err(IdentityError::EntropyTooLow.into());
    }
    let mut hasher = Sha256::new();
    hasher.update(entropy);
    let hashed_entropy = hasher.finalize();
    let mnemonic = Mnemonic::from_entropy(&hashed_entropy[0..16]).unwrap();
    let mut vec: Vec<String> = Vec::new();
    for word in mnemonic.word_iter() {
        vec.push(word.to_string());
    }
    from_mnemonic(&vec)
}

pub fn from_random_seed() -> Box<Identity> {
    let mnemonic = Mnemonic::generate(12).unwrap();
    let mut vec: Vec<String> = Vec::new();
    for word in mnemonic.word_iter() {
        vec.push(word.to_string());
    }
    from_mnemonic(&vec).unwrap()
}

pub fn from_mnemonic(phrase: &Vec<String>) -> Result<Box<Identity>, CargoError> {
    if phrase.len() != 12 {
        return Err(IdentityError::InvalidWordVector.into());
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
            Ok(Box::new(Identity {
                xprv: root_xprv,
                words: words,
            }))
        }
    }
}

fn extend_seed(seed: [u8; 64], target: &mut [u8; 96]) {
    let initial_key_material = seed;
    let info = [87, 105, 108, 100, 108, 97, 110, 100]; // list(b'Wildland')
    let hk = Hkdf::<Sha256>::new(None, &initial_key_material);
    hk.expand(&info, target)
        .expect("Should return 96 bytes of randomness");
}

impl Identity {
    pub fn mnemonic(&self) -> Vec<String> {
        let mut result: Vec<String> = vec!["".to_string(); 12];
        for (i, word) in self.words.iter().enumerate() {
            result[i] = word.to_string();
        }
        result
    }

    pub fn signing_key(&self) -> Box<KeyPair> {
        self.derive(&signing_key_path())
    }

    pub fn encryption_key(&self, index: u64) -> Box<KeyPair> {
        self.derive(&encryption_key_path(index))
    }

    pub fn single_use_encryption_key(&self, index: u64) -> Box<KeyPair> {
        self.derive(&single_use_encryption_key_path(index))
    }

    fn derive(&self, path: &str) -> Box<KeyPair> {
        let mut tokens: Vec<&str> = path.split("/").collect();
        if tokens[1] != "m" {
            panic!("Derivation path must start with m");
        }
        tokens.reverse();
        tokens.pop();
        tokens.pop();
        tokens.reverse();

        let mut secret_xprv: XPrv = self.xprv.clone();
        for derivation_index in tokens {
            let di: u32 = u32::from_str_radix(derivation_index, 16).unwrap();
            secret_xprv = (&secret_xprv).derive(DerivationScheme::V2, di);
        }

        // drop the chain-code from xprv to get secret key
        let secret: Vec<u8> = secret_xprv.as_ref()[..64].to_vec();
        // drop the chain-code from xprv and generate public key from the secret key
        let public = to_public(&secret_xprv.as_ref()[0..64]).to_vec();
        Box::new(KeyPair {
            pubkey: public,
            seckey: secret,
        })
    }
}

fn signing_key_path() -> String {
    // "master/WLD/purpose/index"
    // "574c44" == b'WLD'.hex()
    "/m/574c44/0/0".to_string()
}

fn encryption_key_path(index: u64) -> String {
    format!("/m/574c44/1/{}", index)
}

fn single_use_encryption_key_path(index: u64) -> String {
    format!("/m/574c44/2/{}", index)
}

pub struct KeyPair {
    pubkey: Vec<u8>,
    seckey: Vec<u8>,
}

impl KeyPair {
    pub fn pubkey_str(&self) -> String {
        encode(self.pubkey.as_slice())
    }

    pub fn seckey_str(&self) -> String {
        encode(self.seckey.as_slice())
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use cryptoxide::ed25519;
    use cryptoxide::ed25519::SIGNATURE_LENGTH;
    use hex_literal::hex;

    const MSG: &'static [u8] = b"Hello World";

    fn user() -> Box<Identity> {
        let mnemonic_string = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic_vec: Vec<String> = mnemonic_string
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        from_mnemonic(&mnemonic_vec).unwrap()
    }

    fn sign(message: &[u8], seckey: &[u8]) -> [u8; SIGNATURE_LENGTH] {
        ed25519::signature_extended(message, seckey)
    }

    fn verify(message: &[u8], pubkey: &[u8], signature: [u8; SIGNATURE_LENGTH]) -> bool {
        ed25519::verify(message, pubkey, &signature)
    }

    #[test]
    fn can_sign_and_check_signatures_with_derived_keypair() {
        let user = user();
        let skey: Box<KeyPair> = user.signing_key();
        let signature = sign(MSG, &skey.seckey);
        let is_valid = verify(MSG, &skey.pubkey, signature);
        assert!(is_valid)
    }

    #[test]
    fn can_generate_distinct_keypairs() {
        let user = user();
        let skey: Box<KeyPair> = user.signing_key();
        println!("signing key: {}", skey.seckey_str());
        let e0key: Box<KeyPair> = user.encryption_key(0);
        println!("encryp0 key: {}", e0key.seckey_str());
        let e1key: Box<KeyPair> = user.encryption_key(1);
        println!("encryp1 key: {}", e1key.seckey_str());
        assert_ne!(skey.seckey_str(), e0key.seckey_str());
        assert_ne!(e0key.seckey_str(), e1key.seckey_str());

        assert_eq!(skey.seckey_str().len(), 128);
        assert_eq!(skey.pubkey_str().len(), 64);
    }

    #[test]
    fn can_generate_seed_for_phrase() {
        let user = from_random_seed();
        assert_eq!(user.mnemonic().len(), 12);
    }

    #[test]
    fn can_generate_from_entropy() {
        let entropy = hex!("
            65426aa1176159d1929caea10514cddd
            d11235741001f125922f258a58716b58
            da63e3060fe461fe37e4ed201d76b132
            e35830929b0f4764e577d3da09ecb6d2
            12
        ");
        let user = from_entropy(&entropy.to_vec()).ok().unwrap();
        println!("mnemonic: {}", user.mnemonic().join(" "));
    }

    #[test]
    fn will_crash_on_low_entropy_source() {
        let entropy = hex!("
            65426aa1176159d1929caea10514
        ");
        assert_eq!(true, from_entropy(&entropy.to_vec()).is_err());
    }

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

    #[test]
    fn can_recover_seed() {
        let mnemonic_string = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic_vec: Vec<String> = mnemonic_string
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let user = from_mnemonic(&mnemonic_vec).unwrap();
        assert_eq!(user.mnemonic().len(), 12);
    }
}
