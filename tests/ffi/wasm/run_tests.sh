#!/bin/bash
set -ex

cleanup() {
    kill $(jobs -p)
}

trap "cleanup" SIGINT SIGTERM EXIT

npm install
python3 -m http.server 9200 &
node --unhandled-rejections=strict wasm_test.js
