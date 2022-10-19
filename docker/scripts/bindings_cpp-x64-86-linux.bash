#!/usr/bin/env bash
g++ -std=c++20 -w /ffi_tests/test.cpp \
        -I ./ffi_build \
        -L ./ffi_build \
        -l wildland_cargo_lib \
        -l dl \
        -l pthread \
        -l crypto \
        -l ssl \
        -o /test
# valgrind -leak-check=yes ./test
./test