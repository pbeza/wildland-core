fn main() {
    cxx_build::bridge("src/ffi/mod.rs")
        .flag_if_supported("-std=c++17")
        .compile("cxxlib");
}
