#!/usr/bin/env bash
set -ex

cd /ffi_build

cp /ffi_tests/main.swift .
mkdir -p ./out

swiftc -L . -lwildland_cargo_lib -lstdc++ -lssl -lcrypto \
        -I . -import-objc-header \
        ./ffi_swift.h \
        ./ffi_swift.swift \
        main.swift \
        -o swift_app
./swift_app
mv ./ffi_swift.h /out
mv ./ffi_swift.swift /out
