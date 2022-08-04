#!/usr/bin/env bash
set -ex

cd $PROJECT_DIR
export FFI_BUILD_DIR=$PROJECT_DIR/ffi_build
export TARGET_DIR=$PROJECT_DIR/target/$BUILD_TARGET
export STATIC_LIB=$TARGET_DIR/debug/libwildland_cargo_lib.a

export CARGO_REGISTRIES_WL_DEV_INDEX="https://crates.wildland.dev/git/index"
export SWIFT_BRIDGE_OUT_DIR="$PROJECT_DIR/crates/wildland-cargo-lib/_generated_swift"

mkdir -p "$SWIFT_BRIDGE_OUT_DIR"

rustup target add ${BUILD_TARGET}

/./emsdk/emsdk activate latest
. /emsdk/emsdk_env.sh

cargo clean \
    && EMCC_CFLAGS="-s ERROR_ON_UNDEFINED_SYMBOLS=0" cargo build \
    --features "bindings" \
    --target ${BUILD_TARGET}

mkdir -p "${FFI_BUILD_DIR}"
cp \
    "${SWIFT_BRIDGE_OUT_DIR}/SwiftBridgeCore.h" \
    "$PROJECT_DIR/crates/wildland-cargo-lib/_generated_cpp/ffi_cxx.h" \
    "$PROJECT_DIR/crates/wildland-cargo-lib/_generated_cpp/ffi_swig.i" \
    "${SWIFT_BRIDGE_OUT_DIR}/ffi_swift/ffi_swift.h" \
    "${STATIC_LIB}" \
    "${FFI_BUILD_DIR}"
