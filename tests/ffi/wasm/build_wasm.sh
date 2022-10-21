export EMCC_CFLAGS="-s ERROR_ON_UNDEFINED_SYMBOLS=0" 
cargo build --features "bindings" --target wasm32-unknown-emscripten
em++ ./main.cpp -std=c++20 -g -D WASM \
    -s NO_DISABLE_EXCEPTION_CATCHING \
    -fexceptions \
    -L ../../../target/wasm32-unknown-emscripten/debug/ \
    -I ../../../crates/wildland-cargo-lib/_generated_ffi_code \
    -l wildland_cargo_lib \
    -l embind \
    -s WASM=1 \
    -sMODULARIZE -sEXPORTED_RUNTIME_METHODS=ccall \
    -o wildland.js