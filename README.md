# Wildland Core(X)


This project creates a workspace for Wildland Core rust crates.
Please mind, that this project is currently in early development phase!

Official website:

https://wildland.io/

## Full Documentation

Current renders of the documentation can be found here:
https://docs.wildland.dev/

where definitions are as follows:
HLD - High Level Documentation - concepts, introductions, and abstractions
MLD - Middle Level Documentation - resources that can be useful for development
LLD - Low Level Documentation - code documentation and api, autogenerated from code

## Logging

Logs are gathered automatically. Current log scrubber implementations:

- Console
- File

To initiate logging, following env must be enabled:
`RUST_LOG=XXX`
where XXX is one of `{ trace, debug, info, warn, error, off }`

By default, console logging is disabled.
However, it is highly encouraged to set `RUST_LOG=trace` or `RUST_LOG=debug`
permamently while working on the library.

File logger is non-optional for now and enabled by default.
In case user wishes to change its default name, can be steered by changing
"RUST_LOGFILE=xxx", but if the alternative value is not provided it will
default to `corex.log.<datetime>.`

In addition, file-based logs will rotate each hour by default.
