# Wildland Admin Manager
TODO

## Bindings
Wildland Admin Manager support bindings for the following languages:
 * Java
 * C++
 * C#
 * Python
 * Swift
 * WebAssembly (there is a different repository SDK for this purpose)


### Setup
In order to generate Java, C#, .NET and Swift bindings one needs to:
 * Install latest Swig
 * Install C++ compiler
 * Install Java JDK
 * Install DotNet and Mono (for non-windows users)
 * Install Swift compiler

Once the mentioned SDKs and tools are installed the next step is to update two paths in `Makefile` - `JDK_INC_DIR` and `PYTHON_DIR`. If it runs on Windows it is necessary to update `CSHARP` related variables as well.


Note: In the future the setup problem will be resolved by using docker images.


### Run
Use `make` in order to generate bindings and run tests for them. The following commands are supported:
 * `make java`
 * `make java_test`
 * `make csharp`
 * `make csharp_test`
 * `make python`
 * `make python_test`
 * `make swift`
 * `make swift_test`
 * `make cpp`
 * `make cpp_test`


### Output
Once the given command is done, a `wildland-*` directory should be generated. It consists of a given target langugage glue code that is ready to be used (see `test/ffi` for example).


### Contributing
See examples of binding usage in `test/ffi`. Keep in mind that adding types with templates like `Vec<T>` and `Box<T>` in cxx bridge needs to be followed by adding templates declaration in `wildland.i` file at the bottom.
