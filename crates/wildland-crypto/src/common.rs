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

#[cfg(test)]
pub mod test_utilities {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    pub static MNEMONIC_PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    pub static SIGNING_PUBLIC_KEY: &str =
        "1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f";
    pub static SIGNING_SECRET_KEY: &str =
        "e02cdfa23ad7d94508108ad41410e556c5b0737e9c264d4a2304a7a45894fc57";
    pub static TIMESTAMP: &str = "1648541699814";

    #[derive(Debug, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(rename(serialize = "credentialID"))]
        pub credential_id: String,
        pub timestamp: String,
    }

    pub fn generate_message() -> Vec<u8> {
        let request = TestStruct {
            credential_id: SIGNING_PUBLIC_KEY.into(),
            timestamp: TIMESTAMP.into(),
        };
        serde_json::to_vec(&request).unwrap()
    }

    pub fn get_expected_message() -> Vec<u8> {
        let expected_json_str = r#"
        {
            "credentialID":"1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f",
            "timestamp":"1648541699814"
        }
        "#;
        let expected_json: Value = serde_json::from_str(expected_json_str).unwrap();
        serde_json::to_vec(&expected_json).unwrap()
    }
}
