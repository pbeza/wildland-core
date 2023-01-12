#!/bin/sh
# Maintainers:
#     Michał Kluczek (michal@wildland.io)
#     Piotr Isajew   (pisajew@wildland.io)
set -ex

# Google storage URL for binary SDK uploads.
UPLOAD_URL="gs://wildland-apple-dev-binaries"

# Location of archive built from a previous job
ARTIFACT_ARCHIVE=$CI_PROJECT_DIR/wildlandx_apple_universal.build/out_dir/wildlandx.xcframework.zip

# URL from which binary packages can be fetched.
FETCH_URL="https://xcode-proxy.wildland.dev/wildland-apple-dev-binaries"

# GIT repository URL where package manifests should be pushed.
MANIFEST_REPOSITORY="https://wildland-bot:$HOUSEKEEPER_CI_TOKEN@gitlab.com/wildland/corex/sdk-apple.git"

# MANIFEST_BRANCH = Target branch to which package manifests are to be pushed.
# PACKAGE_SUFFIX = Unique archive filename suffix based on whether its a tag
#                  release or a development release

if [ -z ${CI_COMMIT_TAG+x} ]; then
  # If CI_COMMIT_TAG var is unset (ie. it's *not* a tag pipeline)
  MANIFEST_BRANCH="develop"
  PACKAGE_SUFFIX=$CI_COMMIT_SHORT_SHA
  echo "Detected development commit"
else
  # Otherwise... (ie. tag-triggered pipeline)
  MANIFEST_BRANCH="master"
  PACKAGE_SUFFIX=$CI_COMMIT_TAG
  PUSH_TAG="true"
  echo "Detected a tag commit"
fi

ARCHIVE_NAME="wildlandx.xcframework-${PACKAGE_SUFFIX}.zip"

echo "Uploading archive: $ARCHIVE_NAME"

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
      url: "${FETCH_URL}/${ARCHIVE_NAME}",
      checksum: "$(shasum -a 256 "${ARTIFACT_ARCHIVE}" | awk '{print $1}')"
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
if [ "$PUSH_TAG" = "true" ]; then
  git tag -m "Tag created automatically by CoreX CI/CD" $CI_COMMIT_TAG
  git push origin $CI_COMMIT_TAG
fi
cd $SAVED_WD

gcloud auth activate-service-account --key-file=$CLOUD_CREDENTIALS
gsutil cp "${ARTIFACT_ARCHIVE}" ${UPLOAD_URL}/${ARCHIVE_NAME}
