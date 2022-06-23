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

// TODO unify with crate::error::CryptoError when is needed
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key has incorrect length - should be 32 bytes long. Key length = {0}")]
    CannotCreateKeyError(usize),
    #[error("Cannot encrypt message: {0}")]
    CannotEncryptMessageError(String),
    #[error("Cannot decrypt message from ciphertext: {0}")]
    CannotDecryptMessageError(String),
    #[error("Cannot verify message: {0}")]
    CannotVerifyMessageError(String),
}
