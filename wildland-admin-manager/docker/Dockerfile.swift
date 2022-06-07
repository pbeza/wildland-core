FROM swift:5.6.1-focal

# Run with:
#
# docker-compose -f wildland-admin-manager/docker/docker-compose.yml run --rm wildland-sdk-swift

RUN mkdir -p wildland-core/wildland-admin-manager /root/.cargo
COPY . wildland-core
RUN apt-get -qy update && apt-get install -y curl g++
WORKDIR /wildland-core/wildland-admin-manager
# need to upgrade Rust toolchain to be able to compile the latest swift-bridge (trigerred by `cargo build`)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y \
    && printf '[registries]\nwl-dev = { index = "https://crates.wildland.dev/git/index" }\n' > /root/.cargo/config.toml \
    && mkdir -p wildland_swift \
    && . $HOME/.cargo/env \
    && cargo clean \
    && SWIFT_BRIDGE_OUT_DIR="$PWD/wildland_swift" cargo build --features "bindings,mocks" \
    && cp test/ffi/test.swift wildland_swift/main.swift \
    && swiftc -L ../target/debug -lwildland_admin_manager -lstdc++ \
    -I wildland_swift -import-objc-header \
    swift_header.h \
    ./wildland_swift/SwiftBridgeCore.swift \
    ./wildland_swift/wildland/wildland.swift \
    wildland_swift/main.swift \
    -o wildland_swift/swift_app

# Run the test. If everything was completed successfully, `echo $?` should return `0` exit code.

ENTRYPOINT ["./wildland_swift/swift_app"]
