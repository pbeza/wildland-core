#[cfg(feature = "cxx_binding")]
fn main() {
    cxx_build::bridge("src/ffi/mod.rs")
        .flag_if_supported("-std=c++14")
        .compile("wildland");
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
}

#[cfg(feature = "swift_binding")]
fn main() {
    use std::path::PathBuf;
    let out_dir = PathBuf::from("./wildland_swift");

    let bridges = vec!["src/ffi/mod.rs"];
    for path in &bridges {
        println!("cargo:rerun-if-changed={}", path);
    }

    swift_bridge_build::parse_bridges(bridges).write_all_concatenated(out_dir, "wildland");
}

#[cfg(not(any(feature = "cxx_binding", feature = "swift_binding")))]
fn main() {}
