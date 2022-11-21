# Wildland Core(X)

This project creates a workspace for Wildland Core rust crates.
Please mind, that this project is currently in early development phase!

Official website:

<https://wildland.io/>

## Full Documentation

where definitions are as follows:
HLD - High Level Documentation - concepts, introductions, and abstractions
MLD - Middle Level Documentation - resources that can be useful for development
LLD - Low Level Documentation - code documentation and api, autogenerated from code

Current renders of the documentation can be found here:

HLD link: <https://docs.wildland.dev/>
MLD link: <https://docs.wildland.dev/docs/wildland/mld/index.html>
LLD link: <https://docs.wildland.dev/docs/wildland/lld/doc/wildland_cargo_lib/index.html>

## Local Documentation

to build the documentation locally, cargo tool and some dependencies are required.

* install dependencies:
`cargo install mdbook mdbook-mermaid mdbook-plantuml mdbook-toc mdbook-linkcheck mdbook-graphviz mdbook-katex`
* setup dependency config
`mdbook-mermaid install .`
* generate and open the documentation
`mdbook build --open`

## Logging

Logging works over tracing::subscriber and is only configurable via client
application. Please refer to the logging and configuration documentation.
Environment variables will not work.

### WASM

WASM example project can be found in [this directory](./tests/ffi/wasm/) and it's readme file is located [here](./tests/ffi/wasm/README.md).
