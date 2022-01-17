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


use crate::identity::{Identity, IdentityError, KeyPair, from_random_seed,
                      from_mnemonic, from_entropy};

#[cxx::bridge(namespace="cargo::common")]
mod identity {
    
    extern "Rust" {
        type Identity;
        type IdentityError;
        type KeyPair;

        fn from_entropy(entropy: &Vec<u8>) -> Result<Box<Identity>>;
        fn from_random_seed() -> Result<Box<Identity>>;
        fn from_mnemonic(phrase: &Vec<String>) -> Result<Box<Identity>>;

        fn mnemonic(self: &Identity) -> Vec<String>;

        fn signing_key(self: &Identity) -> Box<KeyPair>;
        fn encryption_key(self: &Identity, index: u64) -> Box<KeyPair>;
        fn single_use_encryption_key(self: &Identity, index: u64) -> Box<KeyPair>;
    }
}
