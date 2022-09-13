# Wildland Cargo Lib
TODO:WILX-207


## Bindings
Wildland Cargo Lib support bindings for the following languages:
 * Java
 * C++
 * C#
 * Python
 * Swift
 * WebAssembly (there is another SDK repository for this purpose)


### Setup
Use docker images in order to generate the bindings glue code for `Java`, `C#` and `Python`. One can find them in `./docker` directory. For `C++` and `Swift` bindings one can simply run `cargo build --features bindings` and get the glue code from `./_generated_cpp` and `./_generated_swift/` directories. **It requires rust toolchain in version `>1.59.0`**.
