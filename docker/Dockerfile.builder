FROM debian:bullseye-slim
ARG DEBIAN_FRONTEND=noninteractive


RUN apt-get -qy update \
    && apt-get install -y \
		curl \
    && curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get -qy update \
    && apt-get install -y \
		g++ \
		swig \
		nodejs \
		mono-mcs \
		openjdk-17-jdk-headless \
		python3-dev \
		ca-certificates \
		file \
		nano \
		less \
		mingw-w64 \
		libssl-dev \
		git \
		jq \
		ruby \
	&& apt-get clean autoclean \
	&& apt-get autoremove --yes \
	&& rm -rf /var/lib/{apt,dpkg,cache,log}/

ENV PATH=/root/.cargo/bin:$PATH

RUN git clone https://github.com/emscripten-core/emsdk.git \
    && cd emsdk \
    && ./emsdk install latest

RUN mkdir -p \
    /bindings \
    /bindings_test \
    /ffi_build \
    /ffi_tests \
    /scripts \
    /app/crates/

RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y

RUN cargo install \
	cargo-release \
	cargo-workspaces

RUN rustup target add wasm32-unknown-emscripten

WORKDIR /
