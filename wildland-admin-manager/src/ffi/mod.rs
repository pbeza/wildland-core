mod rcref;
mod array;


#[cfg(feature = "cxx_binding")]
#[cxx::bridge(namespace = "wildland")]
mod ffi_definition {
    extern "Rust" {
        fn test();
    }
}

#[cfg(feature = "swift_binding")]
#[swift_bridge::bridge]
mod ffi_definition {
    extern "Rust" {
        fn test();
    }
}

fn test() {
    println!("Hello World!");
}
