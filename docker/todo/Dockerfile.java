FROM debian:testing-slim

# Run with:
#
# docker-compose -f wildland-cargo-lib/docker/docker-compose.yml run --rm wildland-sdk-java

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get -qy update && apt-get install -y swig openjdk-17-jdk-headless g++

ENV CARGO_LIB_PATH=/wildland-core/crates/wildland-cargo-lib
ENV TARGET=/wildland-core/target/debug/
ENV CC=g++
ENV JDK_INC_DIR=/usr/lib/jvm/java-17-openjdk-amd64/include

RUN mkdir -p /wildland-core/
WORKDIR /wildland-core

# Copy from base image instead of building new image on top of it to avoid reinstalling packages after source code changes
COPY --from=wildland-sdk-base ${CARGO_LIB_PATH}/_generated_cpp ./_generated_cpp
COPY --from=wildland-sdk-base ${CARGO_LIB_PATH}/_generated_swift ./_generated_swift
COPY --from=wildland-sdk-base ${TARGET}/libwildland_cargo_lib.a ./lib/

RUN mkdir -p _generated_java \
    && swig -java -c++  -module wildland -outdir _generated_java _generated_cpp/ffi_swig.i \
    && ${CC} -fpermissive -shared -fPIC --std=c++14 -w \
    _generated_cpp/ffi_swig_wrap.cxx \
    -Llib \
    -lwildland_cargo_lib \
    -I${JDK_INC_DIR} \
    -I${JDK_INC_DIR}/linux \
    -I_generated_cpp \
    -I_generated_swift \
    -I_generated_swift/ffi_swift \
    -o _generated_java/libwildland.so

COPY test/ffi/test.java ./_generated_java/
RUN cd _generated_java \
    && javac test.java

# Run the test. If everything was completed successfully, `echo $?` should return `0` exit code.

CMD ["java", "-cp", "_generated_java", "-Djava.library.path=./_generated_java", "main"]
