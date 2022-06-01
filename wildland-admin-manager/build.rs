#[cfg(feature = "bindings")]
fn main() {
    use ffi_macro_build::ffi_macro_build;
    // use std::env;
    // let out_dir = env::var("CARGO_BUILD_TARGET_DIR").unwrap();
    // let out_dir = ;
    ffi_macro_build::parse_ffi_module("src/ffi/mod.rs", "./_temporary/").unwrap();
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
}

#[cfg(not(feature = "bindings"))]
fn main() {}
