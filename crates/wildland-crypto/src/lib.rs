//
// Wildland Project
//
// Copyright © 2022 Golem Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as published by
// the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Wildland crypto crate: derivation of keys from mnemonic, signing and encryption
//!
//! This crate provides tools to derive keypairs (for signing and encryption)
//! from 12 mnemonic words or Ethereum signature.

mod common;
pub mod error;
pub mod identity;
pub mod signature;
