FROM debian:testing-slim

# Run with:
#
# docker-compose -f wildland-admin-manager/docker/docker-compose.yml run --rm wildland-sdk-java

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get -qy update && apt-get install -y swig openjdk-17-jdk-headless g++

ENV WLTMP=/wildland-core/crates/wildland-admin-manager/_temporary/
ENV TARGET=/wildland-core/target/debug/
ENV CC=g++
ENV JDK_INC_DIR=/usr/lib/jvm/java-17-openjdk-amd64/include

RUN mkdir -p /wildland-core/
WORKDIR /wildland-core

# Copy from base image instead of building new image on top of it to avoid reinstalling packages after source code changes
COPY --from=wildland-sdk-base ${WLTMP} ./cpp/
COPY --from=wildland-sdk-base ${TARGET}/libwildland_admin_manager.a ./lib/
COPY crates/wildland-admin-manager/wildland.i ./

RUN mkdir -p wildland_java \
    && cd cpp \
    && swig -java -c++ -w516,503,476,302 -outdir ../wildland_java ../wildland.i \
    && mv ../wildland_wrap.cxx . \
    && ${CC} -fpermissive -shared -fPIC --std=c++14 -w \
    wildland_wrap.cxx ffi_cxx.rs.cc \
    -L ../lib \
    -lwildland_admin_manager \
    -I${JDK_INC_DIR} \
    -I${JDK_INC_DIR}/linux \
    -o ../wildland_java/libwildland.so

COPY test/ffi/test.java ./wildland_java/
RUN cd wildland_java \
    && javac test.java

# Run the test. If everything was completed successfully, `echo $?` should return `0` exit code.

CMD ["java", "-cp", "wildland_java", "-Djava.library.path=./wildland_java", "main"]
