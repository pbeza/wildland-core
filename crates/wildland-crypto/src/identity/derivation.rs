//
// Wildland Project
//
// Copyright © 2022 Golem Foundation,
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

use std::convert::TryFrom;

use bip39::{Language::English, Mnemonic, Seed};
use crypto_box::SecretKey as EncryptionSecretKey;
use ed25519_dalek_bip32::{DerivationPath, ExtendedSecretKey};
use sha2::{Digest, Sha256};

use crate::error::KeyDeriveError;
use crate::identity::{MnemonicPhrase, MNEMONIC_LEN};
use crate::{
    error::CryptoError,
    identity::{seed::extend_seed, signing_keypair::SigningKeypair},
};

use super::encrypting_keypair::EncryptingKeypair;

#[tracing::instrument(level = "debug", ret)]
fn signing_key_path(forest_index: u64) -> String {
    // "master/WLD/purpose/index"
    // "5721156" == b'WLD'.hex() converted to decimal
    format!("m/5721156'/0'/{forest_index}'")
}

#[tracing::instrument(level = "debug", ret)]
fn encryption_key_path(forest_index: u64, index: u64) -> String {
    format!("m/5721156'/1'/{forest_index}'/{index}'")
}

#[tracing::instrument(level = "debug", ret)]
fn single_use_encryption_key_path(index: u64) -> String {
    format!("m/5721156'/2'/{index}'")
}

#[tracing::instrument(level = "debug", ret)]
fn backup_key_path() -> String {
    "m/5721156'/3'".to_string()
}

/// This structure represents Wildland cryptographic identity.
///
/// It uses BIP39 and BIP32 processes to derive keypairs of three purposes:
/// - signing (not rotated, used to sign "user manifest")
/// - encryption (used by other people to encrypt secrets to the user, rotated)
/// - single-use-encryption - to transfer secrets in public
#[derive(Debug)]
pub struct Identity {
    extended_seckey: ExtendedSecretKey,
    words: MnemonicPhrase,
}

impl TryFrom<&MnemonicPhrase> for Identity {
    type Error = CryptoError;

    /// Derive identity from mnemonic phrase.
    ///
    /// Derived identity is bound to Wildland project - same 12 words will
    /// produce different seed (number) in other project.
    /// Only English language is accepted.
    #[tracing::instrument(level = "debug")]
    fn try_from(mnemonic_phrase: &MnemonicPhrase) -> Result<Self, Self::Error> {
        let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase.join(" "), English)
            .map_err(|e| CryptoError::MnemonicGenerationError(e.to_string()))?;

        Self::from_mnemonic(mnemonic)
    }
}

impl TryFrom<&[u8]> for Identity {
    type Error = CryptoError;

    /// Deterministically derive Wildland identity from Ethereum
    /// signature (or any random bits). Assumes high quality entropy
    /// and does not perform any checks.
    #[tracing::instrument(level = "debug", skip(entropy))]
    fn try_from(entropy: &[u8]) -> Result<Self, CryptoError> {
        // assume high quality entropy of arbitrary length (>= 32 bytes)
        if (entropy.len() * 8) < 128 {
            return Err(CryptoError::EntropyTooLow);
        }
        let mut hasher = Sha256::new();
        hasher.update(entropy);
        let hashed_entropy = hasher.finalize();
        let mnemonic = Mnemonic::from_entropy(&hashed_entropy[0..16], English)
            .map_err(|e| CryptoError::MnemonicGenerationError(e.to_string()))?;
        Self::from_mnemonic(mnemonic)
    }
}

impl Identity {
    /// Derive the key that represents a forest.
    /// Pubkey represents forest to the world.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn forest_keypair(&self, forest_index: u64) -> Result<SigningKeypair, KeyDeriveError> {
        tracing::debug!("deriving forest keypair");
        self.derive_forest_keypair(&signing_key_path(forest_index))
    }

    /// Derive current encryption key, used to encrypt secrets to the owner of the forest.
    /// This keypair should be rotated whenever any of user's devices
    /// is compromised / stolen / lost.
    /// Current encryption pubkey should be accessible to anyone
    /// willing to communicate with the user.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn encryption_keypair(
        &self,
        forest_index: u64,
        index: u64,
    ) -> Result<EncryptingKeypair, KeyDeriveError> {
        tracing::debug!("deriving encryption keypair");
        self.derive_encryption_keypair(&encryption_key_path(forest_index, index))
    }

    /// Deterministically derive single-use encryption key. Send it to
    /// the seller of storage, so it can use it to encrypt your storage
    /// credentials.
    /// By bumping index, one can create multiple keys to be used
    /// with different on-chain identities, making linking the purchases
    /// harder.
    /// Please note that this keys are not scoped to particular forest,
    /// since they are supposed to be used only once anyway.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn single_use_encryption_keypair(
        &self,
        index: u64,
    ) -> Result<EncryptingKeypair, KeyDeriveError> {
        self.derive_encryption_keypair(&single_use_encryption_key_path(index))
    }

    /// Deterministically derive encryption keypair that can be used
    /// to backup secrets with intent of using them later, during recovery process.
    /// This keypair is not scoped to the forest. It should be used only internally.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn backup_keypair(&self) -> Result<EncryptingKeypair, KeyDeriveError> {
        self.derive_encryption_keypair(&backup_key_path())
    }

    #[tracing::instrument(level = "debug", ret, skip(self))]
    pub fn get_mnemonic(&self) -> MnemonicPhrase {
        self.words.clone()
    }

    #[tracing::instrument(level = "debug")]
    fn from_mnemonic(mnemonic: Mnemonic) -> Result<Self, CryptoError> {
        tracing::debug!("Deriving Identity from mnemonic");
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

        // Now we can use this randomness as bip32-dalek-ed25519 extended private key
        let extended_secret_key =
            ExtendedSecretKey::from_seed(output_key_material.as_slice()).unwrap();

        Ok(Identity {
            extended_seckey: extended_secret_key,
            words: mnemonic
                .phrase()
                .split(' ')
                .map(|word| word.to_owned())
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|e: Vec<_>| {
                    CryptoError::IdentityGenerationError(format!(
                        "Invalid mnemonic phrase length: {} - expected {}",
                        e.len(),
                        MNEMONIC_LEN
                    ))
                })?,
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn derive_forest_keypair(&self, path: &str) -> Result<SigningKeypair, KeyDeriveError> {
        let derived_extended_seckey = self.derive_private_key_from_path(path)?;

        // drop both the chain-code from xprv and last 32 bytes
        let sec_key = *derived_extended_seckey.secret_key.as_bytes();
        SigningKeypair::try_from_secret_bytes(&sec_key).map_err(|e| KeyDeriveError(e.to_string()))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn derive_encryption_keypair(&self, path: &str) -> Result<EncryptingKeypair, KeyDeriveError> {
        let derived_extended_seckey = self.derive_private_key_from_path(path)?;

        // Curve25519 keys are created from random bytes. Here we just trim.
        // As for the key clamping - it is handled by crypto_box::SecretKey
        let curve25519_sk =
            EncryptionSecretKey::from(*derived_extended_seckey.secret_key.as_bytes());
        let curve25519_pk = curve25519_sk.public_key();

        Ok(EncryptingKeypair {
            secret: curve25519_sk,
            public: curve25519_pk,
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn derive_private_key_from_path(
        &self,
        path: &str,
    ) -> Result<ExtendedSecretKey, KeyDeriveError> {
        let derivation_path: DerivationPath = path.parse().unwrap();
        self.extended_seckey
            .derive(&derivation_path)
            .map_err(|e| KeyDeriveError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crypto_box::{
        aead::{Aead, AeadCore},
        SalsaBox,
    };
    use hex::encode;
    use hex_literal::hex;
    use salsa20::XNonce;

    use crate::common::test_utilities::MNEMONIC_PHRASE;

    use super::*;

    const MSG: &[u8] = b"Hello World";

    fn user() -> Identity {
        let mnemonic = Mnemonic::from_phrase(MNEMONIC_PHRASE, English).unwrap();
        Identity::from_mnemonic(mnemonic).unwrap()
    }

    // please note that this helper is for TESTS ONLY!
    fn encrypt(
        nonce: &XNonce,
        alice_keypair: &EncryptingKeypair,
        bob_keypair: &EncryptingKeypair,
    ) -> Vec<u8> {
        let salsa_box =
            crypto_box::SalsaBox::new(&alice_keypair.public.clone(), &bob_keypair.secret.clone());

        salsa_box.encrypt(nonce, MSG).unwrap()
    }

    // please note that this helper is for TESTS ONLY!
    fn decrypt(
        ciphertext: &[u8],
        nonce: &XNonce,
        alice_keypair: &EncryptingKeypair,
        bob_keypair: &EncryptingKeypair,
    ) -> crypto_box::aead::Result<Vec<u8>> {
        let salsa_box =
            crypto_box::SalsaBox::new(&bob_keypair.public.clone(), &alice_keypair.secret.clone());

        salsa_box.decrypt(nonce, ciphertext)
    }

    #[test]
    fn can_encrypt_and_decrypt_message_with_encryption_key() {
        let user = user();
        let alice_keypair: EncryptingKeypair = user.encryption_keypair(0, 0).unwrap();
        let bob_keypair: EncryptingKeypair = user.encryption_keypair(1, 0).unwrap();
        let mut rng = rand_core::OsRng;
        let nonce = SalsaBox::generate_nonce(&mut rng);

        let ciphertext = encrypt(&nonce, &alice_keypair, &bob_keypair);
        let result = decrypt(ciphertext.as_slice(), &nonce, &bob_keypair, &alice_keypair);

        assert_eq!(MSG, result.unwrap().as_slice())
    }

    #[test]
    fn can_sign_and_check_signatures_with_derived_keypair() {
        let user = user();
        let keypair = user.forest_keypair(0).unwrap();
        let signature = keypair.sign(MSG);
        assert!(signature.verify(MSG, &keypair.public()).is_ok());
    }

    #[test]
    fn cannot_verify_signature_for_other_message() {
        let user = user();
        let keypair = user.forest_keypair(0).unwrap();
        let signature = keypair.sign(MSG);
        assert!(signature.verify(MSG, &keypair.public()).is_ok());
    }

    #[test]
    fn can_generate_distinct_keypairs() {
        let user = user();
        let skeypair = user.forest_keypair(0).unwrap();
        let e0key = user.encryption_keypair(0, 0).unwrap();
        let e1key = user.encryption_keypair(0, 1).unwrap();
        assert_ne!(&skeypair.secret(), e0key.secret.as_bytes());
        assert_ne!(e0key.secret.as_bytes(), e1key.secret.as_bytes());

        assert_eq!(encode(skeypair.secret()).len(), 64);
        assert_eq!(encode(skeypair.public()).len(), 64);
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
        let user = Identity::try_from(entropy.as_ref()).ok().unwrap();
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
            user.get_mnemonic()
        );
    }

    #[test]
    fn will_crash_on_low_entropy_source() {
        let entropy = hex!(
            "
            65426aa1176159d1929caea10514
        "
        );
        assert!(Identity::try_from(entropy.as_ref()).is_err());
    }

    #[test]
    fn can_generate_from_mnemonic() {
        let mnemonic_array: MnemonicPhrase = TEST_MNEMONIC_12
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .try_into()
            .unwrap();
        let user = Identity::try_from(&mnemonic_array).unwrap();

        assert_eq!(
            user.extended_seckey.secret_key.to_bytes(),
            ExtendedSecretKey::from_seed(&ROOT_XPRV)
                .unwrap()
                .secret_key
                .to_bytes()
        )
    }

    #[test]
    fn should_fail_on_not_english_mnemonic() {
        let mnemonic_array: MnemonicPhrase = TEST_MNEMONIC_ITALIAN
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .try_into()
            .unwrap();

        assert!(Identity::try_from(&mnemonic_array).is_err());
    }

    #[test]
    fn can_recover_mnemonic() {
        let mnemonic: MnemonicPhrase = TEST_MNEMONIC_12
            .split(' ')
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .try_into()
            .unwrap();
        let user = Identity::try_from(&mnemonic).unwrap();
        assert_eq!(user.get_mnemonic().join(" "), TEST_MNEMONIC_12);
    }
}
