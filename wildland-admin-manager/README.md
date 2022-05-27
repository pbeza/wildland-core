# Wildland Admin Manager


## Bindings 
### Setup
In order to generate Java, C#, .NET and Swift bindings one needs to:
 * Install latest Swig
 * Install Java JDK
 * Install DotNet and Mono (for non-windows users)
 * Install Swift compiler

Once the mentioned SDKs and tools are installed the next step is to update two paths in `Makefile` - `JDK_INC_DIR` and `PYTHON_DIR`. If it runs on Windows it is necessary to update `CSHARP` related variables.

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

### Output
Once the given command is done, a `wildland-*` directory should be generated. It contains of a given target langugage glue code that is ready to use (see `ffi_example` and `test/ffi` to see examples).

### To be done
There is a plan to add docker containers to simplify usage of the bindings generator.
