use ed25519_dalek::SIGNATURE_LENGTH;
use hex::ToHex;

pub fn encode_signature(signature: [u8; SIGNATURE_LENGTH]) -> String {
    signature.encode_hex::<String>()
}
