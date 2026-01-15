# Build stage - Use nightly for edition2024 support
FROM rustlang/rust:nightly-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Set RUSTFLAGS for optimization
ENV RUSTFLAGS="-C target-cpu=native"

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY tests ./tests

# Build release binary
# Using nightly Rust to support edition2024 required by async-graphql
RUN cargo build --release --bin surreal-backend

# Runtime stage - Use same Debian version as builder for GLIBC compatibility
FROM debian:trixie-slim AS runtime

# Install runtime dependencies including curl for healthcheck
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3t64 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/surreal-backend /usr/local/bin/surreal-backend

# Create data directory with proper permissions
RUN mkdir -p /app/data && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["surreal-backend", "serve"]
