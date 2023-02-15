# Examples

This chapter contains simple use cases of CargoLib with a step-by-step explanation of its features and capabilities.

Please, remember that examples in this chapter are written in Rust and they might slightly differ from usage in
native applications languages like Swift, C#, etc. However, the same functionalities should be possible to achieve.
An example difference might be that in Rust some functionality is exposed as a Static method whereas e.g. in other languages
it might be exposed as a global function with a slightly different name.

- [Init CargoLib (create CargoLib object)](./01_creating_cargo_lib.md)
- [Create user](./02_creating_user.md)
- [Create Storage Template for development purposes](./03_creating_lfs_storage_template.md)
- [Create and mount Container using Storage Template](./04_create_and_mount_container.md)
- [Use DFS filesystem-like API to interact with Container's data](./05_using_dfs.md)

## Error propagation

Whenever you see `unwrap()` in code it means that the function/method returns a result type which is
translated by the FFI layer into an Exception in other languages (in fact this behavior depends on the target language).