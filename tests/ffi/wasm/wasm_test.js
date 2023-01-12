const puppeteer = require("puppeteer");

(async () => {
  const browser = await puppeteer.launch({
    headless: true,
    args: ["--no-sandbox"]
  });
  const page = await browser.newPage();

  page.on("console", (consoleObj) => console.log(consoleObj.text()));

  await page.goto("http://localhost:9200");

  await page.evaluate(() =>
    window.Module().then((wlib) => {
      // Local Secure Storage native implementation
      var Lss = wlib.LocalSecureStorage.extend("LocalSecureStorage", {
        store: {},

        insert(key, val) {
          console.log("LSS insert");
          var result;
          var str_key = key.to_string(); // JS cannot compare Rust strings so conversion to native type is necessary
          if (str_key in this.store) {
            console.log("LSS insert: found");
            result = wlib.OptionalRustStringResultWithLssError.from_ok(
              new wlib.OptionalRustString(this.store[str_key])
            );
          } else {
            console.log("LSS insert: not found");
            result = wlib.OptionalRustStringResultWithLssError.from_ok(
              new wlib.OptionalRustString()
            );
          }
          this.store[str_key] = val;
          return result;
        },
        get(key) {
          console.log("LSS get");
          var str_key = key.to_string(); // JS cannot compare Rust strings so conversion to native type is necessary
          if (str_key in this.store) {
            console.log("LSS get: found");
            return wlib.OptionalRustStringResultWithLssError.from_ok(
              new wlib.OptionalRustString(store[str_key])
            );
          } else {
            console.log("LSS get: not found");
            return wlib.OptionalRustStringResultWithLssError.from_ok(
              new wlib.OptionalRustString()
            );
          }
        },
        contains_key(key) {
          console.log("LSS contains_key");
          var str_key = key.to_string(); // JS cannot compare Rust strings so conversion to native type is necessary
          return str_key in this.store;
        },
        keys() {
          console.log("LSS keys");
          console.log("unimplemented!");
        },
        keys_starting_with(prefix) {
          console.log("LSS keys");
          console.log("unimplemented!");
        },
        remove(key) {
          console.log("LSS remove");
          console.log("unimplemented!");
        },
        len() {
          console.log("LSS len");
          return this.store.size;
        },
        is_empty() {
          console.log("LSS is_empty");
          console.log("unimplemented!");
        }
      });

      // Configuration may be provided by an Object - the CargoCfgProvider implementation,
      // which has to be translated with collect_config function into Rust object,
      // or by parsing JSON string with parse_config function
      var CargoCfgProvider = wlib.CargoCfgProvider.extend("CargoCfgProvider", {
        // config: logger general
        get_use_logger() {
          return true;
        },
        get_log_level() {
          return new wlib.String("trace");
        },
        get_log_use_ansi() {
          return false;
        },

        // config: logger file log
        get_log_file_enabled() {
          return false;
        },
        get_log_file_path() {
          return new wlib.OptionalRustString();
        },
        get_log_file_rotate_directory() {
          return new wlib.OptionalRustString();
        },

        get_foundation_cloud_env_mode() {
          return wlib.FoundationCloudMode.Dev;
        }
      });

      var lss = new Lss();
      var cfg = wlib.collect_config(new CargoCfgProvider());

      // Create CargoLib - main API handle
      var cargo_lib = wlib.create_cargo_lib(lss, cfg);

      // Acquire API object for user management
      var user_api = cargo_lib.user_api();

      var mnemonic = user_api.generate_mnemonic();
      // Rust Strings are not automatically converted to native ones yet
      // so to_string() call is required
      console.log(mnemonic.stringify().to_string());

      // // mnemonic can be restored from vec of words
      // // RustVec<T> can be initialized with constructor named with pattern Vec{T type name}()
      var words_vec = new wlib.VecRustString();
      [
        "enroll",
        "fat",
        "stumble",
        "life",
        "apology",
        "rate",
        "fringe",
        "mutual",
        "club",
        "slam",
        "ethics",
        "dinner"
      ].forEach((word) => words_vec.push(new wlib.String(word)));

      var restored_mnemonic = user_api.create_mnemonic_from_vec(words_vec);
      console.log(restored_mnemonic.stringify().to_string());

      var device_name = new wlib.String("WASM device");
      var new_user = user_api.create_user_from_mnemonic(mnemonic, device_name);
      console.log(new_user.stringify().to_string());
    })
  );

  await browser.close();
})();
