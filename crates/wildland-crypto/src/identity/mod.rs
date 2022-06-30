//
// Wildland Project
//
// Copyright © 2021 Golem Foundation,
// 	    	     Pawel Peregud <pepesza@wildland.io>
// 	    	     Piotr K. Isajew <piotr@wildland.io>
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

use crate::error::CryptoError;
pub use crate::identity::{derivation::Identity, signing_keypair::SigningKeypair};
use hex::FromHex;
pub use seed::{generate_random_seed_phrase, SeedPhraseWordsArray, SEED_PHRASE_LEN};

mod derivation;
pub mod encrypting_keypair;
mod seed;
pub mod signing_keypair;

fn bytes_key_from_str(key: &str) -> Result<[u8; 32], CryptoError> {
    <[u8; 32]>::from_hex(key).map_err(|_| CryptoError::KeyParsingError(key.len()))
}
