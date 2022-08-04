#!/bin/sh

set -ex
env
WRKDIR="/tmp/wildland"

TARGETS="x86_64-apple-darwin aarch64-apple-darwin aarch64-apple-ios aarch64-apple-ios-sim"

mkdir -p "/tmp/wildland/Headers"

for target in $TARGETS; do
    SWIFT_BRIDGE_OUT_DIR="$PWD/_generated_swift"  cargo build --features bindings --target $target --target-dir $WRKDIR
    cp _generated_cpp/ffi_cxx.h /tmp/wildland/Headers/ffi_cxx.h
    cp _generated_swift/ffi_swift/ffi_swift.h /tmp/wildland/Headers/ffi_swift.h
    cp _generated_swift/SwiftBridgeCore.h /tmp/wildland/Headers/SwiftBridgeCore.h
done


ls -lR /tmp/wildland > /tmp/wildland.ls-lR

cat > /tmp/wildland/Headers/wildland.h <<EOF
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
     -arch x86_64 /tmp/wildland/x86_64-apple-darwin/debug/libwildland_cargo_lib.a \
     -arch arm64e /tmp/wildland/aarch64-apple-darwin/debug/libwildland_cargo_lib.a

xcodebuild -create-xcframework \
 -library $WRKDIR/libwildland_macos.a \
  -headers $WRKDIR/Headers \
 -library $WRKDIR/aarch64-apple-ios/debug/libwildland_cargo_lib.a \
 -headers $WRKDIR/Headers \
 -library $WRKDIR/aarch64-apple-ios-sim/debug/libwildland_cargo_lib.a \
 -headers $WRKDIR/Headers \
 -output $WRKDIR/wildland.xcframework
