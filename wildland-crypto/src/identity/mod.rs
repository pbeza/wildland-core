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

use crate::error::{CargoError, CargoErrorRepresentable};
use std::fmt;

#[derive(Copy,Clone,PartialEq,Debug)]
pub enum IdentityError {
    InvalidWordVector = 1,
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl CargoErrorRepresentable for IdentityError {
    const CARGO_ERROR_TYPE: &'static str = "IdentityError";

    fn error_code(&self) -> String {
        self.to_string()
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Identity {
}

pub fn from_random_seed() -> Box<Identity> {
    todo!();
}


pub fn from_mnemonic(phrase: &Vec<String>) -> Result<Box<Identity>, CargoError> {
    if phrase.len() != 12 {
        Err(IdentityError::InvalidWordVector.into())
    } else {
        panic!("not implemented");
    }
}

impl Identity {
    pub fn mnemonic(&self) -> Vec<String> {
        vec!("not implemented".to_string(), "yet".to_string())
    }

    pub fn signing_key(&self, index: u64) -> Box<KeyPair> {
        todo!();
    }

    pub fn encryption_key(&self, index: u64) -> Box<KeyPair> {
        todo!();
    }

    pub fn single_use_encryption_key(&self, index: u64) -> Box<KeyPair> {
        todo!();
    }
}

pub struct KeyPair {
    pubkey: Vec<u8>,
    seckey: Vec<u8>
}

impl KeyPair {
    pub fn pubkey_str(&self) -> &String {
        todo!()
    }

    pub fn seckey_str(&self) -> &String {
        todo!()
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    
    #[test]
    fn can_generate_seed_for_phrase() {
        let user = generate_random_identity();
        assert_eq!(user.get_seed_phrase().len(), 12);
    }

    #[test]
    fn can_recover_seed_from_phrase() {
        let identity = generate_random_identity();
        let phrase = identity.get_seed_phrase();
        let recovered_identity_maybe = recover_from_phrase(&phrase);
        match recovered_identity_maybe {
            Ok(recovered_identity) => assert_eq!(identity, recovered_identity),
            Err(error) => panic!(error)
        }
    }
}
