#[cfg(feature = "bindings")]
fn main() -> Result<(), String> {
    use ffi_macro_build::parse_ffi_module;
    let cpp_target_dir =
        std::env::var("CPP_TARGET_DIR").unwrap_or_else(|_| "./_generated_cpp/".to_owned());
    parse_ffi_module("src/ffi/mod.rs", &cpp_target_dir)?;
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
    Ok(())
}

#[cfg(not(feature = "bindings"))]
fn main() {}
