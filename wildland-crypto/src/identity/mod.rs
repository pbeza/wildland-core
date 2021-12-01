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

use std::fmt;

#[derive(Copy,Clone)]
pub enum SeedError {
    InvalidWordVector = 1,
}

impl SeedError {
    fn value(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for SeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct Seed {
}

pub fn generate_random_seed() -> Box<Seed> {
    return Box::new(Seed { });
}


pub fn recover_seed(phrase: &Vec<String>) -> Result<Box<Seed>, SeedError> {
    if phrase.len() != 12 {
        Err(SeedError::InvalidWordVector)
    } else {
        panic!("not implemented");
    }
}

impl Seed {
    pub fn get_phrase(&self) -> Vec<String> {
        vec!("not implemented".to_string(), "yet".to_string())
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    
    #[test]
    fn can_generate_seed_for_phrase() {
        let seed = generate_random_seed();
        assert_eq!(seed.get_phrase().len(), 12);
    }

    #[test]
    fn can_recover_seed_from_phrase() {
        let seed = generate_random_seed();
        let phrase = seed.get_phrase();
        let recovered_seed_maybe = recover_seed(&phrase);
        match recovered_seed_maybe {
            Ok(recovered_seed) => assert_eq!(seed, recovered_seed),
            Err(error) => panic!(error)
        }
    }
}
