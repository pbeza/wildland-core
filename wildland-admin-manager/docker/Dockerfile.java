FROM wildland-sdk-base:latest

# Run with:
#
# docker-compose -f wildland-admin-manager/docker/docker-compose.yml run --rm wildland-sdk-java

ENV CC=g++
ENV JDK_INC_DIR=/usr/lib/jvm/java-17-openjdk-amd64/include

RUN apt-get -qy update && apt-get install -y swig openjdk-17-jdk

RUN mkdir -p wildland_java \
    && cd "${WLTMP}" \
    && swig -java -c++ -w516,503,476,302 -outdir ../wildland_java ../wildland.i \
    && mv ../wildland_wrap.cxx . \
    && ${CC} -fpermissive -shared -fPIC --std=c++14 -w \
    wildland_wrap.cxx ffi_cxx.rs.cc \
    -L../../target/debug \
    -lwildland_admin_manager \
    -I${JDK_INC_DIR} \
    -I${JDK_INC_DIR}/linux \
    -o ../wildland_java/libwildland.so

RUN cp test/ffi/test.java wildland_java/test.java \
    && cd wildland_java \
    && javac test.java

# Run the test. If everything was completed successfully, `echo $?` should return `0` exit code.

ENTRYPOINT ["java", "-cp", "wildland_java", "-Djava.library.path=./wildland_java", "main"]
