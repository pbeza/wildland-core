use ffi_parser::BindingModule;
use std::io::Write;
use syn::{File, Item, __private::ToTokens};

macro_rules! generate_files {
    ($for_cxx:expr, $out_dir:ident, $filename:expr, $module:ident) => {{
        let parsed = if $for_cxx {
            BindingModule::translate_cxx_module($module.clone()).unwrap()
        } else {
            BindingModule::translate_swift_module($module.clone()).unwrap()
        };
        let mut output_rust = std::fs::File::create(format!("{}/{}", $out_dir, $filename)).unwrap();
        output_rust
            .write_all(parsed.get_module().to_token_stream().to_string().as_bytes())
            .unwrap();
        if $for_cxx {
            let mut output_interface =
                std::fs::File::create(format!("{}/generated.i", $out_dir)).unwrap();
            output_interface
                .write_all(
                    parsed
                        .generate_swig_interface_file_from_cxx_module()
                        .as_bytes(),
                )
                .unwrap();
        }
    }};
}

pub fn parse_ffi_module(path: &str, out_dir: &str) -> Result<(), std::io::Error> {
    let file = std::fs::read_to_string(path)?;
    let mut file: File = syn::parse_str(&file).unwrap();
    for item in file.items.iter_mut() {
        if let Item::Mod(module) = item {
            generate_files!(true, out_dir, "ffi_cxx.rs", module);
            generate_files!(false, out_dir, "ffi_swift.rs", module);
        }
    }
    // Build Swift bridge
    use std::path::PathBuf;
    let swift_out_dir = PathBuf::from("./wildland_swift");
    let bridges = vec![format!("{}/ffi_swift.rs", out_dir)];
    swift_bridge_build::parse_bridges(bridges).write_all_concatenated(swift_out_dir, "wildland");

    // Build CXX bridge
    cxx_build::bridge(format!("{}/ffi_cxx.rs", out_dir))
        .flag_if_supported("-std=c++20")
        .compile("wildland");
    Ok(())
}
