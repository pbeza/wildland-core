use std::io::Write;

use ffi_parser::BindingModule;
use syn::{File, Item, __private::ToTokens};

pub fn parse_ffi_module(path: &str, out_dir: &str) -> Result<(), std::io::Error> {
    let file = std::fs::read_to_string(path)?;
    let mut file: File = syn::parse_str(&file).unwrap();
    for item in file.items.iter_mut() {
        if let Item::Mod(module) = item {
            let parsed = BindingModule::transform_module(module.clone(), true).unwrap();
            let mut output_rust = std::fs::File::create(format!("{}/ffi_cxx.rs", out_dir)).unwrap();
            output_rust
                .write_all(
                    parsed
                        .get_cxx_module()
                        .to_token_stream()
                        .to_string()
                        .as_bytes(),
                )
                .unwrap();

            let parsed = BindingModule::transform_module(module.clone(), false).unwrap();
            let mut output_rust =
                std::fs::File::create(format!("{}/ffi_swift.rs", out_dir)).unwrap();
            output_rust
                .write_all(
                    parsed
                        .get_swift_module()
                        .to_token_stream()
                        .to_string()
                        .as_bytes(),
                )
                .unwrap();

            let mut output_interface =
                std::fs::File::create(format!("{}/generated.i", out_dir)).unwrap();
            output_interface
                .write_all(
                    parsed
                        .generate_swig_interface_file_from_cxx_module()
                        .as_bytes(),
                )
                .unwrap();
        }
    }
    // Build Swift bridge
    use std::path::PathBuf;
    let swift_out_dir = PathBuf::from("./wildland_swift");
    let bridges = vec![format!("{}/ffi_swift.rs", out_dir)];
    for path in &bridges {
        println!("cargo:rerun-if-changed={}", path);
    }
    swift_bridge_build::parse_bridges(bridges).write_all_concatenated(swift_out_dir, "wildland");

    // Build CXX bridge
    cxx_build::bridge(format!("{}/ffi_cxx.rs", out_dir))
        .flag_if_supported("-std=c++20")
        .compile("wildland");
    Ok(())
}
