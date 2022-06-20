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

use std::convert::TryFrom;

use bip39::{Language::English, Mnemonic, Seed};
use crypto_box::SecretKey as EncryptionSecretKey;
use ed25519_bip32::{DerivationScheme, XPrv};
use ed25519_dalek::SecretKey as SigningSecretKey;
use sha2::{Digest, Sha256};

use crate::{
    error::CryptoError,
    identity::{
        keys::{EncryptingKeypair, SigningKeypair},
        seed::{extend_seed, SeedPhraseWords, SEED_PHRASE_LEN},
    },
};

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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Identity {
    xprv: XPrv,
    words: SeedPhraseWords,
}

impl TryFrom<&SeedPhraseWords> for Identity {
    type Error = CryptoError;

    fn try_from(seed_phrase: &SeedPhraseWords) -> Result<Self, Self::Error> {
        let mnemonic = Mnemonic::from_phrase(&seed_phrase.join(" "), English)
            .map_err(|e| CryptoError::IdentityGenerationError(e.to_string()))?;
        Self::try_from(mnemonic)
    }
}

impl TryFrom<Mnemonic> for Identity {
    type Error = CryptoError;

    /// Derive identity from Mnemonic.
    ///
    /// Derived identity is bound to Wildland project - same 12 words will
    /// produce different seed (number) in other project.
    fn try_from(mnemonic: Mnemonic) -> Result<Self, Self::Error> {
        // Passphrases are great for plausible deniability in case of a cryptocurrency wallet.
        // We don't need them here.
        let passphrase = "";
        let seed = Seed::new(&mnemonic, passphrase);
        // Seed here is randomness of high quality (it is hard to guess).
        // But we only have 64 bytes of it, and we need extra 32 bytes for
        // BIP32's "chain code", which should satisfy following requirements:
        // 1. be deterministic
        // 2. look like good randomness
        // 3. be public, since it will be used as a part of both XPrv and XPub!
        // To achieve this, we use key derivation function (KDF).
        // A very standard variant of that is HKDF.
        let mut output_key_material = [0u8; 96];
        extend_seed(seed.as_bytes(), &mut output_key_material);

        // Now we can use this randomness as bip32-ed25519 extended private key
        let root_xprv = XPrv::normalize_bytes_ed25519(output_key_material);

        Ok(Identity {
            xprv: root_xprv,
            words: mnemonic
                .phrase()
                .split(' ')
                .map(|word| word.to_owned())
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|e: Vec<_>| {
                    CryptoError::IdentityGenerationError(format!(
                        "Invalid seed phrase length: {} - expected {}",
                        e.len(),
                        SEED_PHRASE_LEN
                    ))
                })?,
        })
    }
}

impl Identity {
    pub fn get_xprv(&self) -> &XPrv {
        &self.xprv
    }

    pub fn get_seed_phrase(&self) -> SeedPhraseWords {
        self.words.clone()
    }

    /// Deterministically derive Wildland identity from Ethereum
    /// signature (or any random bits). Assumes high quality entropy
    /// and does not perform any checks.
    #[allow(clippy::ptr_arg)]
    pub fn from_entropy(entropy: &Vec<u8>) -> Result<Self, CryptoError> {
        // assume high quality entropy of arbitrary length (>= 32 bytes)
        if (entropy.len() * 8) < 128 {
            return Err(CryptoError::EntropyTooLow);
        }
        let mut hasher = Sha256::new();
        hasher.update(entropy);
        let hashed_entropy = hasher.finalize();
        let mnemonic = Mnemonic::from_entropy(&hashed_entropy[0..16], English).unwrap();
        Self::try_from(mnemonic)
    }

    /// Derive the key that can be used to sign user manifest.
    /// Pubkey represents user to the world.
    pub fn signing_keypair(&self) -> SigningKeypair {
        self.derive_signing_keypair(&signing_key_path())
    }

    /// Derive current encryption key, used to encrypt secrets to the user.
    /// This keypair should be rotated whenever any of user's devices
    /// is compromised / stolen / lost.
    /// Current encryption pubkey should be accessible to anyone
    /// willing to communicate with the user.
    pub fn encryption_keypair(&self, index: u64) -> EncryptingKeypair {
        self.derive_encryption_keypair(&encryption_key_path(index))
    }

    /// Deterministically derive single-use encryption key. Send it to
    /// the seller of storage, so it can use it to encrypt your storage
    /// credentials.
    /// By bumping index, one can create multiple keys to be used
    /// with different on-chain identities, making linking the purchases
    /// harder.
    pub fn single_use_encryption_keypair(&self, index: u64) -> EncryptingKeypair {
        self.derive_encryption_keypair(&single_use_encryption_key_path(index))
    }

    fn derive_signing_keypair(&self, path: &str) -> SigningKeypair {
        let private_key = self.derive_private_key_from_path(path);

        // drop both the chain-code from xprv and last 32 bytes
        let seckey_bytes: [u8; 32] = <[u8; 32]>::try_from(&private_key.as_ref()[..32]).unwrap();
        let seckey = SigningSecretKey::from_bytes(&seckey_bytes).unwrap();
        let pubkey = (&seckey).into();
        SigningKeypair {
            secret: seckey,
            public: pubkey,
        }
    }

    fn derive_encryption_keypair(&self, path: &str) -> EncryptingKeypair {
        let private_key: XPrv = self.derive_private_key_from_path(path);

        // Drop the chain-code from xprv - it is no longer needed. This leaves 64 bytes.
        // Encryption in libsodium works on 32 byte keys, while what we have is 64 bytes.
        // Curve25519 keys are created from random bytes. Here we just trim.
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&private_key.as_ref()[..32]);
        // As for the key clamping - it is handled by crypto_box::SecretKey
        let curve25519_sk = EncryptionSecretKey::from(bytes);
        let curve25519_pk = curve25519_sk.public_key();

        EncryptingKeypair {
            secret: curve25519_sk,
            public: curve25519_pk,
        }
    }

    fn derive_private_key_from_path(&self, path: &str) -> XPrv {
        let mut tokens = path.split('/');

        match (tokens.next(), tokens.next()) {
            (Some(""), Some("m")) => {
                tokens.fold(self.xprv.clone(), |secret_xprv, derivation_index| {
                    let di = u32::from_str_radix(derivation_index, 16).unwrap();
                    (&secret_xprv).derive(DerivationScheme::V2, di)
                })
            }
            _ => panic!("Derivation path must start with '/m/'"), // TODO replace panic with some error handling
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::signature::{sign, verify};
    use ed25519_bip32::XPrv;
    use ed25519_dalek::Signature;
    use hex::encode;
    use hex_literal::hex;

    use crate::common::test_utilities::MNEMONIC_PHRASE;

    use super::*;

    const MSG: &[u8] = b"Hello World";

    fn user() -> Identity {
        let mnemonic = Mnemonic::from_phrase(MNEMONIC_PHRASE, English).unwrap();
        Identity::try_from(mnemonic).unwrap()
    }

    #[test]
    fn can_sign_and_check_signatures_with_derived_keypair() {
        let user = user();
        let keypair = user.signing_keypair();
        let signature: Signature = sign(MSG, &keypair);
        assert!(verify(MSG, &signature, &keypair.public).is_ok());
    }

    #[test]
    fn cannot_verify_signature_for_other_message() {
        let user = user();
        let keypair = user.signing_keypair();
        let signature: Signature = sign(MSG, &keypair);
        assert!(verify("invalid message".as_ref(), &signature, &keypair.public).is_err());
    }

    #[test]
    fn can_generate_distinct_keypairs() {
        let user = user();
        let skeypair = user.signing_keypair();
        println!("signing key, sec {}", encode(skeypair.secret.as_bytes()));
        println!("signing key, pub {}", encode(skeypair.public.as_bytes()));
        let e0key = user.encryption_keypair(0);
        println!("encryp0 key, sec: {}", encode(e0key.secret.as_bytes()));
        println!("encryp0 key, pub: {}", encode(e0key.public.as_bytes()));
        let e1key = user.encryption_keypair(1);
        assert_ne!(skeypair.secret.as_bytes(), e0key.secret.as_bytes());
        assert_ne!(e0key.secret.as_bytes(), e1key.secret.as_bytes());

        assert_eq!(encode(skeypair.secret.as_bytes()).len(), 64);
        assert_eq!(encode(skeypair.public.as_bytes()).len(), 64);
    }

    const TEST_MNEMONIC_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
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
        let user = Identity::from_entropy(&entropy.to_vec()).ok().unwrap();
        assert_eq!(
            [
                "expect".to_owned(),
                "cruel".to_owned(),
                "stadium".to_owned(),
                "sand".to_owned(),
                "couch".to_owned(),
                "garden".to_owned(),
                "nothing".to_owned(),
                "wool".to_owned(),
                "grocery".to_owned(),
                "shop".to_owned(),
                "noise".to_owned(),
                "voice".to_owned()
            ],
            user.get_seed_phrase()
        );
    }

    #[test]
    fn will_crash_on_low_entropy_source() {
        let entropy = hex!(
            "
            65426aa1176159d1929caea10514
        "
        );
        assert!(Identity::from_entropy(&entropy.to_vec()).is_err());
    }

    #[test]
    fn can_generate_from_mnemonic() {
        let mnemonic_array: [String; 12] = TEST_MNEMONIC_12
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .try_into()
            .unwrap();
        let user = Identity::try_from(&mnemonic_array).unwrap();

        assert_eq!(user.get_xprv(), &XPrv::normalize_bytes_ed25519(ROOT_XPRV))
    }

    #[test]
    fn should_fail_on_not_english_mnemonic() {
        let mnemonic_array: [String; 12] = TEST_MNEMONIC_ITALIAN
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .try_into()
            .unwrap();

        assert!(Identity::try_from(&mnemonic_array).is_err());
    }

    #[test]
    fn can_recover_seed() {
        let mnemonic_array: [String; 12] = TEST_MNEMONIC_12
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .try_into()
            .unwrap();
        let user = Identity::try_from(&mnemonic_array).unwrap();
        assert_eq!(user.get_seed_phrase().join(" "), TEST_MNEMONIC_12);
    }
}
