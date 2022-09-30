export APP_DIR=$CI_PROJECT_DIR
export BUILD_TARGET=x86_64-unknown-linux-gnu
export FFI_BUILD_DIR_OUT=$APP_DIR/ffi_build
export FFI_TESTS_DIR_OUT=$APP_DIR/ffi_tests
export TARGET_DIR=$APP_DIR/target/${BUILD_TARGET}
export CXX_LIB=$TARGET_DIR/debug/libwildland_cargo_lib.a
export SWIFT_BRIDGE_OUT_DIR=$TARGET_DIR
export CARGO_LIB_DIR=$APP_DIR/crates/wildland-cargo-lib

mkdir -p ${FFI_BUILD_DIR_OUT}

cargo build \
    --package wildland-cargo-lib \
    --features bindings \
    --target ${BUILD_TARGET}

cp \
    "${CARGO_LIB_DIR}/_generated_ffi_code/ffi_cxx.h" \
    "${CARGO_LIB_DIR}/_generated_ffi_code/ffi_swift.h" \
    "${CARGO_LIB_DIR}/_generated_ffi_code/ffi_swift.swift" \
    "${CARGO_LIB_DIR}/_generated_ffi_code/ffi_swig.i" \
    "${CXX_LIB}" \
    "${FFI_BUILD_DIR_OUT}"

cp -r tests/ffi/ ${FFI_TESTS_DIR_OUT}
