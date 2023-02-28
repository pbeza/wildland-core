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
use wildland_catlib::RedisCatLib;
use wildland_corex::interface::CatLib;
use std::collections::{HashSet, HashMap};
use wildland_corex::entities::Identity;
use wildland_corex::StorageTemplate;
use wildland_corex::Forest;
let catlib = RedisCatLib::default();
let forest = catlib.create_forest(
                 Identity([1; 32]),
                 HashSet::from([Identity([2; 32])]),
                 vec![],
             ).unwrap();
let forest = Forest::new(forest);
let storage_template = StorageTemplate::try_new(
    "FoundationStorage",
    &HashMap::from([
            (
                "field1".to_owned(),
                "Some value with container name: {{ CONTAINER_NAME }}".to_owned(),
            ),
            (
                "parameter in key: {{ OWNER }}".to_owned(),
                "enum: {{ ACCESS_MODE }}".to_owned(),
            ),
            ("uuid".to_owned(), "{{ CONTAINER_UUID }}".to_owned()),
            ("paths".to_owned(), "{{ PATHS }}".to_owned()),
        ]),
    )
    .unwrap();
let path = "/some/path".into();
let container = forest.create_container("container name2".to_owned(), &storage_template, path).unwrap();
container.add_path("/bar/baz2".into()).unwrap();
```

### Finding container(s) by paths

```rust
let catlib = RedisCatLib::default();
let containers = catlib.find_containers(vec!["/foo/bar".into()], false).unwrap();
```
