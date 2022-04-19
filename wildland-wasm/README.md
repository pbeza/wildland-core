# WebAssembly interface for Wildland core


## 1. Setup
1. Install a tool needed to generate Wasm bindings - `wasm-pack`. The installation link: https://rustwasm.github.io/wasm-pack/installer/.
2. Run `wasm-pack build --target web` in `wildland-wasm/` directory.

Once the above steps are done, the `pkg` directory with JS bindings should be generated.

## 2. Dummy test
One way to run a simple HTTP server is to use python command in the `test` directory: `python -m http.server 8080` (Python 3 required). Since now the server should be available in the browser.
