#!/bin/sh
# Maintainers:
#     Piotr Isajew (pisajew@wildland.io)
#     Ivan Sinitsa (ivan@wildland.io)

set -ex
DEBUG_OPTS="-g"
DESTROOT="$1"
RUST_SRCDIR="$2"
SWIFT_BRIDGE_OUTDIR="$RUST_SRCDIR/crates/wildland-cargo-lib/_generated_ffi_code"
INPUT="$SWIFT_BRIDGE_OUTDIR/ffi_swift.swift"
RUST_LIB="libwildland_cargo_lib.a"

# Name of Swift module for the SDK.
MODULE="wildlandx"
ARCHS="$3"

RUST_ARCH_ARM64=$4
RUST_ARCH_X86_64=$5

ARCH_EXTENSION="$6"

XCRUN_SDK="$8"

export OS_DEPLOYMENT_TARGET=$7
export SDKROOT=$(xcrun -sdk $XCRUN_SDK --show-sdk-path)

TARGET_PLATFORM="$9"
FW_OUT="$DESTROOT/$MODULE.framework"

if [ $TARGET_PLATFORM = "MacOSX" ]; then
    ADDITIONAL_LIBRARY_PATH="/Versions/A"
else
    ADDITIONAL_LIBRARY_PATH=""
fi

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
    HEADER_OUT="$FW_OUT$ADDITIONAL_LIBRARY_PATH/Headers/"
    test -d "$HEADER_OUT" || mkdir -p "$HEADER_OUT"
    MOD_OUT="$FW_OUT$ADDITIONAL_LIBRARY_PATH/Modules/$MODULE.swiftmodule/"
    mkdir -p $MOD_OUT


cat > "$HEADER_OUT/$MODULE.h" <<EOF
#import <Foundation/Foundation.h>
EOF
cp $GLUE_MOD/*.h $HEADER_OUT
cat > "$FW_OUT$ADDITIONAL_LIBRARY_PATH/Modules/module.modulemap" <<EOF

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
cp $FW_OUT$ADDITIONAL_LIBRARY_PATH/Modules/module.modulemap .
ln -s $HEADER_OUT .
INPUT=input.swift
    cat $SWIFT_BRIDGE_OUTDIR/ffi_swift.swift >> $INPUT

    ARCH_TARGET=$arch-apple-ios$OS_DEPLOYMENT_TARGET$ARCH_EXTENSION
    if [ $TARGET_PLATFORM = "MacOSX" ]; then
        ARCH_TARGET=$arch-apple-macos$OS_DEPLOYMENT_TARGET
    fi

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
          -Xlinker -application_extension \
          -install_name @rpath/$MODULE.framework$ADDITIONAL_LIBRARY_PATH/$MODULE \
          -L. \
          -L/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/$XCRUN_SDK -L/usr/lib/swift \
          -lwildland_cargo_lib \
          -o $DESTDIR/$MODULE

    if [ $TARGET_PLATFORM = "MacOSX" ]; then
        cp -v $MODULE_PATH $MOD_OUT/$arch-apple-macos.swiftmodule
        cp -v $MODULE_INTERFACE_PATH $MOD_OUT/$arch-apple-macos.swiftinterface
        cp -v $DESTDIR/$MODULE-Swift.h $HEADER_OUT
    else
        cp -v $MODULE_PATH $MOD_OUT/$arch-apple-ios$ARCH_EXTENSION.swiftmodule
        cp -v $MODULE_INTERFACE_PATH $MOD_OUT/$arch-apple-ios$ARCH_EXTENSION.swiftinterface
        cp -v $DESTDIR/$MODULE-Swift.h $HEADER_OUT    
    fi
done

xcrun lipo -create $DESTROOT/*/$MODULE -output $FW_OUT$ADDITIONAL_LIBRARY_PATH/$MODULE

RES_OUT="$FW_OUT/"
if [ $TARGET_PLATFORM = "MacOSX" ]; then
    RES_OUT="$FW_OUT$ADDITIONAL_LIBRARY_PATH/Resources/"
fi

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
           <string>$TARGET_PLATFORM</string>
        </array>
        <key>CFBundleVersion</key>
        <string>1</string>
        <key>LSMinimumSystemVersion</key>
        <string>$OS_DEPLOYMENT_TARGET</string>
        <key>NSHumanReadableCopyright</key>
        <string>Copyright (C) 2022, Golem Foundation</string>
</dict>
</plist>
EOF

if [ $TARGET_PLATFORM = "MacOSX" ]; then
    cd $FW_OUT/Versions
    ln -s A Current
    cd ..
    ln -s Versions/Current/Headers .
    ln -s Versions/Current/Modules .
    ln -s Versions/Current/Resources .

    ln -s Versions/Current/$MODULE .
fi
