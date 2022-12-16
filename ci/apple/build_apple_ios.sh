#!/bin/sh
# Maintainers: 
#     Piotr Isajew (pisajew@wildland.io)
#     Ivan Sinitsa (ivan@wildland.io)

set -ex
DEBUG_OPTS="-g"
DESTROOT="$1"
RUST_SRCDIR=$CI_PROJECT_DIR
SWIFT_BRIDGE_OUTDIR="$RUST_SRCDIR/crates/wildland-cargo-lib/_generated_ffi_code"
INPUT="$SWIFT_BRIDGE_OUTDIR/ffi_swift.swift"
RUST_LIB="libwildland_cargo_lib.a"

# Name of Swift module for the SDK.
MODULE="wildlandx"
ARCHS="arm64"
FW_OUT="$DESTROOT/$MODULE.framework"

RUST_ARCH_ARM64=aarch64-apple-ios

export IOS_DEPLOYMENT_TARGET=15.5
export SDKROOT=$(xcrun -sdk iphoneos --show-sdk-path)

if [ -d "$DESTROOT" ]; then
    rm -rf "$DESTROOT"
fi

mkdir "$DESTROOT"
mkdir "$FW_OUT"

cd $DESTROOT

for arch in $ARCHS; do
    DESTDIR="$DESTROOT/$arch"
    WRKDIR="$DESTROOT/${arch}-intermediates"
    INPUT="$DESTDIR/ffi_swift.swift"
    MODULE_PATH="$DESTDIR/$MODULE".swiftmodule
    MODULE_INTERFACE_PATH="$DESTDIR/$MODULE".swiftinterface

    curdir=$(pwd)
    # Build rust glue code
    mkdir $DESTDIR
    cd $RUST_SRCDIR
    eval rustarch="\$RUST_ARCH_$(echo $arch | tr '[:lower:]' '[:upper:]')"
    cargo build --target $rustarch --target-dir "$DESTDIR" --features bindings
    cd $curdir
    # Prepare glue module
    mkdir $WRKDIR
    cd $WRKDIR
    GLUE_MOD="${MODULE}_rust"
    mkdir "$GLUE_MOD"
    cat > "$GLUE_MOD/module.modulemap" <<EOF
module $GLUE_MOD {
    header "${GLUE_MOD}.h"
    export *
    link "$(echo $RUST_LIB | sed 's/^lib//' | sed 's/\.a$//')"
}
EOF
    cp $SWIFT_BRIDGE_OUTDIR/ffi_swift.h $GLUE_MOD
    cp $DESTROOT/$arch/$rustarch/debug/$RUST_LIB .

    cat > "$GLUE_MOD/${GLUE_MOD}.h" <<EOF
#ifndef __${GLUE_MOD}_h__
#define __${GLUE_MOD}_h__
#include "ffi_swift.h"
#endif
EOF
    HEADER_OUT="$FW_OUT/Headers/"
    test -d "$HEADER_OUT" || mkdir -p "$HEADER_OUT"
    MOD_OUT="$FW_OUT/Modules/$MODULE.swiftmodule/"
    mkdir -p $MOD_OUT
 

cat > "$HEADER_OUT/$MODULE.h" <<EOF
#import <Foundation/Foundation.h>
EOF
cp $GLUE_MOD/*.h $HEADER_OUT
cat > "$FW_OUT/Modules/module.modulemap" <<EOF

framework module $MODULE {
  umbrella header "$MODULE.h"

  export *
  module * { export * }

  module $GLUE_MOD {
    header "${GLUE_MOD}.h"
    export *
    link "$(echo $RUST_LIB | sed 's/^lib//' | sed 's/\.a$//')"
  }
}
EOF
cp $FW_OUT/Modules/module.modulemap .
ln -s $HEADER_OUT .
INPUT=input.swift
    cat $SWIFT_BRIDGE_OUTDIR/ffi_swift.swift >> $INPUT
    
    ARCH_TARGET=$arch-apple-ios$IOS_DEPLOYMENT_TARGET
    
    # Prepare module metadata
    xcrun swiftc -v -module-name "$MODULE" \
          -I . \
          $INPUT \
          -Rmodule-loading \
          -target $ARCH_TARGET \
          $DEBUG_OPTS \
          -parse-as-library \
          -emit-dependencies \
          -O -whole-module-optimization \
          -emit-module \
          -emit-module-path $MODULE_PATH \
          -enable-library-evolution \
          -emit-module-interface-path  $MODULE_INTERFACE_PATH \
          -emit-objc-header \
          -emit-objc-header-path "$DESTDIR/$MODULE-Swift.h" \
          -import-underlying-module

    # Compile sources
    OBJ_OUT=$DESTDIR/`basename $INPUT .swift`.o
    xcrun swiftc -c $INPUT \
          -I . \
          -module-name "$MODULE" \
          -stack-check \
          -target $ARCH_TARGET \
          $DEBUG_OPTS \
          -parse-as-library \
          -import-underlying-module \
          -enable-library-evolution \
          -o $OBJ_OUT

    # Link architecture-specific binary
    xcrun clang \
          -v \
          -target $ARCH_TARGET \
          -dynamiclib \
          $OBJ_OUT \
          -install_name @rpath/$MODULE.framework/$MODULE \
          -L. \
          -L/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/iphoneos -L/usr/lib/swift \
          -lwildland_cargo_lib \
          -o $DESTDIR/$MODULE
    cp -v $MODULE_PATH $MOD_OUT/$arch-apple-ios.swiftmodule
    cp -v $MODULE_INTERFACE_PATH $MOD_OUT/$arch-apple-ios.swiftinterface
    cp -v $DESTDIR/$MODULE-Swift.h $HEADER_OUT
done

xcrun lipo -create $DESTROOT/*/$MODULE -output $FW_OUT/$MODULE
RES_OUT="$FW_OUT/"
test -d "$RES_OUT" || mkdir -p "$RES_OUT"

cat > "$RES_OUT/Info.plist" <<EOF
<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE plist PUBLIC "-//Apple/DTD PLIST 1.0/EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
        <key>CFBundleExecutable</key>
        <string>$MODULE</string>
        <key>CFBundleIdentifier</key>
        <string>io.wildland.wildlandx</string>
        <key>CFBundleInfoDictionaryVersion</key>
        <string>6.0</string>
        <key>CFBundleName</key>
        <string>$MODULE</string>
        <key>CFBundlePackageType</key>
        <string>FMWK</string>
        <key>CFBundleShortVersionString</key>
        <string>1.0</string>
        <key>CFBundleSupportedPlatforms</key>
        <array>
           <string>iPhoneOS</string>
        </array>
        <key>CFBundleVersion</key>
        <string>1</string>
        <key>LSMinimumSystemVersion</key>
        <string>$IOS_DEPLOYMENT_TARGET</string>
        <key>NSHumanReadableCopyright</key>
        <string>Copyright (C) 2022, Golem Foundation</string>
</dict>
</plist>
EOF
