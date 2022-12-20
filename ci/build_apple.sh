#!/bin/sh
# Maintainers:
#     Piotr Isajew (pisajew@wildland.io)
#     Ivan Sinitsa (ivan@wildland.io)

set -ex

# Define
# BUILD_ROOT=$CI_PROJECT_DIR
BUILD_ROOT="/Users/wildland/src/wildland-core"
MODULE="wildlandx"
PKG_OUT="out_dir"

# Build iOS framework
DESTROOT="$BUILD_ROOT/wildlandx_ios.build"
FW_IOS_OUT="$DESTROOT/$MODULE.framework"
./ci/apple/build_apple_ios.sh $DESTROOT $BUILD_ROOT

# Build iOS Simulator framework
DESTROOT="$BUILD_ROOT/wildlandx_ios_simulator.build"
FW_IOS_SIM_OUT="$DESTROOT/$MODULE.framework"
./ci/apple/build_apple_ios_simulator.sh $DESTROOT $BUILD_ROOT

## Build macOS framework
DESTROOT="$BUILD_ROOT/wildlandx_mac.build"
FW_MAC_OUT="$DESTROOT/$MODULE.framework"
./ci/apple/build_apple_mac.sh $DESTROOT $BUILD_ROOT

# Create output folder
FW_UNIVERSAL_OUT="wildlandx_apple_universal.build"

if [ -d "$FW_UNIVERSAL_OUT" ]; then
    rm -rf "$FW_UNIVERSAL_OUT"
fi

mkdir $FW_UNIVERSAL_OUT

# Create universal framework for all the previous jobs
cd $FW_UNIVERSAL_OUT
xcodebuild -create-xcframework \
           -framework $FW_IOS_OUT \
           -framework $FW_IOS_SIM_OUT \
           -framework $FW_MAC_OUT \
           -output "wildlandx.xcframework"
mkdir $PKG_OUT
ditto -c -k --sequesterRsrc --keepParent wildlandx.xcframework $PKG_OUT/wildlandx.xcframework.zip
