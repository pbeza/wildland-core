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

use hex::FromHex;

use crate::error::CryptoError;

mod derivation;
mod device;
pub mod encrypting_keypair;
mod seed;
pub mod signing_keypair;

pub use device::new_device_identity;

pub use crate::identity::derivation::Identity;
pub use crate::identity::seed::generate_random_mnemonic;
pub use crate::identity::signing_keypair::SigningKeypair;

pub const MNEMONIC_LEN: usize = 12;
pub type MnemonicPhrase = [String; MNEMONIC_LEN];

fn bytes_key_from_str(key: &str) -> Result<[u8; 32], CryptoError> {
    <[u8; 32]>::from_hex(key).map_err(|_| CryptoError::KeyParsingError(key.len()))
}
