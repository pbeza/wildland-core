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

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum CryptoError {
    #[error("Key has incorrect length - should be 32 bytes long. Key length = {0}")]
    KeyParsingError(usize),
    #[error("Cannot verify message: {0}")]
    MessageVerificationError(String),
    #[error("Invalid key signature: {0}")]
    InvalidSignatureBytesError(String),
    #[error("Failed to create a mnemonic: {0}")]
    MnemonicGenerationError(String),
    #[error("Identity generation failed: {0}")]
    IdentityGenerationError(String),
    #[error("Too low entropy")]
    EntropyTooLow,
}

#[derive(Error, Debug, PartialEq, Eq, Clone)]
#[error("Error while deriving an extended secret key fom the current using a derivation path: {0}")]
pub struct KeyDeriveError(pub String);
