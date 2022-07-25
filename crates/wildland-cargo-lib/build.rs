#[cfg(feature = "bindings")]
fn main() {
    use ffi_macro_build::parse_ffi_module;
    let swift_target_dir =
        std::env::var("SWIFT_TARGET_DIR").unwrap_or_else(|_| "./_generated_swift/".to_owned());
    let cpp_target_dir =
        std::env::var("CPP_TARGET_DIR").unwrap_or_else(|_| "./_generated_cpp/".to_owned());
    parse_ffi_module("src/ffi/mod.rs", &swift_target_dir, &cpp_target_dir);
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
}

#[cfg(not(feature = "bindings"))]
fn main() {}
