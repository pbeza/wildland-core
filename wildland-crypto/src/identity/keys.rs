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


use hex::encode;
use std::convert::TryFrom;

pub struct KeyPair {
    pub pubkey: Vec<u8>,
    pub seckey: Vec<u8>,
}

impl KeyPair {
    pub fn pubkey_str(&self) -> String {
        encode(self.pubkey.as_slice())
    }

    pub fn seckey_str(&self) -> String {
        encode(self.seckey.as_slice())
    }

    pub fn pubkey_bytes(&self) -> &Vec<u8> {
	      return &self.pubkey;
    }

    pub fn seckey_bytes(&self) -> &Vec<u8> {
	      return &self.seckey;
    }

    pub fn packed(&self) -> [u8; 64] {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[..32].copy_from_slice(&self.pubkey[..32]);
        bytes.copy_within(0..32, 32);
        bytes[..32].copy_from_slice(&self.seckey[..32]);
        bytes
    }

    pub fn pubkey_array(&self) -> [u8; 32] {
        <[u8; 32]>::try_from(self.pubkey.as_slice()).unwrap()
    }
}
