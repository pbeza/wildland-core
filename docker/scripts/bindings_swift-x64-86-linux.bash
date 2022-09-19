#!/usr/bin/env bash
swiftc -L target/debug -lwildland_cargo_lib -lstdc++ \
        -L ./ffi_build \
        -I ffi_build -import-objc-header \
        ./ffi_build/ffi_swift.h \
        ./ffi_build/ffi_swift.swift \
        ffi_tests/main.swift \
        -o ffi_build/swift_app
./ffi_build/swift_app