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
$env:CARGO_LIB_PROJECT_DIR = "$env:PROJECT_ROOT/crates/wildland-cargo-lib"
$env:FFI_BUILD_DIR = "/ffi_build"
$env:TARGET_DIR = "$env:PROJECT_ROOT/target/$env:BUILD_TARGET"
$env:CXX_LIB = "$env:TARGET_DIR/debug/wildland_cargo_lib.lib"
$env:SWIFT_BRIDGE_OUT_DIR = $env:TARGET_DIR

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
$p = (Start-Process -PassThru -FilePath "rustup" -Wait -NoNewWindow -ArgumentList "target add $env:BUILD_TARGET")
if ($p.ExitCode -ne 0) {
    exit $p.ExitCode
}

Write-Host "---------- Cargo Build ----------"
$p = (Start-Process -PassThru -FilePath "cargo" -Wait -NoNewWindow -ArgumentList "build --features bindings --target $env:BUILD_TARGET")
if ($p.ExitCode -ne 0) {
    exit $p.ExitCode
}

Write-Host "---------- Cargo Build artifacts ----------"
cp "$env:SWIFT_BRIDGE_OUT_DIR/SwiftBridgeCore.h" "$env:FFI_BUILD_DIR"
cp "$env:CARGO_LIB_PROJECT_DIR/_generated_cpp/ffi_cxx.h" "$env:FFI_BUILD_DIR"
cp "$env:CARGO_LIB_PROJECT_DIR/_generated_cpp/ffi_swig.i" "$env:FFI_BUILD_DIR"
cp "$env:CARGO_LIB_PROJECT_DIR/_generated_swift/ffi_swift/ffi_swift.h" "$env:FFI_BUILD_DIR"
cp "$env:CXX_LIB" "$env:FFI_BUILD_DIR"

Write-Host "---------- Create SWIG-Generated C# Bindings ----------"
$p = (Start-Process -PassThru -WorkingDirectory /ffi_build -FilePath "swig" -Wait -NoNewWindow -ArgumentList "-dllimport $env:DLLIMPORT -csharp -module wildland -c++ -w'516,503,476,302,124' -outdir /bindings ffi_swig.i")
if ($p.ExitCode -ne 0) {
    exit $p.ExitCode
}

Write-Host "---------- BUILD /bindings_test/Wildland.dll ----------"
$p = (Start-Process -PassThru -WorkingDirectory /ffi_build -FilePath $env:CC -Wait -NoNewWindow -ArgumentList "/LD", "/MD", "/std:c++14", "ffi_swig_wrap.cxx", "/link", "wildland_cargo_lib.lib", "ws2_32.lib", "bcrypt.lib", "userenv.lib", "advapi32.lib", "shell32.lib", "Ole32.lib", "/out:/bindings_test/Wildland.dll")
if ($p.ExitCode -ne 0) {
    exit $p.ExitCode
}

Write-Host "---------- BUILD /bindings_test/Wildland.Cargo.dll ----------"
$p = (Start-Process -PassThru -FilePath $env:CSHARP_COMPILER -Wait -NoNewWindow -ArgumentList "/target:library", "/out:/bindings_test/Wildland.Cargo.dll", "/bindings/*.cs")
if ($p.ExitCode -ne 0) {
    exit $p.ExitCode
}

Write-Host "---------- COPY $env:PROJECT_ROOT/tests/ffi/test.cs -> /bindings_test/main.cs ----------"
cp $env:PROJECT_ROOT/tests/ffi/test.cs /bindings_test/main.cs

Set-Location /bindings_test

Write-Host "---------- BUILD /bindings_test/main.exe ----------"
$p = (Start-Process -PassThru -FilePath $env:CSHARP_COMPILER -Wait -NoNewWindow -ArgumentList "/reference:Wildland.Cargo.dll", "/out:main.exe", "main.cs")
if ($p.ExitCode -ne 0) {
    exit $p.ExitCode
}

Write-Host "---------- RUN TEST /bindings_test/main.exe ----------"
$p = (Start-Process -PassThru .\main.exe -Wait -NoNewWindow)
if ($p.ExitCode -ne 0) {
    exit $p.ExitCode
}

Write-Host "---------- COPY artifacts to build directory ----------"
Set-Location $env:PROJECT_ROOT
mkdir -p build
mkdir -p build/lib
mkdir -p build/src
mv $env:CXX_LIB ./build/lib/
mv /bindings/* ./build/src/
