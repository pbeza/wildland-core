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
}
