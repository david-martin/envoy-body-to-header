# Dockerfile for building Envoy with the body-to-header dynamic module

##### Build the Rust library #####

# Use https://github.com/rust-cross/cargo-zigbuild to cross-compile the Rust library for both x86_64 and aarch64 architectures.
FROM --platform=$BUILDPLATFORM ghcr.io/rust-cross/cargo-zigbuild:0.19.8 AS rust_builder

WORKDIR /build

# bindgen requires libclang-dev.
RUN apt update && apt install -y clang

# Fetch the dependencies first to leverage Docker cache.
COPY ./Cargo.toml ./Cargo.lock ./
RUN mkdir src && echo "" > src/lib.rs
RUN cargo fetch
RUN rm -rf src

# Then copy the rest of the source code and build the library.
COPY ./.cargo ./.cargo
COPY ./src ./src
RUN cargo zigbuild --target aarch64-unknown-linux-gnu
RUN cargo zigbuild --target x86_64-unknown-linux-gnu

RUN cp /build/target/aarch64-unknown-linux-gnu/debug/libbody_to_header_module.so /build/arm64_libbody_to_header_module.so
RUN cp /build/target/x86_64-unknown-linux-gnu/debug/libbody_to_header_module.so /build/amd64_libbody_to_header_module.so

##### Build the final image #####
FROM envoyproxy/envoy-dev:73fe00fc139fd5053f4c4a5d66569cc254449896 AS envoy
ARG TARGETARCH
ENV ENVOY_DYNAMIC_MODULES_SEARCH_PATH=/usr/local/lib
COPY --from=rust_builder /build/${TARGETARCH}_libbody_to_header_module.so /usr/local/lib/libbody_to_header_module.so