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

pub trait SigningKeyPair {
    fn pubkey(&self) -> [u8; 32];
    fn seckey(&self) -> [u8; 32];
    fn packed(&self) -> [u8; 64];
}

pub trait EncryptionKeyPair {
    fn pubkey(&self) -> [u8; 32];
    fn seckey(&self) -> [u8; 32];
}

/// KeyPair type.
///
/// Represents a keypair derived from seed. Can be used to sign or to encrypt,
/// depending on the way it was derived.
/// TODO: prevent keypair misuse with rust types!!
pub struct KeyPair {
    seckey: [u8; 32],
    pubkey: [u8; 32],
}

impl KeyPair {
    pub fn new(seckey: [u8; 32], pubkey: [u8; 32]) -> Self {
        Self { seckey, pubkey }
    }
}

impl SigningKeyPair for KeyPair {
    fn pubkey(&self) -> [u8; 32] {
        self.pubkey
    }

    fn seckey(&self) -> [u8; 32] {
        self.seckey
    }

    fn packed(&self) -> [u8; 64] {
        let mut bytes: [u8; 64] = [0; 64];
        bytes[..32].copy_from_slice(&self.seckey[..32]);
        bytes[32..64].copy_from_slice(&self.pubkey[..32]);
        bytes
    }
}

impl EncryptionKeyPair for KeyPair {
    fn pubkey(&self) -> [u8; 32] {
        self.pubkey
    }

    fn seckey(&self) -> [u8; 32] {
        self.seckey
    }
}
