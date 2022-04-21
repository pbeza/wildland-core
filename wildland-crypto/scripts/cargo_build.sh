#!/bin/sh

set -ex

CARGO_TARGET=${1:?"need cargo target"}
CUSTOM_SDK_ROOT=$2

CARGO_TOOL="$SRCROOT/$TARGET_NAME/scripts/cargo.sh"

if [ -z "$CUSTOM_SDK_ROOT" ]; then
    $CARGO_TOOL build --target $CARGO_TARGET $CARGO_OPTS
else
    CUSTOM_LIBRARY_PATH="$CUSTOM_SDK_ROOT/usr/lib:${LIBRARY_PATH:-}"

    SDKROOT=$CUSTOM_SDK_ROOT \
    LIBRARY_PATH=$CUSTOM_LIBRARY_PATH \
        $CARGO_TOOL build --target $CARGO_TARGET $CARGO_OPTS
fi
