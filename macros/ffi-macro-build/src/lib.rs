use ffi_parser::BindingModule;
use std::io::Write;
use std::path::PathBuf;
use syn::{File, Item};

/// Takes the Rust module containing `extern "Rust"` and prepares
/// a version understandable by `swift-bridge-build` crate.
/// It also generates two additional files: SWIG interface and C++
/// glue code for the FFI. Those can be used by SWIG code generator.
pub fn parse_ffi_module(path: &str, swift_dir: &str, cpp_dir: &str) -> Result<(), std::io::Error> {
    let file = std::fs::read_to_string(path)?;
    let mut file: File = syn::parse_str(&file).unwrap();
    for item in file.items.iter_mut() {
        if let Item::Mod(module) = item {
            let parsed = BindingModule::translate_module(module.clone()).unwrap();
            if !std::path::Path::new(swift_dir).exists() {
                std::fs::create_dir(swift_dir).unwrap();
            }
            if !std::path::Path::new(cpp_dir).exists() {
                std::fs::create_dir(cpp_dir).unwrap();
            }

            let mut output_rust =
                std::fs::File::create(format!("{}/ffi_swift.rs", swift_dir)).unwrap();
            output_rust
                .write_all(parsed.get_module().to_string().as_bytes())
                .unwrap();

            let generated_code = parsed.generate_cpp_and_swig_file();
            let mut output_interface =
                std::fs::File::create(format!("{}/ffi_cxx.h", cpp_dir)).unwrap();
            output_interface
                .write_all(generated_code.cpp_header.as_bytes())
                .unwrap();
            let mut output_interface =
                std::fs::File::create(format!("{}/ffi_swig.i", cpp_dir)).unwrap();
            output_interface
                .write_all(generated_code.swig_interface.as_bytes())
                .unwrap();
        }
    }
    let swift_out_dir = PathBuf::from(swift_dir);
    let bridges = vec![format!("{}/ffi_swift.rs", swift_dir)];
    swift_bridge_build::parse_bridges(bridges).write_all_concatenated(swift_out_dir, "ffi_swift");
    Ok(())
}
