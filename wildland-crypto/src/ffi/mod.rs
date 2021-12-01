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


use crate::identity::{Seed, SeedError, generate_random_seed, recover_seed};

#[cxx::bridge]
mod identity {
    
    extern "Rust" {
        type Seed;
        type SeedError;

        fn generate_random_seed() -> Box<Seed>;
        fn recover_seed(phrase: &Vec<String>) -> Result<Box<Seed>>;

        fn get_phrase(self: &Seed) -> Vec<String>;
    }
}
