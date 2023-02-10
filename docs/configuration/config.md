# Wildland Configuration

Wildland configuration can be handled from two side, either from the 
platform-agnostic client side or can be created as a rust structure.

## External Implementation

This describes the implementation that passes the configuration from the outside.
Object passed, has to implement a set of getters described by the
[`CargoCfgProvider`](https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/api/config/trait.CargoCfgProvider.html) trait in the [config.rs](../../crates/wildland-cargo-lib/src/api/config.rs) file.

This basic set of getters will allow the configuration to be translated into
internal structure after being received.

## Internal Implementation

If the library is used as a rust library (not via the bindings), in addition to
to the first method, the configuration can be created as internal data structure
and it will also be accepted and acted upon. `LoggerConfig` structure can be
either created by using `::new()` or `::default()` (with defaults of course), or
full constructor can be used, i.e.

```rust
  config = LoggerConfig {
      use_logger: true,
      log_level: Level::TRACE,
      //... etc
  }
```

The same file does contain rust code examples in tests if required.

## Catlib backend

Running Wildland Core requires Redis server to be accessible from local or remote network. The
server must be accessible at all times even if one does not interact with Catalog backend.

The default connection string is `redis://127.0.0.1:6379/0` where `/0` denotes database with id `0`.
The default connection string can be overriden by specifying it in a `CARGO_REDIS_URL` environment
variable.

_note: this solution is temporary and will be shortly replaced by a cargo configuration entry_
