# Wildland Catlib

This crate provides Wildland Catalog client which allows to persistently store Wildland entities
such as Containers, Storages, Forests in an arbitrary database.

Current implementation stores all entities in an inefficient, single-file, schemaless "database".

Location of the database file depends on the platform where the application runs, these are:

- `Linux: /home/alice/.config/catlib`
- `Windows: C:\Users\Alice\AppData\Roaming\com.wildland.Cargo\catlib`
- `macOS: /Users/Alice/Library/Application Support/com.wildland.Cargo/catlib`

## Example usages

### Creating container with paths

```rust
let catlib = CatLib::default();
let forest = catlib.create_forest(b"owner".to_vec(), Signers::new(), vec![]).unwrap();
let container = forest.create_container("container name".to_owned()).unwrap();
container.add_path("/foo/bar".to_string());
```

### Finding container(s) by paths

```rust
let catlib = CatLib::default();
let containers = catlib.find_containers(vec!["/foo/bar".into()], false).unwrap();
```
