# Create and mount Container using Storage Template

Some technical details about Containers can be found [here](../architecture/forests_and_containers.md).

Container's API is available [here](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/ffi/struct.Container.html).

CargoLib exposes `create_container` via `CargoUser` method - creating container outside of a user context
is not considered as valid.

```rust
let container = user
    .create_container(
        "CONTAINER_1".to_owned(), // name/title
        &lfs_template, // previously deserialized LFS template
        "/some/claimed/path/".to_owned(), // container's primary path
    )
    .unwrap();
```

To access data contained in Container's storage it must be mounted:

```rust
user.mount(&container).unwrap();
```

Similarly, it can be unmounted:

```rust
user.unmount(&container).unwrap();
```