#!/bin/sh
# Maintainers: 
#     Piotr Isajew (pisajew@wildland.io)
#     Ivan Sinitsa (ivan@wildland.io)

set -ex
# Google storage URL for binary SDK uploads.
UPLOAD_URL="gs://wildland-apple-dev-binaries"

# URL from which binary packages can be fetched.
FETCH_URL="https://xcode-proxy.wildland.dev/wildland-apple-dev-binaries"

# GIT repository URL where package manifests should be pushed.
MANIFEST_REPOSITORY="git@gitlab.com:wildland/corex/sdk-apple.git"
# Target branch to which package manifests are to be pushed.
MANIFEST_BRANCH="master"

# Define 
BUILD_ROOT=$CI_PROJECT_DIR
MODULE="wildlandx"
PKG_OUT="out_dir"

# Build iOS framework
DESTROOT="$BUILD_ROOT/wildlandx_ios.build"
FW_IOS_OUT="$DESTROOT/$MODULE.framework"
./ci/build_apple_ios.sh $DESTROOT

# Build iOS Simulator framework
DESTROOT="$BUILD_ROOT/wildlandx_ios_simulator.build"
FW_IOS_SIM_OUT="$DESTROOT/$MODULE.framework"
./ci/build_apple_ios_simulator.sh $DESTROOT

# Build macOS framework
DESTROOT="$BUILD_ROOT/wildlandx_mac.build"
FW_MAC_OUT="$DESTROOT/$MODULE.framework"
./ci/build_apple_mac.sh $DESTROOT

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
cd $PKG_OUT

# Perform framework upload
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
  platforms: [
    .macOS(.v12), .iOS(.v15)
  ],
  products: [
    .library(name: "wildlandx", targets: ["wildlandx"])
  ],
  targets: [
    .binaryTarget(
      name: "wildlandx",
      url: "$FETCH_URL/wildlandx.xcframework.zip",
      checksum: "$(shasum -a 256 $SAVED_WD/wildlandx.xcframework.zip | awk '{print $1}')"
    ),
    .testTarget(
        name: "WildlandXTests",
        dependencies: ["wildlandx"]
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

# Upload the framework
if [ "$CI_COMMIT_BRANCH" = "master" ] || [ "$CI_COMMIT_BRANCH" = "develop" ]; then
    upload_framework wildlandx.xcframework.zip
fi
