FROM swift:5.6.1-focal

# Run with:
#
# docker-compose -f wildland-admin-manager/docker/docker-compose.yml run --rm wildland-sdk-swift


RUN apt-get -qy update && apt-get install -y curl g++ cargo \
    && mkdir -p wildland-core/wildland-admin-manager /root/.cargo \
    && printf '[registries]\nwl-dev = { index = "https://crates.wildland.dev/git/index" }\n' > /root/.cargo/config.toml
# need to upgrade Rust toolchain to be able to compile the latest swift-bridge (trigerred by `cargo build`)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y


# Build only dependencies to cache them in docker layer
WORKDIR /wildland-core/crates/
RUN for crate in \
    wildland-admin-manager \
    wildland-corex \
    wildland-catlib \
    wildland-crypto \
    wildland-dfs\
    wildland-wallet\
    ; do cargo new --lib $crate; done
COPY Cargo.toml /wildland-core/
COPY Cargo.lock /wildland-core/
COPY crates/wildland-admin-manager/Cargo.toml wildland-admin-manager/
COPY crates/wildland-admin-manager/build.rs wildland-admin-manager/
COPY crates/wildland-corex/Cargo.toml wildland-corex/
COPY crates/wildland-catlib/Cargo.toml wildland-catlib/
COPY crates/wildland-crypto/Cargo.toml wildland-crypto/
COPY crates/wildland-dfs/Cargo.toml wildland-dfs/
COPY crates/wildland-wallet/Cargo.toml wildland-wallet/

WORKDIR /wildland-core/
RUN ls ./crates
RUN cargo build --package wildland-admin-manager


# Actual build
COPY . /wildland-core
WORKDIR /wildland-core/crates/wildland-admin-manager
RUN mkdir -p wildland_swift \
    && . $HOME/.cargo/env \
    && cargo clean \
    && SWIFT_BRIDGE_OUT_DIR="$PWD/wildland_swift" cargo build --features "bindings" \
    && cp test/ffi/test.swift wildland_swift/main.swift \
    && swiftc -L ../../target/debug -lwildland_admin_manager -lstdc++ \
    -I wildland_swift -import-objc-header \
    swift_header.h \
    ./wildland_swift/SwiftBridgeCore.swift \
    ./wildland_swift/wildland/wildland.swift \
    wildland_swift/main.swift \
    -o wildland_swift/swift_app

# Run the test. If everything was completed successfully, `echo $?` should return `0` exit code.

ENTRYPOINT ["./wildland_swift/swift_app"]
