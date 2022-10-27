#!/bin/sh
# Maintainers: 
#     Piotr Isajew (pisajew@wildland.io)
set -ex
DEBUG_OPTS="-g"
BUILD_ROOT=$CI_PROJECT_DIR
DESTROOT="$BUILD_ROOT/wildlandx_macos.build"
RUST_SRCDIR=$CI_PROJECT_DIR
SWIFT_BRIDGE_OUTDIR="$RUST_SRCDIR/crates/wildland-cargo-lib/_generated_ffi_code"
INPUT="$SWIFT_BRIDGE_OUTDIR/ffi_swift.swift"
RUST_LIB="libwildland_cargo_lib.a"

# Name of Swift module for the SDK.
MODULE="wildlandx"
ARCHS="x86_64 arm64"
FW_OUT="$DESTROOT/$MODULE.framework"
PKG_OUT="out_dir"

# Google storage URL for binary SDK uploads.
UPLOAD_URL="gs://wildland-apple-dev-binaries"

# URL from which binary packages can be fetched.
FETCH_URL="https://xcode-proxy.wildland.dev/wildland-apple-dev-binaries"

# GIT repository URL where package manifests should be pushed.
MANIFEST_REPOSITORY="git@gitlab.com:wildland/corex/sdk-apple.git"
# Target branch to which package manifests are to be pushed.
MANIFEST_BRANCH="master"

RUST_ARCH_X86_64=x86_64-apple-darwin
RUST_ARCH_ARM64=aarch64-apple-darwin

export MACOS_DEPLOYMENT_TARGET=12.3
export SDKROOT=$(xcrun -sdk macosx${MACOS_DEPLOYMENT_TARGET} --show-sdk-path)

if [ -d "$DESTROOT" ]; then
    rm -rf "$DESTROOT"
fi

upload_framework() {
    local fw="$1"
    echo uploading "$1"

    MANIFEST_DIR=`mktemp -d`
    rmdir $MANIFEST_DIR
    git clone $MANIFEST_REPOSITORY $MANIFEST_DIR
    SAVED_WD=`pwd`
    cd $MANIFEST_DIR
    git checkout $MANIFEST_BRANCH
    
    cat > Package.swift <<EOF
// swift-tools-version: 5.6

import PackageDescription

let package = Package(
  name: "wildlandx",
  products: [
    .library(name: "wildlandx", targets: ["wildlandx"])
  ],
  targets: [
    .binaryTarget(
      name: "wildlandx",
      url: "$FETCH_URL/wildlandx.xcframework.zip",
      checksum: "$(shasum -a 256 $SAVED_WD/wildlandx.xcframework.zip | awk '{print $1}')"
    )
  ]
)
EOF

    git add Package.swift
    git commit -m "Build script updated package manifest at $(date +%Y-%m-%d)"
    git push
    cd $SAVED_WD
    
    gcloud auth activate-service-account --key-file=$CLOUD_CREDENTIALS
    gsutil cp "$1" $UPLOAD_URL
}
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
    HEADER_OUT="$FW_OUT/Versions/A/Headers/"
    test -d "$HEADER_OUT" || mkdir -p "$HEADER_OUT"
    MOD_OUT="$FW_OUT/Versions/A/Modules/$MODULE.swiftmodule/"
    mkdir -p $MOD_OUT
 

cat > "$HEADER_OUT/$MODULE.h" <<EOF
#import <Foundation/Foundation.h>
EOF
cp $GLUE_MOD/*.h $HEADER_OUT
cat > "$FW_OUT/Versions/A/Modules/module.modulemap" <<EOF

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
cp $FW_OUT/Versions/A/Modules/module.modulemap .
ln -s $HEADER_OUT .
INPUT=input.swift
    cat $SWIFT_BRIDGE_OUTDIR/ffi_swift.swift >> $INPUT
    
    # Prepare module metadata
    xcrun swiftc -v -module-name "$MODULE" \
          -I . \
          $INPUT \
          -Rmodule-loading \
          -target $arch-apple-macos12.3 \
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
          -target $arch-apple-macos12.3 \
          $DEBUG_OPTS \
          -parse-as-library \
          -import-underlying-module \
          -enable-library-evolution \
          -o $OBJ_OUT

    # Link architecture-specific binary
    xcrun clang \
          -v \
          -target $arch-apple-macos12.3 \
          -dynamiclib \
          $OBJ_OUT \
          -install_name @rpath/$MODULE.framework/Versions/A/$MODULE \
          -L. \
          -L/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx -L/usr/lib/swift \
          -lwildland_cargo_lib \
          -o $DESTDIR/$MODULE
    cp -v $MODULE_PATH $MOD_OUT/$arch-apple-macos.swiftmodule
    cp -v $MODULE_INTERFACE_PATH $MOD_OUT/$arch-apple-macos.swiftinterface
    cp -v $DESTDIR/$MODULE-Swift.h $HEADER_OUT
done

xcrun lipo -create $DESTROOT/*/$MODULE -output $FW_OUT/Versions/A/$MODULE
RES_OUT="$FW_OUT/Versions/A/Resources/"
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
           <string>MacOSX</string>
        </array>
        <key>CFBundleVersion</key>
        <string>1</string>
        <key>LSMinimumSystemVersion</key>
        <string>12.3</string>
        <key>NSHumanReadableCopyright</key>
        <string>Copyright (C) 2022, Golem Foundation</string>
</dict>
</plist>
EOF

cd $FW_OUT/Versions
ln -s A Current
cd ..
ln -s Versions/Current/Headers .
ln -s Versions/Current/Modules .
ln -s Versions/Current/Resources .

ln -s Versions/Current/$MODULE .

# TODO: Create frameworks for different platforms
cd $DESTROOT
xcodebuild -create-xcframework \
           -framework $FW_OUT \
           -output wildlandx.xcframework
mkdir $PKG_OUT
ditto -c -k --sequesterRsrc --keepParent wildlandx.xcframework $PKG_OUT/wildlandx.xcframework.zip
cd $PKG_OUT

if [ "$CI_COMMIT_BRANCH" = "main" ]; then
    upload_framework wildlandx.xcframework.zip
fi
