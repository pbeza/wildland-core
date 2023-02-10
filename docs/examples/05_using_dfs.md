# Use DFS filesystem-like API to interact with Container's data

When there are some Containers mounted in Wildland's context their data is accessible via
[`DfsFrontend`](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/ffi/trait.DfsFrontend.html)
(filesystem-like API) which can be obtained with
[`dfs_api`](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/api/cargo_lib/struct.CargoLib.html#method.dfs_api)
of `CargoLib` object.

```rust
let dfs = cargo_lib.dfs_api();
let mut dfs = dfs.lock().unwrap();
```

Exemplary API usage:

```rust
// File operations
let file = dfs.open("/some/claimed/path/file".to_owned()).unwrap();

dfs.write(&file, vec![1, 2, 3, 4, 5]).unwrap();
dfs.seek_from_start(&file, 1).unwrap();
let read_buf = dfs.read(&file, 3).unwrap();

dfs.close(&file).unwrap();
```