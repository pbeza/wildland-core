pub const WILDLAND_SIGNATURE_HEADER: &str = "X-Wildland-Signature";

#[cfg(test)]
pub mod test_utilities {
    pub static STORAGE_ID: &str = "29ad32f3-e951-4567-a68f-1bef0c564fac";
    pub static CREDENTIALS_ID: &str =
        "000e402c4b4f6638024a39180a3787ae090102e90c6a0ee1302c87fb0ac6e0e9";
    pub static CREDENTIALS_SECRET: &str =
        "c6f0a7601def190c96aaac1bedaf5dae8088dd625bf0446e10400b62ec21ba06";
    pub static SIGNATURE: &str = "db0dd4ebe6a365edcb7626c7de77b9e635564932366a1c7c314b9215d1b3b4e8cbbed2178ffe3cde361e76aef919b08efbe3b832c587d3fa9bf2d9fde85bf309";
    pub static MESSAGE: &str = "http://localhost:9000/wlbucket/d7dfa87a-23d9-42d9-addd-a0ab60db0712/?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Credential=WLSC%2F20220412%2FREGION%2Fs3%2Faws4_request&X-Amz-Date=20220412T090041Z&X-Amz-Expires=604800&X-Amz-Signature=51ca43ed012e6dfa55840880f44e9aa3fa1f38030a5471e7cdd9e0a5d2344727&X-Amz-SignedHeaders=host&x-amz-user-agent=aws-sdk-js%2F3.43.0&x-id=PutObject";
    pub static SC_RESPONSE: &str = "{\"message\":\"message\"}";
    pub static TIMESTAMP: &str = "1647950512321";
}
