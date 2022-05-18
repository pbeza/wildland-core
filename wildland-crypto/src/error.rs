//
// Wildland Project
//
// Copyright © 2021 Golem Foundation,
// 	    	     Piotr K. Isajew <piotr@wildland.io>
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

// Generic error wrapper for Rust errors that need to propagate into
// the native bridge.

use wildland_admin_manager_api::AdminManagerError;

#[derive(Debug)]
pub enum CryptoError {
    SeedPhraseGenerationError(String),
    IdentityGenerationError(String),
    EntropyTooLow,
}

impl From<CryptoError> for AdminManagerError {
    fn from(crypto_err: CryptoError) -> Self {
        match crypto_err {
            CryptoError::SeedPhraseGenerationError(msg) => {
                AdminManagerError::SeedPhraseGenerationError(msg)
            }
            CryptoError::IdentityGenerationError(msg) => {
                AdminManagerError::IdentityGenerationError(msg)
            }
            CryptoError::EntropyTooLow => AdminManagerError::EntropyTooLow,
        }
    }
}
