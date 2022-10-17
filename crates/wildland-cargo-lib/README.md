# Wildland Cargo Lib

High level interface for the Cargo clients. It is built on top of the Wildland CoreX library and provides Cargo specific abstractions like "user", "device" or "sharing logic".

## Bindings
Wildland Cargo Lib support bindings for the following languages:
 * Java
 * C++
 * C#
 * Python
 * Swift
 * WebAssembly (there is another SDK repository for this purpose)


### Setup
Use docker images in order to generate the bindings glue code for `Java`, `C#` and `Python`. One can find them in `./docker` directory. For `C++` and `Swift` bindings one can simply run `cargo build --features bindings` and get the glue code from `./_generated_ffi_code` and `./_generated_swift/` directories. **It requires rust toolchain in version `>1.59.0`**.
