#!/usr/bin/env bash
set -ex


/./emsdk/emsdk activate latest
. /emsdk/emsdk_env.sh

cp ./tests/ffi/wasm/main.cpp $PROJECT_DIR/ffi_build
cp ./tests/ffi/wasm/wasm_test.js $PROJECT_DIR/ffi_build

cd $PROJECT_DIR/ffi_build

export EMCC_CFLAGS="-s ERROR_ON_UNDEFINED_SYMBOLS=0 -sFETCH"

em++ ./main.cpp \
        -std=c++20 -g -D WASM \
        -s NO_DISABLE_EXCEPTION_CATCHING \
        -fexceptions \
        -L . \
        -I . \
        -l wildland_cargo_lib \
        -l embind \
        -g -s WASM=1 \
        -sMODULARIZE -sEXPORTED_RUNTIME_METHODS=ccall \
        -o wildland.js \
        --debug
node wasm_test.js
mkdir -p /out
mv wildland.js /out/
mv wildland.wasm /out/
