#!/usr/bin/env bash

# This is a helper script for release branches. Its purpose is to automate version bumping
# for releases to prevent mistakes that may occur in manually driven flows.
#
# This script will:
#   - make sure the working tree is clean
#   - bump versions for all crates in the workspace depending on given argument (major, minor or patch)
#   - mark the crates versions as release canditates (eg. 0.1.2-rc.[0|1|2|...])
#
# This script will NOT:
#   - create any tags
#   - push anything to the remote
#
# Usage:
#   ./bump.sh [major|minor|patch|release]

if !(cargo install --list | grep 'cargo-workspaces v' > /dev/null); then
  cargo install cargo-workspaces
fi

if [[ $(git diff --stat) != '' ]]; then
  echo "[!] Your git workspace is dirty. Make sure it's clean before bumping the version."
  exit 1
fi

if [ "$#" -ne 1 ]; then
  echo "[!] Wrong number of arguments"
  exit 1
fi

case $1 in
  major)
    ;;
  minor)
    ;;
  patch)
    ;;
  release)
    ;;
  *)
    echo "[!] Invalid argument"
    exit 1
    ;;
esac

cargo workspaces version \
  --no-git-tag \
  --no-git-push \
  --allow-branch 'master' \
  --pre-id 'rc' \
  "pre$1"

git log HEAD -1 | ruby ci/commit_helper.rb | xargs -I@ git commit --amend --message "(bump) pre-release crates version to @"
