#!/usr/bin/env bash
set -ex

cd $PROJECT_DIR
export FFI_BUILD_DIR=$PROJECT_DIR/ffi_build
export TARGET_DIR=$PROJECT_DIR/target/$BUILD_TARGET
export STATIC_LIB=$TARGET_DIR/debug/libwildland_cargo_lib.a

export CARGO_REGISTRIES_WL_DEV_INDEX="https://crates.wildland.dev/git/index"

rustup target add ${BUILD_TARGET}

/./emsdk/emsdk activate latest
. /emsdk/emsdk_env.sh

cargo clean \
    && EMCC_CFLAGS="-s ERROR_ON_UNDEFINED_SYMBOLS=0 -sFETCH" cargo build \
    --features "bindings" \
    --target ${BUILD_TARGET}

mkdir -p "${FFI_BUILD_DIR}"
cp \
    "$PROJECT_DIR/crates/wildland-cargo-lib/_generated_ffi_code/ffi_cxx.h" \
    "$PROJECT_DIR/crates/wildland-cargo-lib/_generated_ffi_code/ffi_swig.i" \
    "${STATIC_LIB}" \
    "${FFI_BUILD_DIR}"
