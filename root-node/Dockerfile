FROM ubuntu:24.04 AS builder

WORKDIR /substrate
COPY . .

# Install Rust and build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
	clang \
    build-essential \
    pkg-config \
    libssl-dev \
	protobuf-compiler \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . $HOME/.cargo/env \
	&& rustup toolchain install 1.81.0 \
	&& rustup default 1.81.0 \
	&& rustup target add wasm32-unknown-unknown --toolchain 1.81.0 \
	&& rustup target add wasm32-unknown-unknown --toolchain 1.81.0-aarch64-unknown-linux-gnu \
	&& rustup component add rust-src --toolchain 1.81.0-aarch64-unknown-linux-gnu \
	&& cargo build --release

FROM ubuntu:24.04

ENV RUST_BACKTRACE 1

WORKDIR /substrate

# Copy built binary from builder stage
COPY --from=builder /substrate/target/release/node-template /substrate/target/release/
COPY ./scripts/generate_single_node_spec.sh /substrate/
COPY ./scripts/entrypoint.sh /substrate/

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    autoconf \
    automake \
    libtool \
    curl \
    make \
    g++ \
    unzip \
    clang \
    protobuf-compiler

# Create user and set up directories
RUN useradd -m -u 1001 -U -s /bin/sh -d /substrate substrate && \
    mkdir -p /data /substrate/.local/share/substrate /substrate/chains/local_testnet/db && \
    chown -R substrate:substrate /data /substrate && \
    chmod +x ./entrypoint.sh && \
    chmod +x ./generate_single_node_spec.sh && \
    ./generate_single_node_spec.sh

USER substrate
EXPOSE 30333 9933 9944 9945 9615
VOLUME ["/data"]
ENTRYPOINT ["./entrypoint.sh"]