fn main() {
    cxx_build::bridge("src/ffi.rs")
        .flag_if_supported("-std=c++14")
        .compile("admin_demo");
    println!("cargo:rerun-if-changed=src/ffi.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
}
