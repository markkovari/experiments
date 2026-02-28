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

log_info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1"; }

# ---------------------------------------------------------------------------
# Teardown — called on EXIT via trap
# ---------------------------------------------------------------------------
cleanup() {
    log_info "Tearing down auth test environment..."

    log_info "Draining all wasmCloud hosts..."
    wash drain all --ctl-port 4222 2>/dev/null || true
    sleep 2

    log_info "Undeploying auth applications..."
    local apps
    apps=$(wash app list --output json 2>/dev/null | jq -r '.[].name' 2>/dev/null || true)
    if [ -n "$apps" ]; then
        while IFS= read -r app; do
            if [ -n "$app" ] && [ "$app" != "null" ]; then
                log_info "  Undeploying $app..."
                wash app undeploy "$app" 2>/dev/null || true
                local retries=0
                while [ $retries -lt 10 ]; do
                    if ! wash app list --output json 2>/dev/null \
                            | jq -e ".[] | select(.name==\"$app\")" >/dev/null 2>&1; then
                        break
                    fi
                    sleep 1
                    retries=$((retries + 1))
                done
            fi
        done <<< "$apps"
    fi

    log_info "Deleting NATS WADM manifests bucket..."
    nats kv rm wadm_manifests --server=nats://127.0.0.1:4222 --force 2>/dev/null || true
    sleep 1

    log_info "Stopping wasmCloud host..."
    wash down 2>/dev/null || true
    pkill -9 wash       2>/dev/null || true
    pkill -9 wasmcloud  2>/dev/null || true
    pkill -9 nats-server 2>/dev/null || true
    sleep 3

    log_success "Teardown complete"
}

# ---------------------------------------------------------------------------
# Dependency check
# ---------------------------------------------------------------------------
check_dependencies() {
    log_info "Checking dependencies..."
    local missing=()
    command -v wash  &>/dev/null || missing+=("wash")
    command -v cargo &>/dev/null || missing+=("cargo")
    command -v jq    &>/dev/null || missing+=("jq")

    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing[*]}"
        log_info "Install wash: curl -s https://wasmcloud.com/install.sh | bash"
        log_info "Install jq:   brew install jq  (macOS)  |  apt-get install jq  (Linux)"
        exit 1
    fi
    log_success "All dependencies found"
}

# ---------------------------------------------------------------------------
# Build auth cdylib components
# ---------------------------------------------------------------------------
build_auth_components() {
    log_info "Checking if auth components need to be built..."

    # auth-session/jwt/oauth target wasm32-wasip2 (component model, no adapter needed)
    local session="$PROJECT_ROOT/target/wasm32-wasip2/release/auth_session.wasm"
    local jwt="$PROJECT_ROOT/target/wasm32-wasip2/release/auth_jwt.wasm"
    local oauth="$PROJECT_ROOT/target/wasm32-wasip2/release/auth_oauth.wasm"

    if [ ! -f "$session" ] || [ ! -f "$jwt" ] || [ ! -f "$oauth" ]; then
        log_info "Building auth components (wasm32-wasip2)..."
        cd "$PROJECT_ROOT"
        rustup target add wasm32-wasip2

        for pkg in auth-session auth-jwt auth-oauth; do
            log_info "  Building $pkg..."
            cargo build --package "$pkg" --target wasm32-wasip2 --release
            log_success "  $pkg built"
        done
    else
        log_success "Auth components already built, skipping"
    fi
}

# ---------------------------------------------------------------------------
# Also run unit tests for the core libs before starting infra
# ---------------------------------------------------------------------------
run_unit_tests() {
    log_info "Running auth unit tests (native)..."
    cd "$PROJECT_ROOT"
    if cargo test -p auth-session-core -p auth-jwt-core -p auth-oauth-core --quiet; then
        log_success "Auth unit tests passed"
    else
        log_error "Auth unit tests failed — aborting e2e run"
        exit 1
    fi
}

# ---------------------------------------------------------------------------
# Start wasmCloud host
# ---------------------------------------------------------------------------
start_wasmcloud() {
    log_info "Starting wasmCloud host..."
    wash up --detached

    log_info "Waiting for wasmCloud host to be ready..."
    local retries=0
    while [ $retries -lt 30 ]; do
        if wash get hosts 2>/dev/null | grep -q "Host ID"; then
            log_success "wasmCloud host is ready"
            return 0
        fi
        sleep 1
        retries=$((retries + 1))
    done

    log_error "wasmCloud host failed to start within 30 seconds"
    return 1
}

# ---------------------------------------------------------------------------
# Deploy a WADM application and wait for Deployed status
# ---------------------------------------------------------------------------
deploy_application() {
    local app_name=$1
    local manifest=$2

    log_info "Deploying ${app_name}..."
    cd "$PROJECT_ROOT"
    wash app deploy "$manifest"

    log_info "Waiting for ${app_name} to reach Deployed status..."
    local retries=0
    while [ $retries -lt 30 ]; do
        local status
        status=$(wash app list --output json 2>/dev/null \
            | jq -r ".[] | select(.name==\"${app_name}\") | .status" 2>/dev/null || true)

        if [ "$status" = "Deployed" ]; then
            log_success "${app_name} deployed successfully"
            return 0
        elif [ "$status" = "Failed" ]; then
            log_error "${app_name} deployment failed"
            wash app status "${app_name}" || true
            return 1
        fi

        sleep 2
        retries=$((retries + 1))
    done

    log_warning "${app_name} status unclear after 30 attempts — continuing"
    wash app status "${app_name}" 2>/dev/null || true
    return 0
}

# ---------------------------------------------------------------------------
# Run the Rust e2e tests (both inline logic tests and #[ignore] infra tests)
# ---------------------------------------------------------------------------
run_tests() {
    log_info "Running auth e2e tests (cargo test -- --ignored)..."
    cd "$PROJECT_ROOT"

    if cargo test --package e2e-tests -- --ignored test_auth_ --test-threads=1 --nocapture; then
        log_success "All auth e2e tests passed"
        return 0
    else
        log_error "Auth e2e tests failed"
        return 1
    fi
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    echo "======================================"
    echo "  wasmCloud Auth Component E2E Tests"
    echo "======================================"
    echo ""

    # Register teardown for any exit (success or failure)
    trap cleanup EXIT

    check_dependencies

    # Clean slate — in case a previous run left state behind
    log_info "Cleaning up any existing wasmCloud state..."
    cleanup

    run_unit_tests

    build_auth_components

    start_wasmcloud || exit 1
    sleep 3

    deploy_application "auth-session" "wadm/auth-session.yaml" || {
        log_error "Failed to deploy auth-session"
        exit 1
    }
    sleep 2

    deploy_application "auth-jwt" "wadm/auth-jwt.yaml" || {
        log_error "Failed to deploy auth-jwt"
        exit 1
    }
    sleep 2

    deploy_application "auth-oauth" "wadm/auth-oauth.yaml" || {
        log_error "Failed to deploy auth-oauth"
        exit 1
    }
    sleep 5

    log_info "Current deployment status:"
    wash app list
    echo ""

    if run_tests; then
        log_success "✅ All auth e2e tests passed!"
        exit 0
    else
        log_error "❌ Auth e2e tests failed"
        exit 1
    fi
}

main "$@"
