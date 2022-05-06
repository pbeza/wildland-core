#!/usr/bin/env bash

if !(cargo install --list | grep 'cargo-workspaces v' > /dev/null); then
    cargo install cargo-workspaces
fi

if [[ $(git diff --stat) != '' ]]; then
  echo "[!] Your git workspace is dirty. Make sure it's clean before bumping the version."
  exit 1
fi

cargo workspaces version --no-git-tag --no-git-push --amend --allow-branch '*'
