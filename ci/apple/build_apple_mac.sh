#!/bin/sh
# Maintainers: 
#     Piotr Isajew (pisajew@wildland.io)
#     Ivan Sinitsa (ivan@wildland.io)

DESTROOT="$1"
RUST_SRCDIR="$2"

# Name of Swift module for the SDK.
ARCHS="x86_64 arm64"
ARCH_EXTENSION="None"
RUST_ARCH_X86_64=x86_64-apple-darwin
RUST_ARCH_ARM64=aarch64-apple-darwin
SUPPORTED_PLATFORM=MacOSX
XCRUN_SDK=macosx

MACOS_DEPLOYMENT_TARGET=12.3

# Perform framework build
./ci/apple/build_apple_framework.sh $DESTROOT $RUST_SRCDIR "$ARCHS" "$RUST_ARCH_ARM64" "$RUST_ARCH_X86_64" $ARCH_EXTENSION $MACOS_DEPLOYMENT_TARGET $XCRUN_SDK $SUPPORTED_PLATFORM