//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
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

use crate::error::CryptoError;
use hex::FromHex;

mod derivation;
mod device;
pub mod encrypting_keypair;
mod seed;
pub mod signing_keypair;

pub use crate::identity::{
    derivation::Identity, seed::generate_random_mnemonic, signing_keypair::SigningKeypair,
};
pub use device::new_device_identity;

pub const MNEMONIC_LEN: usize = 12;
pub type MnemonicPhrase = [String; MNEMONIC_LEN];

#[tracing::instrument(level = "debug", ret)]
fn bytes_key_from_str(key: &str) -> Result<[u8; 32], CryptoError> {
    <[u8; 32]>::from_hex(key).map_err(|_| CryptoError::KeyParsingError(key.len()))
}
