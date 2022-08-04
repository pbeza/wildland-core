#!/usr/bin/env bash
set -ex


/./emsdk/emsdk activate latest
. /emsdk/emsdk_env.sh
cd $PROJECT_DIR/ffi_build
mv ffi_cxx.h wildland.cpp
em++ wildland.cpp \
        -std=c++14 -g -D WASM \
        -s NO_DISABLE_EXCEPTION_CATCHING \
        -fexceptions \
        -L . \
        -I . \
        -l wildland_cargo_lib \
        -l embind \
        -g -s WASM=1 \
        -s EXPORT_ALL=1 \
        -o wildland.js \
        --debug
mkdir -p /out
mv wildland.js /out/
mv wildland.wasm /out/
