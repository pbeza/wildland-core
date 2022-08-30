//
// Wildland Project
//
// Copyright © 2021 Golem Foundation,
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

use crate::identity::signing_keypair::SigningKeypair;
use rand_7::prelude::ThreadRng;
use rand_7::thread_rng;

/// Generate a device identity keypair.
/// Each forest identity will have multiple device identities - one per user's device.
/// These identities are used to sign container manifests on behalf of the forest.
/// List of device identities should be a part of "forest manifest", which is
/// signed by forest keypair itself.
/// This establishes a trust chain, where by knowing forest identity pubkey,
/// one can tell if particular container is legitimate or not.
/// All without requiring that forest keypair secret is present on any of the devices.
#[tracing::instrument(level = "debug")]
pub fn new_device_identity() -> SigningKeypair {
    let mut csprng: ThreadRng = thread_rng();
    let pair = SigningKeypair::generate(&mut csprng);
    tracing::debug!("requested new device identity",);
    pair
}
