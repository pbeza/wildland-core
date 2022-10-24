var wildland = require("./wildland.js")

wildland().then((wlib) => {
    // Local Secure Storage native implementation
    var Lss = wlib.LocalSecureStorage.extend("LocalSecureStorage", {
        insert: function (key, val) { },
        get: function (key) { },
        contains_key: function (key) { },
        keys: function () { },
        remove: function (key) { },
        len: function () { },
        is_empty: function () { },
    });

    // Configuration may be provided by an Object - the CargoCfgProvider implementation,
    // which has to be translated with collect_config function into Rust object,
    // or by parsing JSON string with parse_config function
    var CargoCfgProvider = wlib.CargoCfgProvider.extend("CargoCfgProvider", {
        get_log_level: function () { return new wlib.String("debug"); },
        get_log_file: function () { return wlib.new_none_string() }
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
    console.log(mnemonic.get_string().to_string());

    // mnemonic can be restored from vec of words
    // RustVec<T> can be initialized with constructor named with pattern `RustVec_{T type name}()`
    var words_vec = new wlib.RustVec_RustString();
    for (const word of [
        "enroll", "fat", "stumble", "life", "apology", "rate",
        "fringe", "mutual", "club", "slam", "ethics", "dinner"]) {
        words_vec.push(new wlib.String(word));
    }
    var restored_mnemonic = user_api.create_mnemonic_from_vec(words_vec);
    console.log(restored_mnemonic.get_string().to_string());

    var new_user = user_api.create_user_from_mnemonic(mnemonic, device_name);
});