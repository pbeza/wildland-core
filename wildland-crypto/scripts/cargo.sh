#!/bin/sh

set -ex

PATH="$PATH:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin"
CARGO_TOOL=$(which cargo)

${CARGO_TOOL} "$@"
