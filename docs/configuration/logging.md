# Configuration Options Related to Logging

## Universal Configuration

### use_logger

- platform: universal
- type: bool
- default: false

Boolean value that enables the config. It is possible to implement your own
logger subscriber, however it may be non-trivial over the ffi bindings. This
option enables default logger subscriber, which is owned and managed by the
Cargo library. Default logger implementation uses `tracing` rust library, and
most of the logs are emitted that way.

### log_level

- platform: universal
- type:
  - internal: tracing::Level
  - external: String
- default: INFO

Enable log level filtering for the logger. By default, logs of level INFO and
higher will be caught. On lower levels additional logs are provided to the user,
however those may not be as useful outside debugging environment.

Release builds can not enable log levels lower than INFO.

### log_file_enabled

- platform: universal
- type: bool
- default: false

Enables or disables logging to file. This can be coupled with default stderr
logging, however does not work if you choose special logging facilities like
OSlog.

### log_file_path

- platform: universal
- type: Option\<string\>
- default: "cargolib_log"

File path representing current log. If `log_file_enabled` is true, this is the
path that will be used to create current log file. This log will be rotated
each hour. to the `log_file_rotate_directory`. Will default if absent.

Paths are checked for existence but not for permissions. Please make sure,
that the application will have rights to write to the specific directory.

### log_file_rotate_directory

- platform: universal
- type: Option\<string\>
- default: "."

File path representing the directory where logs should rotate. Currently, file
rotation is set to one hour and is not configurable. Current logging file
defined by `log_file_path` will be moved here and fresh, empty file will be used
for next rotation period.

Paths are checked for existence but not for permissions. Please make sure,
that the application will have rights to write to the specific directory.

## Apple Specific Configuration

### oslog_category

- platform: macos, ios
- type: Option\<string\>
- default: "None"
- example: `default`

If both `oslog_category` and `oslog_subsystem` are provided on
applicable platform, logging to OSlog will be enabled. Does not work on any
other platform.

### oslog_subsystem

- platform: macos, ios
- type: Option\<string\>
- default: "None"
- example: `io.wildland.cargo`

If both `oslog_category` and `oslog_subsystem` are provided on
applicable platform, logging to OSlog will be enabled. Does not work on any
other platform.
