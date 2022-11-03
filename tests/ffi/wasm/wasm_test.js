var wildland = require("./wildland.js")


wildland().then((wlib) => {
    function rust_vec_u8_of_bytes_to_js_string(vec) {
        result = "";
        for (let i = 0; i < vec.size(); i++) {
            result += String.fromCharCode(parseInt(vec.at(i).unwrap()))
        }
        return result;
    }

    function js_string_to_rust_vec_u8(js_str) {
        var result = new wlib.RustVec_u8();
        for (var i = 0; i < js_str.length; i++) {
            result.push(js_str[i].charCodeAt(0));
        }
        return result
    }

    // Local Secure Storage native implementation
    var Lss = wlib.LocalSecureStorage.extend("LocalSecureStorage", {
        store: {},

        insert: function (key, val) {
            console.log("LSS insert");
            var result;
            var str_key = key.to_string(); // JS cannot compare Rust strings so conversion to native type is necessary
            if (str_key in this.store) {
                console.log("LSS insert: found");
                result = wlib.OptionalVecu8ResultWithLssError.from_ok(wlib.OptionalVecu8(this.store[str_key]))
            } else {
                console.log("LSS insert: not found");
                result = wlib.OptionalVecu8ResultWithLssError.from_ok(wlib.OptionalVecu8());
            }
            this.store[str_key] = rust_vec_u8_of_bytes_to_js_string(val);
            return result;
        },
        get: function (key) {
            console.log("LSS get");
            var str_key = key.to_string(); // JS cannot compare Rust strings so conversion to native type is necessary
            if (str_key in this.store) {
                console.log("LSS get: found");
                return wlib.OptionalVecu8ResultWithLssError(wlib.OptionalVecu8(store[str_key]))
            } else {
                console.log("LSS get: not found");
                return wlib.OptionalVecu8ResultWithLssError(wlib.OptionalVecu8());
            }
        },
        contains_key: function (key) {
            console.log("LSS contains_key");
            var str_key = key.to_string(); // JS cannot compare Rust strings so conversion to native type is necessary
            return str_key in this.store;
        },
        keys: function () {
            console.log("LSS keys");
            console.log("unimplemented!");
        },
        keys_starting_with: function (prefix) {
            console.log("LSS keys");
            console.log("unimplemented!");
        },
        remove: function (key) {
            console.log("LSS remove");
            console.log("unimplemented!");
        },
        len: function () {
            console.log("LSS len");
            return this.store.size;
        },
        is_empty: function () {
            console.log("LSS is_empty");
            console.log("unimplemented!");
        },
    });

    // Configuration may be provided by an Object - the CargoCfgProvider implementation,
    // which has to be translated with collect_config function into Rust object,
    // or by parsing JSON string with parse_config function
    var CargoCfgProvider = wlib.CargoCfgProvider.extend("CargoCfgProvider", {
        // config: logger general
        get_use_logger: function () { return false; },
        get_log_level: function () { return new wlib.String("debug"); },
        get_log_use_ansi: function () { return false; },

        // config: logger file log
        get_log_file_enabled: function () { return false; },
        get_log_file_path: function () { return new wlib.OptionalRustString(); },
        get_log_file_rotate_directory: function () { return new wlib.OptionalRustString(); },

        // config: logger oslog
        get_oslog_category: function () { return new wlib.OptionalRustString(); },
        get_oslog_subsystem: function () { return new wlib.OptionalRustString(); },

        get_foundation_cloud_env_mode: function () { return wlib.FoundationCloudMode.Dev }
    });

    var lss = new Lss;
    var cfg = wlib.collect_config(new CargoCfgProvider);

    // Create CargoLib - main API handle
    var cargo_lib = wlib.create_cargo_lib(lss, cfg);

    // Acquire API object for user management
    var user_api = cargo_lib.user_api();

    var mnemonic = user_api.generate_mnemonic();
    // Rust Strings are not automatically converted to native ones yet
    // so to_string() call is required
    console.log(mnemonic.stringify().to_string());

    // mnemonic can be restored from vec of words
    // RustVec<T> can be initialized with constructor named with pattern `RustVec_{T type name}()`
    var words_vec = new wlib.RustVec_RustString();
    for (const word of [
        "enroll", "fat", "stumble", "life", "apology", "rate",
        "fringe", "mutual", "club", "slam", "ethics", "dinner"]) {
        words_vec.push(new wlib.String(word));
    }
    var restored_mnemonic = user_api.create_mnemonic_from_vec(words_vec);
    console.log(restored_mnemonic.stringify().to_string());

    var device_name = new wlib.String("WASM device");
    var new_user = user_api.create_user_from_mnemonic(mnemonic, device_name);
    console.log(new_user.stringify().to_string())
});
