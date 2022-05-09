use crate::identity::keys::SigningKeyPair;
use cryptoxide::ed25519;
use cryptoxide::ed25519::SIGNATURE_LENGTH;
use hex::ToHex;

pub fn sign(message: &[u8], keypair: &dyn SigningKeyPair) -> [u8; SIGNATURE_LENGTH] {
    ed25519::signature(message, &keypair.packed())
}

pub fn verify(
    message: &[u8],
    keypair: &dyn SigningKeyPair,
    signature: [u8; SIGNATURE_LENGTH],
) -> bool {
    ed25519::verify(message, &keypair.pubkey(), &signature)
}

pub fn encode_signature(signature: [u8; SIGNATURE_LENGTH]) -> String {
    signature.encode_hex::<String>()
}

#[cfg(test)]
mod tests {
    use crate::constants::test_utilities::{PUBLIC_KEY, SECRET_KEY, TIMESTAMP};
    use crate::identity::KeyPair;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(rename(serialize = "credentialID"))]
        pub credential_id: String,
        pub timestamp: String,
    }

    #[test]
    fn should_sign_custom_struct() {
        // given
        let keypair = KeyPair::from_str(PUBLIC_KEY, SECRET_KEY).unwrap();
        let request = TestStruct {
            credential_id: PUBLIC_KEY.into(),
            timestamp: TIMESTAMP.into(),
        };
        let message = serde_json::to_vec(&request).unwrap();

        // when
        let signature = sign(&message, &keypair);
        let expected_json_str = r#"
        {
            "credentialID":"1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f",
            "timestamp":"1648541699814"
        }
        "#;
        let expected_json: Value = serde_json::from_str(expected_json_str).unwrap();
        let expected_message = serde_json::to_vec(&expected_json).unwrap();

        // then
        assert!(verify(&expected_message, &keypair, signature,));
    }
}
