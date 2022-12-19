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

pub mod catlib_service;
mod error;
mod identity;
mod lss;
mod storage;
#[cfg(test)]
mod test_utils;

pub use catlib_service::*;
pub use error::*;
pub use identity::{master::*, wildland::*};
pub use lss::*;
pub use storage::*;
pub use wildland_crypto::{
    error::CryptoError,
    identity::{
        encrypting_keypair::EncryptingKeypair, generate_random_mnemonic, Identity, MnemonicPhrase,
        SigningKeypair,
    },
    utils,
};

pub type CorexResult<T> = Result<T, CoreXError>;

pub const DEFAULT_FOREST_KEY: &str = "wildland.forest.0";

#[cfg(test)]
pub mod test_utilities {
    use crate::WildlandIdentity;
    use wildland_crypto::identity::SigningKeypair;

    pub static SIGNING_PUBLIC_KEY: &str =
        "1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f";
    pub static SIGNING_SECRET_KEY: &str =
        "e02cdfa23ad7d94508108ad41410e556c5b0737e9c264d4a2304a7a45894fc57";

    pub fn create_signing_keypair() -> SigningKeypair {
        SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap()
    }

    pub fn create_wildland_forest_identity() -> WildlandIdentity {
        WildlandIdentity::Forest(0, create_signing_keypair())
    }
}
