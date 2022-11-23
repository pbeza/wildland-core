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

use ed25519_dalek::{PublicKey, Verifier};
use hex::{encode, ToHex};

use crate::error::CryptoError;

#[derive(Debug)]
pub struct Signature(pub ed25519_dalek::Signature);

impl Signature {
    pub fn encode_signature(self) -> String {
        self.0.encode_hex::<String>()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn verify(&self, msg: &[u8], public_key: &[u8; 32]) -> Result<(), CryptoError> {
        PublicKey::from_bytes(public_key)
            .map_err(|e| CryptoError::MessageVerificationError(e.to_string()))?
            .verify(msg, &self.0)
            .map_err(|_| CryptoError::MessageVerificationError(encode(msg)))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::test_utilities::{
        generate_message, get_expected_message, SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY,
    };
    use crate::identity::SigningKeypair;

    #[test]
    fn should_sign_custom_struct() {
        // given
        let keypair = SigningKeypair::try_from_str(SIGNING_PUBLIC_KEY, SIGNING_SECRET_KEY).unwrap();
        let message_to_sign = generate_message();
        let expected_message = get_expected_message();

        // when
        let signature = keypair.sign(&message_to_sign);

        // then
        signature
            .verify(&expected_message, &keypair.public())
            .expect("OK");
    }
}
