$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# For Windows, we most likely will support the following targets:
#   x86_64-pc-windows-msvc
#   aarch64-pc-windows-msvc
#
# The target archs in MSVC buildtools are respectively
#   amd64
#   arm64

$env:BUILD_TARGET = "x86_64-pc-windows-msvc"
$env:ARCH = "amd64"
$env:DLLIMPORT = "Wildland.dll"
$env:CC = "cl.exe"
$env:CSHARP_COMPILER = "csc"

$env:CARGO_REGISTRIES_WL_DEV_INDEX = "https://crates.wildland.dev/git/index"

$env:PROJECT_ROOT = if ($env:CI_PROJECT_DIR) { $env:CI_PROJECT_DIR } else { "/app" }
$env:FFI_BUILD_DIR = "/ffi_build"
$env:TARGET_DIR = "$env:PROJECT_ROOT/target/$env:BUILD_TARGET"
$env:CXX_OUT = "$env:TARGET_DIR/cxxbridge/wildland-admin-manager/_temporary"
$env:CXX_RUST = "$env:TARGET_DIR/cxxbridge/rust"
$env:CXX_LIB = "$env:TARGET_DIR/debug/wildland_admin_manager.lib"

# Create build dirs
Write-Host "---------- Creating build directories ----------"
mkdir -p /bindings
mkdir -p /bindings_test
mkdir -p /ffi_build
mkdir -p /ffi_tests
mkdir -p /scripts

# Make sure we're in the project's root directory
Set-Location $env:PROJECT_ROOT

Write-Host "---------- Adding rust target $env:BUILD_TARGET ----------"
Start-Process -FilePath "rustup" -Wait -NoNewWindow -ArgumentList "target add $env:BUILD_TARGET"

Write-Host "---------- Cargo Build ----------"
Start-Process -FilePath "cargo" -Wait -NoNewWindow -ArgumentList "build --features bindings --target $env:BUILD_TARGET"

Write-Host "---------- Cargo Build artifacts ----------"
cp "$env:CXX_RUST/cxx.h" "$env:FFI_BUILD_DIR"
cp "$env:CXX_OUT/ffi_cxx.rs.h" "$env:FFI_BUILD_DIR"
cp "$env:CXX_OUT/ffi_cxx.rs.cc" "$env:FFI_BUILD_DIR"
cp "$env:PROJECT_ROOT/crates/wildland-admin-manager/wildland.i" "$env:FFI_BUILD_DIR"
cp "$env:PROJECT_ROOT/crates/wildland-admin-manager/_temporary/generated.i" "$env:FFI_BUILD_DIR"
cp "$env:CXX_LIB" "$env:FFI_BUILD_DIR"

Write-Host "---------- Apply SWIG Workarounds ----------"
sed -i 's/final//g' "$env:FFI_BUILD_DIR/ffi_cxx.rs.h"
sed -i 's/::rust/::rust::cxxbridge1/g' "$env:FFI_BUILD_DIR/ffi_cxx.rs.h"
sed -i 's/\[\[noreturn\]\]//g' "$env:FFI_BUILD_DIR/ffi_cxx.rs.h"
sed -i 's/\.\.\.//g' "$env:FFI_BUILD_DIR/ffi_cxx.rs.h"

Write-Host "---------- Create SWIG-Generated C# Bindings ----------"
Start-Process -WorkingDirectory /ffi_build -FilePath "swig" -Wait -NoNewWindow -ArgumentList "-dllimport $env:DLLIMPORT -csharp -c++ -w'516,503,476,302,124' -outdir /bindings wildland.i"

Write-Host "---------- BUILD /bindings_test/Wildland.dll ----------"
Start-Process -WorkingDirectory /ffi_build -FilePath $env:CC -Wait -NoNewWindow -ArgumentList "/LD", "/MD", "/std:c++14", "wildland_wrap.cxx", "ffi_cxx.rs.cc", "/link", "wildland_admin_manager.lib", "ws2_32.lib", "bcrypt.lib", "userenv.lib", "advapi32.lib", "/out:/bindings_test/Wildland.dll"

Write-Host "---------- BUILD /bindings_test/Wildland.Cargo.dll ----------"
Start-Process -FilePath $env:CSHARP_COMPILER -Wait -NoNewWindow -ArgumentList "/target:library", "/out:/bindings_test/Wildland.Cargo.dll", "/bindings/*.cs"

Write-Host "---------- COPY $env:PROJECT_ROOT/tests/ffi/test.cs -> /bindings_test/main.cs ----------"
cp $env:PROJECT_ROOT/tests/ffi/test.cs /bindings_test/main.cs

Set-Location /bindings_test

Write-Host "---------- BUILD /bindings_test/main.exe ----------"
Start-Process -FilePath $env:CSHARP_COMPILER -Wait -NoNewWindow -ArgumentList "/reference:Wildland.Cargo.dll", "/out:main.exe", "main.cs"

Write-Host "---------- RUN TEST /bindings_test/main.exe ----------"
Start-Process .\main.exe -Wait -NoNewWindow

Write-Host "---------- COPY artifacts to build directory ----------"
Set-Location $env:PROJECT_ROOT
mkdir -p build
mkdir -p build/lib
mkdir -p build/src
mv $env:CXX_LIB ./build/lib/
mv /bindings/* ./build/src/
