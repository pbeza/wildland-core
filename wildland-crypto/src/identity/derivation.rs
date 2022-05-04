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

use std::convert::TryFrom;
use crate::identity::keys::{EncryptionKeyPair, SigningKeyPair};

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

        KeyPair::new(seckey, pubkey)
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

        KeyPair::new(*curve25519_sk.as_bytes(), *curve25519_pk.as_bytes())
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

    use crypto_box::aead::Aead;
    use cryptoxide::ed25519;
    use cryptoxide::ed25519::SIGNATURE_LENGTH;
    use salsa20::XNonce;

    use hex::encode;

    use super::*;

    const MSG: &'static [u8] = b"Hello World";
    const MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn user() -> Identity {
        let mnemonic = Mnemonic::from_str(MNEMONIC).unwrap();
        Identity::from_mnemonic(mnemonic)
    }

    fn sign(message: &[u8], keypair: &dyn SigningKeyPair) -> [u8; SIGNATURE_LENGTH] {
        ed25519::signature(message, &keypair.packed())
    }

    fn verify(message: &[u8], keypair: &dyn SigningKeyPair, signature: [u8; SIGNATURE_LENGTH]) -> bool {
        ed25519::verify(message, &keypair.pubkey(), &signature)
    }

    fn generate_nonce() -> XNonce {
        let mut rng = crypto_box::rand_core::OsRng;
        crypto_box::generate_nonce(&mut rng)
    }

    fn encrypt(nonce: &XNonce, alice_keypair: &dyn EncryptionKeyPair, bob_keypair: &dyn EncryptionKeyPair) -> Vec<u8> {
        let salsa_box = crypto_box::Box::new(
            &crypto_box::PublicKey::from(alice_keypair.pubkey()),
            &crypto_box::SecretKey::from(bob_keypair.seckey()));

        salsa_box.encrypt(nonce, MSG).unwrap()
    }

    fn decrypt(ciphertext: &[u8], nonce: &XNonce, alice_keypair: &dyn EncryptionKeyPair, bob_keypair: &dyn EncryptionKeyPair) -> crypto_box::aead::Result<Vec<u8>> {
        let salsa_box = crypto_box::Box::new(
            &crypto_box::PublicKey::from(bob_keypair.pubkey()),
            &crypto_box::SecretKey::from(alice_keypair.seckey()));

        salsa_box.decrypt(nonce, ciphertext)
    }

    #[test]
    fn can_sign_and_check_signatures_with_derived_keypair() {
        let user = user();
        let skey = user.signing_key();
        let signature = sign(MSG, &skey);
        assert!(verify(MSG, &skey, signature));
        let mut broken_signature: [u8; 64] = [0; 64];
        broken_signature.copy_from_slice(&signature);
        broken_signature[0] = !signature[0];
        assert!(!verify(MSG, &skey, broken_signature));
    }

    #[test]
    fn can_encrypt_and_decrypt_message_with_encryption_key() {
        let user = user();
        let alice_keypair = user.encryption_key(0);
        let bob_keypair = user.encryption_key(1);
        let nonce = generate_nonce();

        let ciphertext = encrypt(
            &nonce,
            &alice_keypair,
            &bob_keypair);
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce,
            &alice_keypair,
            &bob_keypair);

        assert_eq!(MSG, result.unwrap().as_slice())
    }

    #[test]
    fn can_encrypt_and_decrypt_message_with_single_use_encryption_key() {
        let user = user();
        let alice_keypair = user.single_use_encryption_key(0);
        let bob_keypair = user.single_use_encryption_key(1);
        let nonce = generate_nonce();

        let ciphertext = encrypt(
            &nonce,
            &alice_keypair,
            &bob_keypair);
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce,
            &bob_keypair,
            &alice_keypair);

        assert_eq!(MSG, result.unwrap().as_slice())
    }

    #[test]
    fn cannot_decrypt_message_when_invalid_encryption_key_is_used() {
        let user = user();
        let alice_keypair = user.encryption_key(0);
        let bob_keypair = user.encryption_key(1);
        let charlie_keypair = user.encryption_key(2);
        let nonce = generate_nonce();

        let ciphertext = encrypt(
            &nonce,
            &alice_keypair,
            &bob_keypair);
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce,
            &charlie_keypair,
            &alice_keypair);

        assert!(result.is_err())
    }

    #[test]
    fn cannot_decrypt_message_when_invalid_single_use_encryption_key_is_used() {
        let user = user();
        let alice_keypair = user.single_use_encryption_key(0);
        let bob_keypair = user.single_use_encryption_key(1);
        let charlie_keypair = user.single_use_encryption_key(2);
        let nonce = generate_nonce();

        let ciphertext = encrypt(
            &nonce,
            &alice_keypair,
            &bob_keypair);
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce,
            &charlie_keypair,
            &alice_keypair);

        assert!(result.is_err())
    }

    #[test]
    fn cannot_decrypt_message_with_different_nonce() {
        let user = user();
        let alice_keypair = user.single_use_encryption_key(0);
        let bob_keypair = user.single_use_encryption_key(1);
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();

        let ciphertext = encrypt(
            &nonce1,
            &alice_keypair,
            &bob_keypair);
        let result = decrypt(
            ciphertext.as_slice(),
            &nonce2,
            &bob_keypair,
            &alice_keypair);

        assert!(result.is_err())
    }

    #[test]
    fn can_generate_distinct_keypairs() {
        let user = user();
        let skey = user.signing_key();
        println!("signing key, sec {}", encode(skey.seckey()));
        println!("signing key, pub {}", encode(skey.pubkey()));
        let e0key = user.encryption_key(0);
        println!("encryp0 key, sec: {}", encode(e0key.seckey()));
        println!("encryp0 key, pub: {}", encode(e0key.pubkey()));
        let e1key = user.encryption_key(1);
        assert_ne!(skey.seckey(), e0key.seckey());
        assert_ne!(e0key.seckey(), e1key.seckey());

        assert_eq!(encode(skey.seckey()).len(), 64);
        assert_eq!(encode(skey.pubkey()).len(), 64);
    }
}
