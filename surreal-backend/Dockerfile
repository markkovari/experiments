# Build stage - Use nightly for edition2024 support
FROM rustlang/rust:nightly-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY tests ./tests

# Build release binary with optimizations
RUN cargo build --release --bin surreal-backend

# Runtime stage - Use minimal Debian trixie
FROM debian:trixie-slim

# Install only essential runtime dependencies
RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    ca-certificates \
    libssl3t64 \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /usr/share/doc/* \
    && rm -rf /usr/share/man/* \
    && rm -rf /var/cache/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/surreal-backend /usr/local/bin/surreal-backend

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Run the application
CMD ["surreal-backend", "serve"]
