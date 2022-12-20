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

use crate::WildlandIdentity::{Device, Forest};
use std::fmt::{self, Display, Formatter};
use wildland_crypto::identity::{
    signing_keypair::{PubKey, SecKey},
    SigningKeypair,
};

#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum WildlandIdentity {
    Forest(u64, SigningKeypair),
    Device(String, SigningKeypair),
}

impl Display for WildlandIdentity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Forest(index, _) => write!(f, "wildland.forest.{index}"),
            Device(name, _) => write!(f, "wildland.device.{name}"),
        }
    }
}

impl WildlandIdentity {
    pub fn get_identifier(&self) -> String {
        match self {
            Forest(index, _) => index.to_string(),
            Device(name, _) => name.to_string(),
        }
    }

    pub fn get_public_key(&self) -> PubKey {
        match self {
            Forest(_, keypair) | Device(_, keypair) => keypair.public(),
        }
    }

    pub fn get_private_key(&self) -> SecKey {
        match self {
            Forest(_, keypair) | Device(_, keypair) => keypair.secret(),
        }
    }

    pub fn get_keypair_bytes(&self) -> Vec<u8> {
        match self {
            Forest(_, keypair) | Device(_, keypair) => keypair.to_bytes(),
        }
    }

    pub fn get_keypair(&self) -> SigningKeypair {
        match self {
            Forest(_, keypair) | Device(_, keypair) => SigningKeypair::from(keypair),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        test_utilities::{SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY},
        WildlandIdentity,
    };
    use wildland_crypto::identity::SigningKeypair;

    #[test]
    fn should_get_correct_fingerprint() {
        let keypair = SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap();
        let wildland_identity = WildlandIdentity::Device("Device 1".to_string(), keypair);

        assert_eq!(
            wildland_identity.to_string(),
            "wildland.device.Device 1".to_string()
        )
    }
}
