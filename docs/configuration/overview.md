# Wildland Configuration

Wildland configuration can be handled from two side, either from the 
platform-agnostic client side or can be created as a rust structure.

## External Implementation

This describes the implementation that passes the configuration from the outside.
Object passed, has to implement a set of getters described by the
`CargoCfgProvider` trait in the [config.rs](../../crates/wildland-cargo-lib/src/api/config.rs) file.

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
