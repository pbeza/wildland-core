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

//! Wildland cryto crate: derivation of keys from mnemonic, signing and encryption
//!
//! This crate provides tools to derive keypairs (for signing and encryption)
//! from 12 seed words or Ethereum signature.

mod common;
pub mod error;
pub mod identity;
pub mod signature;
