#!/usr/bin/env bash
set -euo pipefail

echo "Building wasmCloud rate limiter components..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

build_component() {
    local component=$1
    echo -e "${BLUE}Building ${component}...${NC}"

    cargo build --package ${component} --target wasm32-wasip1 --release

    # Note: Signing with wash is optional for local development
    # For production, use: wasm-tools component new target/wasm32-wasip1/release/${component//-/_}.wasm
    if command -v wasm-tools &> /dev/null; then
        echo -e "${BLUE}Creating component from ${component}...${NC}"
        wasm-tools component new target/wasm32-wasip1/release/${component//-/_}.wasm \
            -o target/wasm32-wasip1/release/${component//-/_}_component.wasm 2>/dev/null || true
    fi

    echo -e "${GREEN}✓ ${component} built${NC}"
}

# Ensure wasm32-wasip1 target is installed
rustup target add wasm32-wasip1

# Build all rate-limit components
build_component "token-bucket"
build_component "leaky-bucket"
build_component "sliding-window"

# Build auth component-model crates targeting wasm32-wasip2 (produces native
# Component Model wasm, no preview1 adapter needed).
echo -e "${BLUE}Building auth components (wasm32-wasip2)...${NC}"
rustup target add wasm32-wasip2
for pkg in auth-session auth-jwt auth-oauth; do
    echo -e "${BLUE}Building ${pkg} (wasm32-wasip2)...${NC}"
    cargo build --package "${pkg}" --target wasm32-wasip2 --release
    echo -e "${GREEN}✓ ${pkg} built${NC}"
done

echo -e "${GREEN}All components built successfully!${NC}"
echo ""
echo "Components are located in: target/wasm32-wasip1/release/"
echo ""
echo "Next steps:"
echo "  1. Start wasmCloud: wash up"
echo "  2. Deploy app: wash app deploy wadm/token-bucket.yaml"
echo "  3. Run tests: cargo test --package e2e-tests -- --ignored"
