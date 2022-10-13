#[cfg(feature = "bindings")]
fn main() -> Result<(), String> {
    use rusty_bind_build::parse_ffi_module;
    let ffi_code_target_dir = std::env::var("FFI_CODE_TARGET_DIR")
        .unwrap_or_else(|_| "./_generated_ffi_code/".to_owned());
    parse_ffi_module("src/ffi/mod.rs", &ffi_code_target_dir)?;
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
    Ok(())
}

#[cfg(not(feature = "bindings"))]
fn main() {}
