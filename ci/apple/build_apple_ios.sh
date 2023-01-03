#!/bin/sh
# Maintainers: 
#     Piotr Isajew (pisajew@wildland.io)
#     Ivan Sinitsa (ivan@wildland.io)

. ./ci/apple/build_apple_constants.sh

DESTROOT="$1"
RUST_SRCDIR="$2"

# Name of Swift module for the SDK.
ARCHS="arm64"
ARCH_EXTENSION=""
RUST_ARCH_X86_64=""
RUST_ARCH_ARM64=aarch64-apple-ios
SUPPORTED_PLATFORM=iPhoneOS
XCRUN_SDK=iphoneos

# Perform framework build
./ci/apple/build_apple_framework.sh $DESTROOT $RUST_SRCDIR "$ARCHS" "$RUST_ARCH_ARM64" "$RUST_ARCH_X86_64" "$ARCH_EXTENSION" $IOS_DEPLOYMENT_TARGET $XCRUN_SDK $SUPPORTED_PLATFORM