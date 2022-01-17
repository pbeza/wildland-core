//
// Wildland Project
//
// Copyright © 2021 Golem Foundation,
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


use cryptoxide::ed25519::to_public;
use ed25519_bip32::{DerivationScheme, XPrv};
use sodiumoxide::crypto::sign::ed25519::SecretKey;
use sodiumoxide::crypto::sign::to_curve25519_sk;

use crate::identity::KeyPair;

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

#[derive(Debug, PartialEq)]
pub struct Identity {
    pub xprv: XPrv,
    pub words: [String; 12],
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
        self.derive_signing_key(&signing_key_path())
    }

    pub fn encryption_key(&self, index: u64) -> Box<KeyPair> {
        self.derive_encryption_key(&encryption_key_path(index))
    }

    pub fn single_use_encryption_key(&self, index: u64) -> Box<KeyPair> {
        self.derive_encryption_key(&single_use_encryption_key_path(index))
    }

    fn derive_signing_key(&self, path: &str) -> Box<KeyPair> {
        let private_key = self.derive_private_key_from_path(path);

        // drop the chain-code from xprv to get secret key
        let secret: Vec<u8> = private_key.as_ref()[..64].to_vec();
        // drop the chain-code from xprv and generate public key from the secret key
        let public = to_public(&private_key.as_ref()[0..64]).to_vec();

        Box::new(KeyPair {
            pubkey: public,
            seckey: secret,
        })
    }

    fn derive_encryption_key(&self, path: &str) -> Box<KeyPair> {
        let private_key = self.derive_private_key_from_path(path);

        // drop the chain-code from xprv to get secret key
        let ed25519_sk = SecretKey::from_slice(&private_key.as_ref()[..64]).unwrap();

        // In order to use a secret key for encryption/decryption it must be converted from 64-bytes long
        // ed25519 key to 32-bytes long curve25519 key
        let curve25519_sk = to_curve25519_sk(&ed25519_sk).unwrap();
        let curve25519_pk = curve25519_sk.public_key();

        Box::new(KeyPair {
            seckey: curve25519_sk.as_ref().to_vec(),
            pubkey: curve25519_pk.as_ref().to_vec(),
        })
    }

    fn derive_private_key_from_path(&self, path: &str) -> XPrv {
        let mut tokens: Vec<&str> = path.split("/").collect();
        if tokens[1] != "m" {
            panic!("Derivation path must start with m");
        }

        let mut secret_xprv: XPrv = self.xprv.clone();
        for derivation_index in &tokens[2..] {
            let di: u32 = u32::from_str_radix(derivation_index, 16).unwrap();
            secret_xprv = (&secret_xprv).derive(DerivationScheme::V2, di);
        }
        secret_xprv
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::sync::Once;

    use crypto_box::aead::Aead;
    use cryptoxide::ed25519;
    use cryptoxide::ed25519::SIGNATURE_LENGTH;
    use salsa20::XNonce;

    use super::*;

    const MSG: &'static [u8] = b"Hello World";
    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const ROOT_XPRV: [u8; 96] = [
        24, 97, 125, 255, 78, 254, 242, 4, 80, 221, 94, 175, 192, 96, 253, 133, 250, 172, 202,
        19, 217, 90, 206, 59, 218, 11, 227, 46, 70, 148, 252, 215, 161, 178, 196, 120, 102, 114,
        194, 12, 205, 218, 138, 151, 244, 166, 214, 35, 131, 140, 194, 70, 236, 205, 123, 72, 70,
        215, 44, 36, 182, 15, 25, 158, 117, 161, 211, 29, 125, 195, 12, 236, 138, 155, 206, 3,
        16, 11, 54, 143, 209, 223, 7, 250, 9, 252, 142, 87, 79, 214, 211, 69, 2, 147, 159, 63
    ];

    fn user() -> Box<Identity> {
        let words: [String; 12] = <[String; 12]>::try_from(MNEMONIC.split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>()).unwrap();

        Box::new(Identity {
            xprv: XPrv::normalize_bytes_ed25519(ROOT_XPRV),
            words,
        })
    }

    fn sign(message: &[u8], seckey: &[u8]) -> [u8; SIGNATURE_LENGTH] {
        ed25519::signature_extended(message, seckey)
    }

    fn verify(message: &[u8], pubkey: &[u8], signature: [u8; SIGNATURE_LENGTH]) -> bool {
        ed25519::verify(message, pubkey, &signature)
    }

    fn generate_nonce() -> XNonce {
        let mut rng = crypto_box::rand_core::OsRng;
        crypto_box::generate_nonce(&mut rng)
    }

    fn encrypt(nonce: &XNonce, secret_key: [u8; 32], public_key: [u8; 32]) -> Vec<u8> {
        let salsa_box = crypto_box::Box::new(
            &crypto_box::PublicKey::from(public_key),
            &crypto_box::SecretKey::from(secret_key));

        salsa_box.encrypt(nonce, MSG).unwrap()
    }

    fn decrypt(ciphertext: &[u8], nonce: &XNonce, secret_key: [u8; 32], public_key: [u8; 32]) -> crypto_box::aead::Result<Vec<u8>> {
        let salsa_box = crypto_box::Box::new(
            &crypto_box::PublicKey::from(public_key),
            &crypto_box::SecretKey::from(secret_key));

        salsa_box.decrypt(nonce, ciphertext)
    }

    fn convert(key: Vec<u8>) -> [u8; 32] {
        <[u8; 32]>::try_from(key.as_slice()).unwrap()
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
    fn can_encrypt_and_decrypt_message_with_encryption_key() {
        let user = user();
        let alice_keypair: Box<KeyPair> = user.encryption_key(0);
        let bob_keypair: Box<KeyPair> = user.encryption_key(1);
        let nonce = generate_nonce();

        let ciphertext = encrypt(
            &nonce,
            convert(alice_keypair.seckey),
            convert(bob_keypair.pubkey));
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce,
            convert(bob_keypair.seckey),
            convert(alice_keypair.pubkey));

        assert_eq!(MSG, result.unwrap().as_slice())
    }

    #[test]
    fn can_encrypt_and_decrypt_message_with_single_use_encryption_key() {
        let user = user();
        let alice_keypair: Box<KeyPair> = user.single_use_encryption_key(0);
        let bob_keypair: Box<KeyPair> = user.single_use_encryption_key(1);
        let nonce = generate_nonce();

        let ciphertext = encrypt(
            &nonce,
            convert(alice_keypair.seckey),
            convert(bob_keypair.pubkey));
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce,
            convert(bob_keypair.seckey),
            convert(alice_keypair.pubkey));

        assert_eq!(MSG, result.unwrap().as_slice())
    }

    #[test]
    fn cannot_decrypt_message_when_invalid_encryption_key_is_used() {
        let user = user();
        let alice_keypair: Box<KeyPair> = user.encryption_key(0);
        let bob_keypair: Box<KeyPair> = user.encryption_key(1);
        let charlie_keypair: Box<KeyPair> = user.encryption_key(2);
        let nonce = generate_nonce();

        let ciphertext = encrypt(
            &nonce,
            convert(alice_keypair.seckey),
            convert(bob_keypair.pubkey));
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce,
            convert(charlie_keypair.seckey),
            convert(alice_keypair.pubkey));

        assert!(result.is_err())
    }

    #[test]
    fn cannot_decrypt_message_when_invalid_single_use_encryption_key_is_used() {
        let user = user();
        let alice_keypair: Box<KeyPair> = user.single_use_encryption_key(0);
        let bob_keypair: Box<KeyPair> = user.single_use_encryption_key(1);
        let charlie_keypair: Box<KeyPair> = user.single_use_encryption_key(2);
        let nonce = generate_nonce();

        let ciphertext = encrypt(
            &nonce,
            convert(alice_keypair.seckey),
            convert(bob_keypair.pubkey));
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce,
            convert(charlie_keypair.seckey),
            convert(alice_keypair.pubkey));

        assert!(result.is_err())
    }

    #[test]
    fn cannot_decrypt_message_with_different_nonce() {
        let user = user();
        let alice_keypair: Box<KeyPair> = user.single_use_encryption_key(0);
        let bob_keypair: Box<KeyPair> = user.single_use_encryption_key(1);
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();

        let ciphertext = encrypt(
            &nonce1,
            convert(alice_keypair.seckey),
            convert(bob_keypair.pubkey));
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce2,
            convert(bob_keypair.seckey),
            convert(alice_keypair.pubkey));

        assert!(result.is_err())
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
}
