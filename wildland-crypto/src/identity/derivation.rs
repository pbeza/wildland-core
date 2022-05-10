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

use bip39::Mnemonic;
use cryptoxide::ed25519::keypair;
use ed25519_bip32::{DerivationScheme, XPrv};

use crate::identity::seed::extend_seed;
use crate::identity::KeyPair;

use crate::identity::keys::{EncryptionKeyPair, SigningKeyPair};
use std::convert::TryFrom;

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

/// This structure represents Wildland cryptographic identity.
///
/// It uses BIP39 and BIP32 processes to derive keypairs of three purposes:
/// - signing (not rotated, used to sign "user manifest")
/// - encryption (used by other people to encrypt secrets to the user, rotated)
/// - single-use-encryption - to transfer secrets in public
#[derive(Debug, PartialEq)]
pub struct Identity {
    pub xprv: XPrv,
    pub words: [String; 12],
}

impl Identity {
    /// Derive identity from Mnemonic.
    ///
    /// Derived identity is bound to Wildland project - same 12 words will
    /// produce different seed (number) in other project.
    pub fn from_mnemonic(mnemonic: Mnemonic) -> Identity {
        // Passphrases are great for plausible deniability in case of a cryptocurrency wallet.
        // We don't need them here.
        let passphrase = "";
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
        for (i, word) in mnemonic.word_iter().enumerate() {
            words[i] = word.to_string();
        }
        Identity {
            xprv: root_xprv,
            words,
        }
    }

    /// Retrieve mnemonic from identity. Useful during onboarding process.
    pub fn mnemonic(&self) -> Vec<String> {
        let mut result: Vec<String> = vec!["".to_string(); 12];
        for (i, word) in self.words.iter().enumerate() {
            result[i] = word.to_string();
        }
        result
    }

    /// Derive the key that can be used to sign user manifest.
    /// Pubkey represents user to the world.
    pub fn signing_key(&self) -> impl SigningKeyPair {
        self.derive_signing_key(&signing_key_path())
    }

    /// Derive current encryption key, used to encrypt secrets to the user.
    /// This keypair should be rotated whenever any of user's devices
    /// is compromised / stolen / lost.
    /// Current encryption pubkey should be accessible to anyone
    /// willing to communicate with the user.
    pub fn encryption_key(&self, index: u64) -> impl EncryptionKeyPair {
        self.derive_encryption_key(&encryption_key_path(index))
    }

    /// Deterministically derive single-use encryption key. Send it to
    /// the seller of storage, so it can use it to encrypt your storage
    /// credentials.
    /// By bumping index, one can create multiple keys to be used
    /// with different on-chain identities, making linking the purchaces
    /// harder.
    pub fn single_use_encryption_key(&self, index: u64) -> impl EncryptionKeyPair {
        self.derive_encryption_key(&single_use_encryption_key_path(index))
    }

    fn derive_signing_key(&self, path: &str) -> impl SigningKeyPair {
        let private_key = self.derive_private_key_from_path(path);

        // drop both the chain-code from xprv and last 32 bytes
        let seckey: [u8; 32] = <[u8; 32]>::try_from(&private_key.as_ref()[..32]).unwrap();

        // drop the chain-code from xprv and generate public key from the secret key
        let (_, pubkey) = keypair(&seckey);

        KeyPair::from_bytes(seckey, pubkey)
    }

    fn derive_encryption_key(&self, path: &str) -> impl EncryptionKeyPair {
        let private_key: XPrv = self.derive_private_key_from_path(path);

        // Drop the chain-code from xprv - it is no longer needed. This leaves 64 bytes.
        // Encryption in libsodium works on 32 byte keys, while what we have is 64 bytes.
        // Curve25519 keys are created from random bytes. Here we just trim.
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&private_key.as_ref()[..32]);
        // As for the key clamping - it is handled by crypto_box::SecretKey
        let curve25519_sk = &crypto_box::SecretKey::from(bytes);
        let curve25519_pk = curve25519_sk.public_key();

        KeyPair::from_bytes(*curve25519_sk.as_bytes(), *curve25519_pk.as_bytes())
    }

    fn derive_private_key_from_path(&self, path: &str) -> XPrv {
        let tokens: Vec<&str> = path.split('/').collect();
        if !tokens[0].is_empty() || (tokens[1] != "m") {
            panic!("Derivation path must start with '/m/'");
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
    use std::str::FromStr;

    use crate::common::test_utilities::generate_random_nonce;
    use hex::encode;

    use super::*;

    const MSG: &'static [u8] = b"Hello World";
    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn user() -> Identity {
        let mnemonic = Mnemonic::from_str(MNEMONIC).unwrap();
        Identity::from_mnemonic(mnemonic)
    }

    #[test]
    fn can_sign_and_check_signatures_with_derived_keypair() {
        let user = user();
        let skey = user.signing_key();
        let signature = skey.sign(MSG);
        assert!(skey.verify(MSG, &signature));
    }

    #[test]
    fn cannot_verify_signature_for_other_message() {
        let user = user();
        let skey = user.signing_key();
        let signature = skey.sign(MSG);

        assert!(!skey.verify("invalid message".as_ref(), &signature));
    }

    #[test]
    fn can_encrypt_and_decrypt_message_with_encryption_key() {
        let user = user();
        let alice_keypair = user.encryption_key(0);
        let bob_keypair = user.encryption_key(1);
        let nonce = generate_random_nonce();

        let ciphertext = alice_keypair
            .encrypt(MSG, &nonce, &bob_keypair.pubkey())
            .unwrap();
        let result = bob_keypair.decrypt(ciphertext.as_slice(), &nonce, &alice_keypair.pubkey());

        assert_eq!(MSG, result.unwrap().as_slice())
    }

    #[test]
    fn can_encrypt_and_decrypt_message_with_single_use_encryption_key() {
        let user = user();
        let alice_keypair = user.single_use_encryption_key(0);
        let bob_keypair = user.single_use_encryption_key(1);
        let nonce = generate_random_nonce();

        let ciphertext = alice_keypair
            .encrypt(MSG, &nonce, &bob_keypair.pubkey())
            .unwrap();
        let result = bob_keypair.decrypt(ciphertext.as_slice(), &nonce, &alice_keypair.pubkey());

        assert_eq!(MSG, result.unwrap().as_slice())
    }

    #[test]
    fn cannot_decrypt_message_when_invalid_encryption_key_is_used() {
        let user = user();
        let alice_keypair = user.encryption_key(0);
        let bob_keypair = user.encryption_key(1);
        let charlie_keypair = user.encryption_key(2);
        let nonce = generate_random_nonce();

        let ciphertext = alice_keypair
            .encrypt(MSG, &nonce, &bob_keypair.pubkey())
            .unwrap();
        let result =
            charlie_keypair.decrypt(ciphertext.as_slice(), &nonce, &alice_keypair.pubkey());

        assert!(result.is_err())
    }

    #[test]
    fn cannot_decrypt_message_when_invalid_single_use_encryption_key_is_used() {
        let user = user();
        let alice_keypair = user.single_use_encryption_key(0);
        let bob_keypair = user.single_use_encryption_key(1);
        let charlie_keypair = user.single_use_encryption_key(2);
        let nonce = generate_random_nonce();

        let ciphertext = alice_keypair
            .encrypt(MSG, &nonce, &bob_keypair.pubkey())
            .unwrap();
        let result =
            charlie_keypair.decrypt(ciphertext.as_slice(), &nonce, &alice_keypair.pubkey());

        assert!(result.is_err())
    }

    #[test]
    fn cannot_decrypt_message_with_different_nonce() {
        let user = user();
        let alice_keypair = user.single_use_encryption_key(0);
        let bob_keypair = user.single_use_encryption_key(1);
        let nonce1 = generate_random_nonce();
        let nonce2 = generate_random_nonce();

        let ciphertext = alice_keypair
            .encrypt(MSG, &nonce1, &bob_keypair.pubkey())
            .unwrap();
        let result = bob_keypair.decrypt(ciphertext.as_slice(), &nonce2, &bob_keypair.pubkey());

        assert!(result.is_err())
    }

    #[test]
    fn can_generate_distinct_keypairs() {
        let user = user();
        let skey = user.signing_key();
        println!("signing key, sec {}", encode(skey.seckey_as_bytes()));
        println!("signing key, pub {}", encode(skey.pubkey_as_bytes()));
        let e0key = user.encryption_key(0);
        println!("encryp0 key, sec: {}", encode(e0key.seckey().as_bytes()));
        println!("encryp0 key, pub: {}", encode(e0key.pubkey().as_bytes()));
        let e1key = user.encryption_key(1);
        assert_ne!(skey.seckey_as_bytes(), *e0key.seckey().as_bytes());
        assert_ne!(e0key.seckey().as_bytes(), e1key.seckey().as_bytes());

        assert_eq!(encode(skey.seckey_as_bytes()).len(), 64);
        assert_eq!(encode(skey.pubkey_as_bytes()).len(), 64);
    }
}
