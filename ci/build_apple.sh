#!/bin/sh

set -ex
env
WRKDIR="$PWD/wildland-build"


TARGETS="x86_64-apple-darwin aarch64-apple-darwin aarch64-apple-ios aarch64-apple-ios-sim"
export SWIFT_BRIDGE_OUT_DIR="$PWD/_generated_swift"
export CPP_BRIDGE_OUT_DIR="$PWD/_generated_cpp"
test -d "$SWIFT_BRIDGE_OUT_DIR" || mkdir -p "$SWIFT_BRIDGE_OUT_DIR"
test -d "$CPP_BRIDGE_OUT_DIR" || mkdir -p "$CPP_BRIDGE_OUT_DIR"
mkdir -p "$WRKDIR/Headers"

for target in $TARGETS; do
    cargo build --features bindings --target $target --target-dir $WRKDIR
    cp $CPP_BRIDGE_OUT_DIR/ffi_cxx.h $WRKDIR/Headers/ffi_cxx.h
    cp $SWIFT_BRIDGE_OUT_DIR/ffi_swift/ffi_swift.h $WRKDIR/Headers/ffi_swift.h
    cp $SWIFT_BRIDGE_OUT_DIR/SwiftBridgeCore.h $WRKDIR/Headers/SwiftBridgeCore.h
done


cat > $WRKDIR/Headers/wildland.h <<EOF
#ifndef __wildland_h__
#define __wildland_h__
extern "C" {
#include <SwiftBridgeCore.h>
#include <ffi_swift.h>
}
#include <ffi_cxx.h>
#endif // _wildland_h__
EOF

lipo -create -output $WRKDIR/libwildland_macos.a \
     -arch x86_64 $WRKDIR/x86_64-apple-darwin/debug/libwildland_cargo_lib.a \
     -arch arm64e $WRKDIR/aarch64-apple-darwin/debug/libwildland_cargo_lib.a

xcodebuild -create-xcframework \
 -library $WRKDIR/libwildland_macos.a \
  -headers $WRKDIR/Headers \
 -library $WRKDIR/aarch64-apple-ios/debug/libwildland_cargo_lib.a \
 -headers $WRKDIR/Headers \
 -library $WRKDIR/aarch64-apple-ios-sim/debug/libwildland_cargo_lib.a \
 -headers $WRKDIR/Headers \
 -output $WRKDIR/wildland.xcframework
