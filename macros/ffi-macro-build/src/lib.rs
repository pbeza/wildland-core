use std::io::Write;

use ffi_parser::BindingModule;
use syn::{File, Item, __private::ToTokens};
// use cxx_build::cxx_build;

pub struct ffi_macro_build {}

impl ffi_macro_build {
    pub fn parse_ffi_module(path: &str, out_dir: &str) -> Result<(), std::io::Error> {
        let file = std::fs::read_to_string(path)?;
        let mut file: File = syn::parse_str(&file).unwrap();
        for item in file.items.iter_mut() {
            match item {
                Item::Mod(module) => {
                    let parsed = BindingModule::transform_module(module.clone()).unwrap();
                    let mut output =
                        std::fs::File::create(format!("{}/mod.rs", out_dir)).unwrap();
                    output
                        .write_all(
                            &parsed
                                .get_cxx_module()
                                .to_token_stream()
                                .to_string()
                                .as_bytes(),
                        )
                        .unwrap();
                }
                _ => {}
            }
        }

        // // // Build Swift bridge
        // // use std::path::PathBuf;
        // // let out_dir = PathBuf::from("./wildland_swift");
        // // let bridges = vec!["src/ffi/mod.rs"];
        // // for path in &bridges {
        // //     println!("cargo:rerun-if-changed={}", path);
        // // }
        // // swift_bridge_build::parse_bridges(bridges).write_all_concatenated(out_dir, "wildland");

        // Build CXX bridge
        cxx_build::bridge(format!("{}/mod.rs", out_dir))
            .flag_if_supported("-std=c++20")
            .compile("wildland");
        Ok(())
    }
}
