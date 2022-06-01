# Wildland Core(X)

This project creates a workspace for Wildland Core rust crates.

### Crates registry

The project is in an experimental phase, thus for the time being we use a [custom crates registry](https://doc.rust-lang.org/cargo/reference/registries.html#using-an-alternate-registry). It is abbreviated as `wl-dev` and used throughout the repositories in this project.

To use this registry, add the following lines to your `~/.cargo/config.toml` file

```toml
[registries]
wl-dev = { index = "https://crates.wildland.dev/git/index" }
```

### Using crates from a custom registry

In order to be able to access any of the crates published to the `wl-dev` registry, you must specify that in the dependencies of your project

```toml
wildland-corex = { version = "0.1.0", path = "../wildland-corex", registry = "wl-dev" }
```

### Publishing to the registry

This registry does not have a web frontend. In order to get the publish token, use the following endpoint: [https://crates.wildland.dev/me](https://crates.wildland.dev/me)

### Building with nightly toolchain

Some unstable Rust features are used in `wildland-admin-manager` crate so it requires building with nightly toolchain.

It can be set as a default toolchain with the command:

```bash
rustup default nightly
```

or set for individual build command:

```bash
cargo +nightly build
```