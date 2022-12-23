#!/bin/sh
# Maintainers: 
#     Piotr Isajew (pisajew@wildland.io)
#     Ivan Sinitsa (ivan@wildland.io)

. ./ci/apple/build_apple_constants.sh

DESTROOT="$1"
RUST_SRCDIR="$2"

# Name of Swift module for the SDK.
ARCHS="x86_64 arm64"
ARCH_EXTENSION="-simulator"
RUST_ARCH_X86_64=x86_64-apple-ios
RUST_ARCH_ARM64=aarch64-apple-ios-sim
SUPPORTED_PLATFORM=iPhoneSimulator
XCRUN_SDK=iphonesimulator

# Perform framework build
./ci/apple/build_apple_framework.sh $DESTROOT $RUST_SRCDIR "$ARCHS" "$RUST_ARCH_ARM64" "$RUST_ARCH_X86_64" $ARCH_EXTENSION $IOS_DEPLOYMENT_TARGET $XCRUN_SDK $SUPPORTED_PLATFORM