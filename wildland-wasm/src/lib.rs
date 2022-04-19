use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_version() -> String {
    wildland_admin_manager::get_version().into()
}
