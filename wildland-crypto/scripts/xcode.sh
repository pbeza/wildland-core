#!/bin/sh

set -ex

# Build libcargo_common.a for current platform family
cd $TARGET_NAME

CARGO_BUILD_CONFIGURATION=$(printf $CONFIGURATION | tr '[:upper:]' '[:lower:]' )
CARGO_BUILD=$SRCROOT/$TARGET_NAME/scripts/cargo_build.sh
CARGO_LIB_NAME="libcargo_common.a"
CARGO_LIBS=()
if [[ $PLATFORM_NAME == "macosx" ]]; then
    if [[ " ${ARCHS[*]} " =~ " "arm64" " ]]; then
        echo "Building for macOS - ARM"
        $CARGO_BUILD "aarch64-apple-darwin"
        CARGO_LIBS+=("$CARGO_TARGET_DIR/aarch64-apple-darwin/$CARGO_BUILD_CONFIGURATION/$CARGO_LIB_NAME")
    fi
    if [[ " ${ARCHS[*]} " =~ " "x86_64" " ]]; then
        echo "Building for macOS - Intel"
        $CARGO_BUILD "x86_64-apple-darwin"
        CARGO_LIBS+=("$CARGO_TARGET_DIR/x86_64-apple-darwin/$CARGO_BUILD_CONFIGURATION/$CARGO_LIB_NAME")
    fi
elif [[ $PLATFORM_NAME == "iphoneos" ]]; then
    echo "Building for iOS"
    IOS_SDKROOT=`xcrun --sdk iphoneos --show-sdk-path`
    $CARGO_BUILD "aarch64-apple-ios" "$IOS_SDKROOT"
    CARGO_LIBS+=("$CARGO_TARGET_DIR/aarch64-apple-ios/$CARGO_BUILD_CONFIGURATION/$CARGO_LIB_NAME")
elif [[ $PLATFORM_NAME == "iphonesimulator" ]]; then
    SIM_SDKROOT=`xcrun --sdk iphonesimulator --show-sdk-path`
    if [[ " ${ARCHS[*]} " =~ " "arm64" " ]]; then
        echo "Building for iPhoneSimulator - ARM is not supported yet"
        exit 1
    fi
    if [[ " ${ARCHS[*]} " =~ " "x86_64" " ]]; then
        echo "Building for iPhoneSimulator - Intel"
        $CARGO_BUILD "x86_64-apple-ios" "$SIM_SDKROOT"
        CARGO_LIBS+=("$CARGO_TARGET_DIR/x86_64-apple-ios/$CARGO_BUILD_CONFIGURATION/$CARGO_LIB_NAME")
    fi
else
    echo "Unsupported platform family = $PLATFORM_FAMILY_NAME"
    exit 1
fi

# Create final library
echo "Create final library for architectures $ARCHS"

LIPO_LIBS=""
for CARGO_LIB in ${CARGO_LIBS[@]}; do
    LIPO_LIBS="$LIPO_LIBS $CARGO_LIB"
done

FINAL_CARGO_LIB="$BUILT_PRODUCTS_DIR/$CARGO_LIB_NAME"
rm -f "$FINAL_CARGO_LIB"
lipo -create $LIPO_LIBS -output $FINAL_CARGO_LIB
