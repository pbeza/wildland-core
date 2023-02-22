# Init CargoLib (create CargoLib object)

CargoLib is an object aggregating and exposing the public API of the library.
All functionalities are exposed towards a native application side through this object but not necessarily directly.

It can be created with `create_cargo_lib` global function.

As mentioned above, CargoLib does not try to expose all functionalities directly by its methods,
but it can be treated as a starting point for using wildland core in a native app.
To avoid programming invalid logic on the native app side, some functionalities are
hidden in subsequent objects that can be obtained from CargoLib.

Usage of **Foundation Storage API** makes sense only within a user's context, so to avoid
calling its methods before a user is created/retrieved access to **Foundation Storage API** is
enclosed within `CargoUser` object.

Filesystem-like interface for interacting with containers' data can be retrieved with the `CargoLib::dfs_api` method.

## 1. CargoConfig

One of the two required arguments of `create_cargo_lib` function is `CargoConfig` structure. CargoLib does not try
to read configuration parameters from any source like files, envs, etc. CargoLib leaves loading configuration to
a native app and requires valid values, regardless of their source.

More information about `CargoConfig` can be found [here](../configuration/config.md).

`CargoConfig` can have platform-specific parameters, e.g. on apple platforms it requires parameters
for OSlog configuration.

Example of parsing `CargoLib` in Rust. In other languages, it can be achieved with
[`parse_config` global function](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/api/config/fn.parse_config.html).

```rust
let config_str = r#"{
        "log_level": "trace",
        "log_use_ansi": false,
        "log_file_enabled": true,
        "log_file_path": "cargo_lib_log",
        "log_file_rotate_directory": ".",
        "evs_url": "some_url",
        "sc_url": "some_url"
    }"#;
let cfg: CargoConfig = serde_json::from_str(config_str).unwrap();
```

## 2. LSS (Local Secure Storage)

More information about LSS can be found [here](../architecture/lss.md).

LSS must be provided to `create_cargo_lib` function as a static reference to some object implementing
[`LocalSecureStorage`](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/ffi/trait.LocalSecureStorage.html) trait.

**IMPORTANT!** CargoLib expects a static reference to the LSS object, meaning it cannot be freed while CargoLib is in use.

The following code shows exemplary LSS implementation based on `HashMap`, which is not valid in terms of Wildland
assumptions because it is not persistent.

```rust
#[derive(Default)]
struct HashMapLss {
    storage: RefCell<HashMap<String, String>>,
}

impl LocalSecureStorage for HashMapLss {
    fn insert(&self, key: String, value: String) -> LssResult<Option<String>> {
        Ok(self.storage.borrow_mut().insert(key, value))
    }

    fn get(&self, key: String) -> LssResult<Option<String>> {
        Ok(self.storage.try_borrow().unwrap().get(&key).cloned())
    }

    fn contains_key(&self, key: String) -> LssResult<bool> {
        Ok(self.storage.borrow().contains_key(&key))
    }

    fn keys(&self) -> LssResult<Vec<String>> {
        Ok(self.storage.borrow().keys().cloned().collect())
    }

    fn keys_starting_with(&self, prefix: String) -> LssResult<Vec<String>> {
        Ok(self
            .storage
            .borrow()
            .keys()
            .filter(|key| key.starts_with(&prefix))
            .cloned()
            .collect())
    }

    fn remove(&self, key: String) -> LssResult<Option<String>> {
        Ok(self.storage.borrow_mut().remove(&key))
    }

    fn len(&self) -> LssResult<usize> {
        Ok(self.storage.borrow().len())
    }

    fn is_empty(&self) -> LssResult<bool> {
        Ok(self.storage.borrow().is_empty())
    }
}

let lss: &'static dyn LocalSecureStorage = Box::leak(Box::<LssStub>::default());
```

## 3. Init CargoLib

Having `CargoConfig` and `LocalSecureStorage` initialized, we can now create `CargoLib` instance.

**NOTE** Subsequent calls of `create_cargo_lib` should return a reference to the same object.

```rust
let cargo_lib = create_cargo_lib(lss_stub, cfg);
// cargo_lib is in fact a mutex so it must be locked to use it
let cargo_lib = cargo_lib.lock().unwrap();
```