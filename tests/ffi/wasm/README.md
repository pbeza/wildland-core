# Wildland WASM usage example

This directory consists of files needed to compile **CargoLib** and run it with nodejs.

## Compile

**Requirements**:
- rustc compiler (min. version 1.63)
- cargo with `wasm32-unknown-emscripten` target installed

    Install with command:
    ```
    rustup target add wasm32-unknown-emscripten
    ```
- available emscripten compiler (`em++`)

    Follow the instructions https://emscripten.org/docs/getting_started/downloads.html#installation-instructions-using-the-emsdk-recommended

    or install on **ubuntu** with command:
    ```
    sudo apt install emscripten
    ```

Run `./build_wasm.sh` command in directory of this *README.md* file. This script will compile **CargoLib** to `wasm32-unknown-emscripten` target. Then compiled library along with generated C++ glue code and handwritten *main.cpp* (includes dependencies) file are used as an input for emscripten in order to generate wasm package and JS glue code. The result is two files: `wildland.wasm` and `wildland.js`.

## Example usage of wasm package

Example usage of wasm package is shown in the [`wasm_test.js`](./wasm_test.js) file which can be run with command `node wasm_test.js`.

Technical docs of Rust CargoLib content can be found [here](https://docs.wildland.dev/lld/doc/wildland_cargo_lib/index.html).

Expected sample output:

```bash
2022-10-14T10:28:04.000000Z DEBUG created new instance
2022-10-14T10:28:04.000000Z DEBUG generate_mnemonic:generate_random_mnemonic: return=Ok(["shrimp", "sniff", "gown", "match", "share", "figure", "plate", "video", "inside", "olive", "chunk", "fault"])
2022-10-14T10:28:04.000000Z DEBUG generate_mnemonic:from{mnemonic=["shrimp", "sniff", "gown", "match", "share", "figure", "plate", "video", "inside", "olive", "chunk", "fault"]}: return=MnemonicPayload(["shrimp", "sniff", "gown", "match", "share", "figure", "plate", "video", "inside", "olive", "chunk", "fault"])
2022-10-14T10:28:04.000000Z DEBUG generate_mnemonic: return=Ok(MnemonicPayload(["shrimp", "sniff", "gown", "match", "share", "figure", "plate", "video", "inside", "olive", "chunk", "fault"]))
2022-10-14T10:28:04.000000Z DEBUG stringify: return="shrimp sniff gown match share figure plate video inside olive chunk fault"
shrimp sniff gown match share figure plate video inside olive chunk fault
```