cargo build \
    --target "wasm32-unknown-unknown" \
    --package wildland-cargo-lib
wasm-bindgen \
    --target nodejs \
    target/wasm32-unknown-unknown/debug/wildland_cargo_lib.wasm \
    --out-dir ./wasm_test
node \
    --experimental-modules \
    --experimental-wasm-modules \
    app.mjs