mod array;
mod rcref;

#[cxx::bridge(namespace = "wildland")]
mod ffi_cxx {
    extern "Rust" {
        fn test();
    }
}

#[swift_bridge::bridge]
mod ffi_bridge {
    extern "Rust" {
        fn test();
    }
}

#[allow(dead_code)]
fn test() {
    println!("Hello World!");
}
