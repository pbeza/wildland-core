use std::env;
use std::path::PathBuf;

fn main() {
    if build_target::target_os() == Ok(build_target::Os::Emscripten) {
        let emscripten_path = env::var("EMSDK").unwrap();

        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .clang_arg(format!(
                "-I{emscripten_path}/upstream/emscripten/cache/sysroot/include/"
            ))
            .clang_arg("-fvisibility=default") // fix for https://github.com/rust-lang/rust-bindgen/issues/751
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .unwrap();
    }
}
