# Build stage - Install dependencies and build the component
FROM rust:latest as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install wash CLI via cargo
RUN cargo install wash-cli --locked

# Add wasm32-wasip2 target
RUN rustup target add wasm32-wasip2

# Set working directory
WORKDIR /app

# Copy only dependency files first for better layer caching
COPY Cargo.toml wasmcloud.toml ./
COPY wit/world.wit wit/deps.toml ./wit/

# Download WASI dependencies
RUN mkdir -p wit/deps/http && \
    curl -L https://github.com/WebAssembly/wasi-http/archive/refs/tags/v0.2.0.tar.gz | tar xz -C wit/deps/http --strip-components=1 && \
    mkdir -p wit/deps/keyvalue && \
    curl -L https://github.com/WebAssembly/wasi-keyvalue/archive/refs/heads/main.tar.gz | tar xz -C wit/deps/keyvalue --strip-components=1

# Copy source code
COPY src ./src/

# Build the component
RUN wash build

# Runtime stage - Minimal image with just the built component
FROM alpine:latest

# Install curl for healthchecks
RUN apk add --no-cache curl

WORKDIR /app

# Copy the built component from builder
COPY --from=builder /app/build/*.wasm ./build/
COPY --from=builder /app/wadm.yaml ./

# The component itself doesn't run directly - it's deployed to wasmCloud
# This container is just for building and storing the artifact
CMD ["echo", "Component built successfully. Deploy to wasmCloud using wadm.yaml"]
