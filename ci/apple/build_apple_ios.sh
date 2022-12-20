#!/bin/sh
# Maintainers: 
#     Piotr Isajew (pisajew@wildland.io)
#     Ivan Sinitsa (ivan@wildland.io)

DESTROOT="$1"
RUST_SRCDIR="$2"

# Name of Swift module for the SDK.
ARCHS="arm64"
ARCH_EXTENSION="None"
RUST_ARCH_X86_64="None"
RUST_ARCH_ARM64=aarch64-apple-ios
SUPPORTED_PLATFORM=iPhoneOS
XCRUN_SDK=iphoneos

IOS_DEPLOYMENT_TARGET=15.5

# Perform framework build
./ci/apple/build_apple_framework.sh $DESTROOT $RUST_SRCDIR "$ARCHS" "$RUST_ARCH_ARM64" "$RUST_ARCH_X86_64" $ARCH_EXTENSION $IOS_DEPLOYMENT_TARGET $XCRUN_SDK $SUPPORTED_PLATFORM