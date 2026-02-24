#!/bin/bash
# Run tests against the deployed wasmCloud application

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  wasmCloud HTTP KV Counter Test Suite     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""

# Parse arguments
TEST_TYPE="${1:-all}"

# Function to run unit tests
run_unit_tests() {
    echo -e "${YELLOW}📦 Running unit tests...${NC}"
    cargo test --lib
    echo -e "${GREEN}✓ Unit tests passed${NC}"
    echo ""
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${YELLOW}🔗 Running integration tests...${NC}"
    cargo test --test integration_test
    echo -e "${GREEN}✓ Integration tests passed${NC}"
    echo ""
}

# Function to run e2e tests
run_e2e_tests() {
    echo -e "${YELLOW}🌐 Running end-to-end tests...${NC}"

    # Wait for services
    ./scripts/wait-for-ready.sh

    # Wait a bit more to ensure HTTP endpoint is ready
    echo -e "${YELLOW}⏳ Waiting for HTTP endpoint...${NC}"
    MAX_RETRIES=30
    RETRY_COUNT=0
    while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
        if curl -s http://localhost:8080/ > /dev/null 2>&1; then
            echo -e "${GREEN}✓ HTTP endpoint is ready${NC}"
            break
        fi
        RETRY_COUNT=$((RETRY_COUNT + 1))
        if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
            echo -e "${RED}✗ HTTP endpoint not responding${NC}"
            echo "Container logs:"
            docker compose logs wasmcloud
            exit 1
        fi
        sleep 2
    done

    # Run e2e tests
    cargo test --test e2e_test -- --ignored --test-threads=1

    echo -e "${GREEN}✓ End-to-end tests passed${NC}"
    echo ""
}

# Main test execution
case "$TEST_TYPE" in
    unit)
        run_unit_tests
        ;;
    integration)
        run_integration_tests
        ;;
    e2e)
        run_e2e_tests
        ;;
    all)
        run_unit_tests
        run_integration_tests
        run_e2e_tests
        ;;
    *)
        echo -e "${RED}Unknown test type: $TEST_TYPE${NC}"
        echo "Usage: $0 [unit|integration|e2e|all]"
        exit 1
        ;;
esac

echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     All tests passed successfully! 🎉     ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
