#!/usr/bin/env bash
set -ex

DLLIMPORT=wildland
CC=g++
CSHARP_COMPILER=mcs

# --------------------------------------------------------------------------------------------------

cd /ffi_build
swig -dllimport ${DLLIMPORT} -module wildland -csharp -c++ -w516,503,476,302,124 -outdir /bindings ffi_swig.i

# --------------------------------------------------------------------------------------------------

${CC} -fpermissive \
      -shared \
      -fPIC \
      -std=c++20 \
      -w \
      -L . \
      ffi_swig_wrap.cxx \
      -lwildland_cargo_lib \
      -o /bindings_test/lib${DLLIMPORT}.so

${CSHARP_COMPILER} \
    -out:/bindings_test/Wildland.Cargo.dll \
    -target:library \
    /bindings/*.cs

cd /bindings_test

cp /ffi_tests/test.cs ./main.cs
${CSHARP_COMPILER} \
    -reference:Wildland.Cargo.dll \
    -out:main.exe \
    main.cs

mono main.exe

mkdir -p /out/lib /out/src
mv /bindings_test/lib${DLLIMPORT}.so /out/lib
mv /bindings/*.cs /out/src
