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

pub struct Seed {
}

pub fn make_seed(phrase: &Vec<String>) -> Result<Box<Seed>, SeedError> {
    panic!("it's broken");
}

impl Seed {
    pub fn get_phrase(&self) -> Vec<String> {
        panic!("not going to do this");
    }
}
