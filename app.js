import * as wasm from './wasm_test/wildland_cargo_lib.js'

const cfg_provider = {
    get_use_logger: () => false,
    "log_level": "debug",
    get_log_file_path: () => "path" // null means None
}

var store = {};

const lss = {

    insert: (key, value) => {
        var result = null;
        if (key in store) {
            result = store[key];
        }
        store[key] = value;
        return result;
    },
    get: (key) => {
        if (key in store) {
            return store[key]
        } else {
            return null
        }
    },
    contains_key: (key) => {
        return key in store
    },
    keys: () => object.keys(store),
    keys_starting_with: () => console.log("TODO"),
    remove: (key) => null,
    len: () => Object.keys(store).length,
    is_empty: () => Object.keys(store).length == 0,
}

console.log("Hello")
var cfg = wasm.collect_cfg(cfg_provider);

var cargo_lib = wasm.create_cargo_lib_wasm(cfg, lss);
var user_api = cargo_lib.user_api();
var mnemonic = user_api.generate_mnemonic();
console.log(mnemonic.stringify())
try {
    var user = user_api.create_user_from_mnemonic(mnemonic, "device");
} catch (e) {
    console.log(e)
}
