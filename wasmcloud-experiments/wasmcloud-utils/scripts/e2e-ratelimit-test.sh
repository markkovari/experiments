#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cleanup() {
    log_info "Cleaning up..."

    # Method 1: Drain all hosts first (most effective)
    log_info "Draining all wasmCloud hosts..."
    wash drain all --ctl-port 4222 2>/dev/null || true
    sleep 2

    # Method 2: Undeploy all apps - wait for each to complete
    log_info "Undeploying applications..."
    local apps=$(wash app list --output json 2>/dev/null | jq -r '.[].name' 2>/dev/null || echo "")
    if [ -n "$apps" ]; then
        while IFS= read -r app; do
            if [ -n "$app" ] && [ "$app" != "null" ]; then
                log_info "Undeploying $app..."
                wash app undeploy "$app" 2>/dev/null || true
                # Wait for undeploy to complete
                local retries=0
                while [ $retries -lt 10 ]; do
                    if ! wash app list --output json 2>/dev/null | jq -e ".[] | select(.name==\"$app\")" >/dev/null 2>&1; then
                        break
                    fi
                    sleep 1
                    retries=$((retries + 1))
                done
            fi
        done <<< "$apps"
    fi

    # Method 3: Delete NATS WADM manifests KV bucket (prevents zombie deployments)
    log_info "Deleting NATS WADM manifests..."
    nats kv rm wadm_manifests --server=nats://127.0.0.1:4222 --force 2>/dev/null || true
    sleep 1

    # Stop wasmCloud host
    log_info "Stopping wasmCloud host..."
    wash down 2>/dev/null || true

    # Kill any lingering processes
    pkill -9 wash 2>/dev/null || true
    pkill -9 wasmcloud 2>/dev/null || true
    pkill -9 nats-server 2>/dev/null || true

    # Wait a bit for cleanup
    sleep 3

    log_success "Cleanup complete"
}

check_dependencies() {
    log_info "Checking dependencies..."

    local missing_deps=()

    if ! command -v wash &> /dev/null; then
        missing_deps+=("wash")
    fi

    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo")
    fi

    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Install wash: curl -s https://wasmcloud.com/install.sh | bash"
        log_info "Install jq: brew install jq (macOS) or apt-get install jq (Linux)"
        exit 1
    fi

    log_success "All dependencies found"
}

build_components() {
    log_info "Checking if components need to be built..."

    local needs_build=false

    if [ ! -f "$PROJECT_ROOT/target/wasm32-wasip1/release/token_bucket.wasm" ] || \
       [ ! -f "$PROJECT_ROOT/target/wasm32-wasip1/release/leaky_bucket.wasm" ] || \
       [ ! -f "$PROJECT_ROOT/target/wasm32-wasip1/release/sliding_window.wasm" ]; then
        needs_build=true
    fi

    if [ "$needs_build" = true ]; then
        log_info "Building components..."
        cd "$PROJECT_ROOT"
        ./scripts/build-components.sh
        log_success "Components built"
    else
        log_success "Components already built, skipping build"
    fi
}

start_wasmcloud() {
    log_info "Starting wasmCloud host..."

    # Start wasmCloud in detached mode
    wash up --detached

    # Wait for host to be ready
    log_info "Waiting for wasmCloud host to be ready..."
    local retries=0
    local max_retries=30

    while [ $retries -lt $max_retries ]; do
        if wash get hosts 2>/dev/null | grep -q "Host ID"; then
            log_success "wasmCloud host is ready"
            return 0
        fi
        sleep 1
        retries=$((retries + 1))
    done

    log_error "wasmCloud host failed to start within ${max_retries} seconds"
    return 1
}

deploy_application() {
    local app_name=$1
    local manifest=$2

    log_info "Deploying ${app_name}..."

    cd "$PROJECT_ROOT"
    wash app deploy "$manifest"

    # Wait for deployment to be ready
    log_info "Waiting for ${app_name} to be deployed..."
    local retries=0
    local max_retries=30

    while [ $retries -lt $max_retries ]; do
        local status=$(wash app list --output json 2>/dev/null | jq -r ".[] | select(.name==\"${app_name}\") | .status" 2>/dev/null || echo "")

        if [ "$status" = "Deployed" ]; then
            log_success "${app_name} deployed successfully"
            return 0
        elif [ "$status" = "Failed" ]; then
            log_error "${app_name} deployment failed"
            wash app status "${app_name}"
            return 1
        fi

        sleep 2
        retries=$((retries + 1))
    done

    log_warning "${app_name} deployment status unclear after ${max_retries} attempts"
    wash app status "${app_name}" || true
    return 0
}

run_tests() {
    log_info "Running e2e tests..."

    cd "$PROJECT_ROOT"

    # Run tests (timeout not available on macOS by default)
    if cargo test --package e2e-tests -- --ignored test_ratelimit_ --test-threads=1 --nocapture; then
        log_success "All e2e tests passed"
        return 0
    else
        log_error "E2E tests failed"
        return 1
    fi
}

main() {
    echo "======================================"
    echo "  wasmCloud Rate Limiter E2E Tests"
    echo "======================================"
    echo ""

    # Trap to ensure cleanup on exit
    trap cleanup EXIT

    # Check dependencies
    check_dependencies

    # Clean up any existing instances
    log_info "Cleaning up any existing wasmCloud instances..."
    cleanup

    # Build components if needed
    build_components

    # Start wasmCloud
    start_wasmcloud || exit 1

    # Give it a moment to stabilize
    sleep 3

    # Deploy token-bucket application
    deploy_application "token-bucket-ratelimiter" "wadm/token-bucket.yaml" || {
        log_error "Failed to deploy token-bucket"
        exit 1
    }

    # Wait a bit before deploying next
    sleep 2

    # Deploy leaky-bucket application
    deploy_application "leaky-bucket-ratelimiter" "wadm/leaky-bucket.yaml" || {
        log_error "Failed to deploy leaky-bucket"
        exit 1
    }

    # Wait a bit before deploying next
    sleep 2

    # Deploy sliding-window application
    deploy_application "sliding-window-ratelimiter" "wadm/sliding-window.yaml" || {
        log_error "Failed to deploy sliding-window"
        exit 1
    }

    # Give deployments time to stabilize
    sleep 5

    # Show deployment status
    log_info "Current deployment status:"
    wash app list
    echo ""

    # Run e2e tests
    if run_tests; then
        log_success "✅ All tests passed!"
        exit 0
    else
        log_error "❌ Tests failed"
        exit 1
    fi
}

# Run main function
main "$@"
