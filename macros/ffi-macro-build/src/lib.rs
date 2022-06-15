use ffi_parser::BindingModule;
use std::io::Write;
use syn::{File, Item, __private::ToTokens};

pub fn parse_ffi_module(path: &str, out_dir: &str) -> Result<(), std::io::Error> {
    let file = std::fs::read_to_string(path)?;
    let mut file: File = syn::parse_str(&file).unwrap();
    for item in file.items.iter_mut() {
        if let Item::Mod(module) = item {
            let parsed = BindingModule::translate_module(module.clone()).unwrap();
            if !std::path::Path::new(out_dir).exists() {
                std::fs::create_dir(out_dir).unwrap();
            }

            let mut output_rust =
                std::fs::File::create(format!("{}/ffi_swift.rs", out_dir)).unwrap();
            output_rust
                .write_all(parsed.get_module().to_token_stream().to_string().as_bytes())
                .unwrap();

            let generated_code = parsed.generate_cpp_interface_file();
            let mut output_interface = std::fs::File::create(format!("{}/ffi.h", out_dir)).unwrap();
            output_interface
                .write_all(generated_code.cpp_header.as_bytes())
                .unwrap();
            let mut output_interface =
                std::fs::File::create(format!("{}/swig_ffi.i", out_dir)).unwrap();
            output_interface
                .write_all(generated_code.swig_interface.as_bytes())
                .unwrap();
        }
    }
    use std::path::PathBuf;
    let swift_out_dir = PathBuf::from("./wildland_swift");
    let bridges = vec![format!("{}/ffi_swift.rs", out_dir)];
    swift_bridge_build::parse_bridges(bridges).write_all_concatenated(swift_out_dir, "wildland");
    Ok(())
}
