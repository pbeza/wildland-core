FROM swift:5.6.1-focal

# Run with:
#
# docker-compose -f wildland-admin-manager/docker/docker-compose.yml run --rm wildland-sdk-swift

ENV ADMIN_MANAGER_PATH=/wildland-core/crates/wildland-admin-manager
ENV TARGET=/wildland-core/target/debug/
ENV CC=g++
ENV CSHARP_COMPILER=mcs

RUN apt-get -qy update && apt-get install -y swig mono-mcs g++

RUN mkdir -p /wildland-core/
WORKDIR /wildland-core

# Copy from base image instead of building new image on top of it to avoid reinstalling packages after source code changes
COPY --from=wildland-sdk-base ${ADMIN_MANAGER_PATH}/_generated_cpp ./_generated_cpp
COPY --from=wildland-sdk-base ${ADMIN_MANAGER_PATH}/_generated_swift ./_generated_swift
COPY --from=wildland-sdk-base ${ADMIN_MANAGER_PATH}/swift_header.h .
COPY --from=wildland-sdk-base ${TARGET}/libwildland_admin_manager.a ./lib/


COPY test/ffi/test.swift _generated_swift/main.swift
RUN swiftc -L lib -lwildland_admin_manager -lstdc++ \
        -I _generated_swift -import-objc-header \
        swift_header.h \
        ./_generated_swift/SwiftBridgeCore.swift \
        ./_generated_swift/ffi_swift/ffi_swift.swift \
        _generated_swift/main.swift \
        -o _generated_swift/swift_app

# Run the test. If everything was completed successfully, `echo $?` should return `0` exit code.

CMD ["./_generated_swift/swift_app"]
