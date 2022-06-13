#[cfg(feature = "bindings")]
fn main() {
    use ffi_macro_build::parse_ffi_module;
    parse_ffi_module("src/ffi/mod.rs", "./_temporary/").unwrap();
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
}

#[cfg(not(feature = "bindings"))]
fn main() {}