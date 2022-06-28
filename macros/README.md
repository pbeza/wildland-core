# FFI bindings generator
This is a set of crates for generating Swift, C++ and SWIG glue code out of the input rust module containing declarations of types and functions. **It requires rust toolchain in version `>1.59.0`**.

## Setup
The following instruction is a good starting point to generate binding code in your project:

#### 0. Add the following dependencies to the `Cargo.toml` file:
```toml
[lib]
crate-type = ["staticlib", "lib"]
# [ ... ]

[dependencies]
ffi-macro = { version = "0.1.0", path = "../macros/ffi-macro" }
swift-bridge = { version = "0.1", optional = true }
# [ ... ]

[build-dependencies]
ffi-macro-build = { version = "0.1.0", path = "../macros/ffi-macro-build" }
```

#### 1. Prepare a `build.rs` file, e.g.:
```rust
fn main() {
    use ffi_macro_build::parse_ffi_module;
    parse_ffi_module("src/ffi/mod.rs", "./_generated_swift/", "./_generated_cpp/").unwrap();
    println!("cargo:rerun-if-changed=src/ffi/mod.rs");
}
```

#### 2. Prepare an FFI module and add a single attribute `#[binding_wrapper]` like in the following example:
```rust
//
// File: src/ffi/mod.rs
//

use ffi_macro::binding_wrapper;
use std::sync::{Arc, Mutex};

// Define Error type and `()` type.
type ErrorType = String;
type VoidType = ();

pub trait SomeTrait: std::fmt::Debug {
    fn some_trait_method(&self);
}

#[derive(Clone, Debug)]
pub struct Foo(u32);
impl SomeTrait for Foo {
    fn some_trait_method(&self) {
    }
}

#[derive(Clone, Debug)]
pub struct CustomType(u32);
impl CustomType {
    pub fn return_result_with_dynamic_type(&self) -> Result<Arc<Mutex<dyn SomeTrait>>, ErrorType> {
        Ok(Arc::new(Mutex::new(Foo(10u32))))
    }
    pub fn return_another_custom_type(&self) -> AnotherCustomType {
        AnotherCustomType(20u64)
    }
}

#[derive(Clone, Debug)]
pub struct AnotherCustomType(u64);
impl AnotherCustomType {
    pub fn take_primitive_type_and_return_primitive_type(&self, a: u32) -> String {
        "Result".to_owned()
    }
}

#[binding_wrapper]
mod ffi {
    use super::SomeTrait;
    extern "Rust" {
        type CustomType;
        fn return_result_with_dynamic_type(self: &CustomType) -> Result<Arc<Mutex<dyn SomeTrait>>>;
        fn return_another_custom_type(self: &CustomType) -> AnotherCustomType;

        type AnotherCustomType;
        fn take_primitive_type_and_return_primitive_type(self: &AnotherCustomType, a: u32) -> String;        
        
        fn some_trait_method(self: &Arc<Mutex<dyn SomeTrait>>);
        type ErrorType;
    }
}
```

#### 3. Run `cargo build` in order to generate the glue code and build the static library. The generated code should be visible in two directories: `./_generated_swift/` and `./_generated_cpp/`. They contain a code that can be used directly in C++ and Swift applications. It's important to import a static library during the compilation process in both languages.


#### 4. Run SWIG in order to generate glue code for other languages:
 - `swig -java -c++ -outdir _generated_java wildland.i`
 - `swig -csharp -c++ -outdir _generated_csharp wildland.i`
 - `swig -python -c++ -outdir _generated_python wildland.i`


## Supported types
- `CustomUserType` --> `CustomUserType` - the structure members are not visible beyond the FFI layer - they can be reached indirectly using object's methods.
- `Vec<T>` --> `RustVec<T>` (C++ and Swift) --> `VecT` (other languages)
- `String` --> `String` (C++) --> `RustString` (other languages)
- `Option<T>` --> `OptionT`
- `Result<T, ErrorType>` --> `ResultT`
- `Arc<Mutex<T>>` --> `SharedMutexT`
- `Arc<Mutex<dyn T>>` --> `SharedMutexT`
- `u8, i8, u16, i16, ...` --> `unsigned char, char, unsigned short ...` (C++)
