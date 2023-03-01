# Retrieving Containers

Containers can be retrieved from a user's context with the [find_containers](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/api/cargo_user/struct.CargoUser.html#method.find_containers) method.

`find_containers` expects 2 arguments:

- `filter`: an optional (passing `None` means no filtering) filter that is passed to CatLib, so a database query could be optimized.

    The `filter` must be one of the following variants:

    - HasExactPath(path_string),
    - HasPathStartingWith(path_string),
    - Or(filter, filter),
    - And(filter, filter),
    - Not(filter),

    A filter may be a kind of match operation, checking the container's properties (`HasExactPath`, `HasPathStartingWith`),
    or a logical combination of filters (`Or`, `And`, `Not`).

    Every variant has its own static method for instantiation.

- `mount_state` - specifies whether to include mounted, unmounted or all containers in the result.

## Rust exemplary usage

```rust
// Find all unmounted containers
let containers = cargo_user
    .find_containers(
        None,
        MountState::Unmounted,
    )
    .unwrap();
```

```rust
// Find all containers that have either a path "/some/path" or some path starting with "/some/other/".
// Does not matter whether containers are mounted or not.
let containers = cargo_user
    .find_containers(
        Some(CargoContainerFilter::or(
            CargoContainerFilter::has_exact_path("/some/path".into()),
            CargoContainerFilter::has_path_starting_with("/some/other/".into()),
        )),
        MountState::MountedOrUnmounted,
    )
    .unwrap();
```

```rust
// Find all mounted containers that don't have a path "/some/path"
let containers = cargo_user
    .find_containers(
        Some(CargoContainerFilter::not(
            CargoContainerFilter::has_exact_path("/some/path".into()),
        )),
        MountState::Mounted,
    )
    .unwrap();
```

## Swift exemplary usage

```swift
// Find all unmounted containers
let containers = try user.findContainers(
    Optional.none.toRustOptional(),
    MountState_Unmounted)
```

```rust
// Find all containers that have either a path "/some/path" or some path starting with "/some/other/".
// Does not matter whether containers are mounted or not.
let containers = try user.findContainers(
    Optional.some(
        orFilter(
            hasExactPath(RustString("/some/path")),
            hasPathStartingWith(RustString("/some/other/"))
        )
    ).toRustOptional(),
    MountState_MountedOrUnmounted)
```

```rust
// Find all mounted containers that don't have a path "/some/path"
let containers = try user.findContainers(
    Optional.some(
        notFilter(
            hasExactPath(RustString("/some/path")),
        )
    ).toRustOptional(),
    MountState_Mounted)
```
