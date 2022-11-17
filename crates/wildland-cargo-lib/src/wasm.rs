use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn test_rust_fn() -> String {
    "Hello from Rust".to_string()
}
