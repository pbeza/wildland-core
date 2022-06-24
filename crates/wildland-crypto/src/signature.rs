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

use ed25519_dalek::{PublicKey, Signature, Verifier};
use hex::{encode, ToHex};

use crate::identity::error::CryptoError;
use crate::identity::error::CryptoError::CannotVerifyMessageError;
use crate::identity::SigningKeypair;

pub fn encode_signature(signature: Signature) -> String {
    signature.encode_hex::<String>()
}

pub fn sign(msg: &[u8], keypair: &SigningKeypair) -> Signature {
    keypair.sign(msg)
}

pub fn verify(
    msg: &[u8],
    signature: &Signature,
    public_key: &PublicKey,
) -> Result<(), CryptoError> {
    public_key
        .verify(msg, signature)
        .map_err(|_| CannotVerifyMessageError(encode(msg)))
}

#[cfg(test)]
mod tests {
    use crate::common::test_utilities::{
        generate_message, get_expected_message, SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY,
    };
    use crate::identity::keys::Keypair;
    use crate::identity::SigningKeypair;
    use crate::signature::{sign, verify};

    #[test]
    fn should_sign_custom_struct() {
        // given
        let keypair = SigningKeypair::from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap();
        let message_to_sign = generate_message();
        let expected_message = get_expected_message();

        // when
        let signature = sign(&message_to_sign, &keypair);

        // then
        verify(&expected_message, &signature, &keypair.public).expect("OK");
    }
}
