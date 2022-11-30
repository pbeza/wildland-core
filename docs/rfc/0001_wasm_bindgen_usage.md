# Possibility of wasm_bindgen usage as a tool generating WASM bindings exposed from CargoLib

```
Title           : wasm_bindgen usage in CargoLib
Category        : Feature
Author(s)       : Szymon Bagiński <szymon@wildland.io>
Team            : Corex Team
Created         : 2022-11-28
Deadline        : 2022-12-05
Feature ID      : WILX-267
```

# Motivation

Many problems were encountered while generating WASM package exposing CargoLib functionalities with rusty-bind and emscripten. They may be possible to solve but the motivation was to check if that problems can be easily avoided by using wasm_bindgen. Those are:

- We could not use async runtimes like `tokio` or `async_std` (although tokio's single threaded event loop may be possible to run)
- We could not use crates that depends on `tokio` or `async_str`.
- The only http client that compiles on `wasm32-unknown-emscripten`, which is `minreq`, can not be compiled on that target with `https` feature turned on (what is rather understandable since in wasm environment we should rather use fetch api of a browser).
- It will be hardly possible to interact with JS event loop so we could expose some kind of asynchronous API (and use JS promises somehow)
- We intend to use (for some time at least) Redis as a catalog backend which by default accepts raw tcp connections. Raw TCP is not accessible on wasm target. We could use though some proxy like Webdis (exposes http API).

Workload estimate (for 1 developer, assuming that we have ready database exposing http or websocket api [not a file like now]):

- Add wasm_bindgen gluecode - 2 days
- Asynchronous version of corex (extract as much common sync code as possible ) - 5 days

# Impact Analysis

Experimental usage of wasm_bindgen for wildland-core can be found on branch `szymon/wasm-bindgen-spike`.

## Dependencies

`wasm_bindgen` requires compiling on `wasm32-unknown-unknown` target. In order to switch that target some dependency may need to be replaced or include some additional features.

### getrandom

This crate requires including `js` feature to compile.

### memmap

This dependency of the `rustbreak` crate which provides file backed database. For obvious reasons this crate won't be used in case of WASM platform where communication with production database should be performed with Websockets or Http.

## Code Injection from native app (WASM) to CargoLib

Used for instance for providing `LocalSecureStorage` implementation to CargoLib.

It is in detail described here: https://rustwasm.github.io/wasm-bindgen/reference/working-with-duck-typed-interfaces.html

JS LocalSecureStorage could be defined for example as the following:

```rust
#[wasm_bindgen]
extern "C" {
    pub type JsLss;

    #[wasm_bindgen(typescript_type = "Array<string>")]
    pub type StringArray;

    #[wasm_bindgen(structural, method, catch, js_name = insert)]
    pub fn js_insert(this: &JsLss, key: String, value: String) -> Result<Option<String>, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = get)]
    pub fn js_get(this: &JsLss, key: String) -> Result<Option<String>, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = contains_key)]
    pub fn js_contains_key(this: &JsLss, key: String) -> Result<bool, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = keys)]
    pub fn js_keys(this: &JsLss) -> Result<StringArray, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = keys_starting_with)]
    pub fn js_keys_starting_with(this: &JsLss, prefix: String) -> Result<StringArray, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = remove)]
    pub fn js_remove(this: &JsLss, key: String) -> Result<Option<String>, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = len)]
    pub fn js_len(this: &JsLss) -> Result<usize, LssError>;

    #[wasm_bindgen(structural, method, catch, js_name = is_empty)]
    pub fn js_is_empty(this: &JsLss) -> Result<bool, LssError>;
}

impl LocalSecureStorage for JsLss {
    fn insert(&self, key: String, value: String) -> wildland_corex::LssResult<Option<String>> {
        self.js_insert(key, value)
    }

    fn get(&self, key: String) -> wildland_corex::LssResult<Option<String>> {
        self.js_get(key)
    }

    fn contains_key(&self, key: String) -> wildland_corex::LssResult<bool> {
        self.js_contains_key(key)
    }

    fn keys(&self) -> wildland_corex::LssResult<Vec<String>> {
        let keys = self.js_keys()?;
        let keys: js_sys::Array = keys.unchecked_into();
        Ok(keys.iter().map(|x| x.as_string().unwrap()).collect())
    }

    fn keys_starting_with(&self, prefix: String) -> wildland_corex::LssResult<Vec<String>> {
        let keys = self.js_keys_starting_with(prefix)?;
        let keys: js_sys::Array = keys.unchecked_into();
        Ok(keys.iter().map(|x| x.as_string().unwrap()).collect())
    }

    fn remove(&self, key: String) -> wildland_corex::LssResult<Option<String>> {
        self.js_remove(key)
    }

    fn len(&self) -> wildland_corex::LssResult<usize> {
        self.js_len()
    }

    fn is_empty(&self) -> wildland_corex::LssResult<bool> {
        self.is_empty()
    }
}
```

It is worth to notice that methods can't use `Vec<String>` as a return type so we must introduce new opaque type (`StringArray`) which represents JS array and then convert it to `js_sys::Array` and further collect it as a `Vec<String>`. In the above code, if JS developer provides non-string value in that array, the Rust code would crash, so some additional check would be required.

`Option` type works intuitively, meaning `null` value is converted into `None`.

## Using Results

All `Result` types passed from JS to Rust (like in the previous section, `LssResult` has `LssError` defiend as its error type) must implement `From<wasm_bindgen::JsValue>`.

```Rust
impl From<JsValue> for LssError {
    fn from(json: JsValue) -> Self {
        LssError::Error(
            json.as_string()
                .unwrap_or("JS didn't provide an error message!".to_string()),
        )
    }
}
```

Similarly, result types returned from `Rust` to `JS` must implement `Into<wasm_bindgen::JsValue>`.

```Rust
impl Into<JsValue> for CargoLibCreationError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}
```

## Passing `LocalSecureStorage` reference

`create_cargo_lib` function on platforms handled by `rusty-bind` expect to get LSS as a `&'static dyn LocalSecureStorage` what is not allowed in wasm_bindgen.

> it is currently not sound to use lifetimes in function signatures

`create_cargo_lib` function would have to look differently on WASM, like the following:

```Rust
use crate::wasm::JsLss;
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn create_cargo_lib(
    cfg: CargoConfig,
    lss: JsLss,
) -> Result<CargoLib, CargoLibCreationError> {
    let lss = Box::leak(Box::new(lss));
    let lss = lss as &'static dyn LocalSecureStorage;
    let cargo_lib = CargoLib::new(lss, cfg.fsa_config);

    Ok(cargo_lib)
}
```

In the above code we can see how to cast `JsLSS` (which under the hood is only kind of reference to WASM JS stack [in fact this is an index: https://rustwasm.github.io/wasm-bindgen/reference/reference-types.html]) to `&'static dyn LocalSecureStorage`.

## A lot of `skip` directives

wasm_bindgen tries to expose all structures' fields or all methods placed within tha same `impl` block. It will require to use `skip directive` or to manage two `impl` block for a struct to differentiate things to be exposed from the ones that are not.

All fields that do not implement `Copy` trait must be skipped by wasm_bindgen design.

## Passing enums around

Only C-style enums are allowed by design

## Arc<Mutex<_>>

Passing some object to JS wrapped in a `Arc<Mutex<_>>` is not possible because those wrappers don't implement `IntoWasmAbi` trait.

## Using tracing::instrument

Skipping arguments in `#[tracing::instrument(skip(self))]` does not compile within `impl` blocks marked as `#[wasm_bindgen]`, so we must give up either tracing::instrument or skips, what would mean obligatory defining `std::fmt::Debug` for all arguments (including `self`).

## Receiving `Vec<String>`

As mentioned in [code injection section](#code-injection-from-native-app-wasm-to-cargolib) `Vec<String>` type is not supported by wasm_bindgen, so functions/methods that receive this type as an argument or a return type must be conditionally compiled for WASM and for other platforms.

```Rust
#[cfg(not(target_arch = "wasm32"))]
pub fn create_mnemonic_from_vec(
    &self,
    words: Vec<String>,
) -> Result<MnemonicPayload, CreateMnemonicError> {
    ...
}

#[cfg(target_arch = "wasm32")]
pub fn create_mnemonic_from_vec(
    &self,
    words: StringArray,
) -> Result<MnemonicPayload, CreateMnemonicError> {
    let words = self.js_keys_starting_with(words)?;
    let words: js_sys::Array = words.unchecked_into();
    let words: Vec<String> = words.iter().map(|x| x.as_string().unwrap()).collect();
    ...
}
```

## HTTP and Websockets

Http or Websocket connection can be easily established:

- http: https://rustwasm.github.io/wasm-bindgen/examples/fetch.html
- websocket: https://rustwasm.github.io/wasm-bindgen/examples/websockets.html

It requires browser environment to be loaded so projects like wasm-cli run in clear node environment may not have a lot of sense (it does not provide fetch and websocket api like the browsers do).

Unfortunately HTTP API returns `Promise` type which can be turned into `JsFuture` (implements Rust std Future trait). This raises some compatibility issues with our mechanisms of exposing bindings to other platforms than WASM. On those other platforms, our call stack is fully synchronous and bindings are also exposed as synchronous methods/functions. If we don't want to maintain two different code bases (async for WASM and sync for other) we could unify those call stacks (by stack I mean e.g. async http function in catlib, called by async corex function, called by async cargo_lib function) to be asynchronous somehow. Again unfortunately, `JsFuture` is not `Send`able and `Sync`able so we can't use async/await syntax what leaves us with returning explicit types like `Box<dyn Future<Output = String>>`. Although the problem in this case would be executing such a future. While on WASM we could just use JS executor which does not require futures to be `Send + Sync`, on other platforms using even single threaded runtime requires at least `Sync`.

Another problem is related with Websockets which API is based on callback so we would need also some unified (for WASM and others) way of passing ones from native language to Rust. Some solution can be always a kind of channel kept alive in other than main application thread.

To sum up, it is unlikely that we would manage to workout some common approach for designing API for all platforms. WASM probably should be treated as other library, which in the best case could reuse some small components from other platforms core.
