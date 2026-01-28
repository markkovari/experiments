#!/bin/bash
#
# Cross-platform Tailscale installer
# Supports: Linux ARM64 (Raspberry Pi), macOS ARM64 (Apple Silicon)
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Default values
TAILSCALE_KEY=""
HOSTNAME_SUFFIX=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --authkey=*|--tailscale-key=*)
            TAILSCALE_KEY="${1#*=}"
            shift
            ;;
        --hostname=*)
            HOSTNAME_SUFFIX="${1#*=}"
            shift
            ;;
        --help)
            echo "Usage: $0 --authkey=tskey-auth-xxx [--hostname=my-device]"
            echo ""
            echo "Options:"
            echo "  --authkey     Tailscale auth key (required)"
            echo "  --hostname    Custom hostname for this device"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$TAILSCALE_KEY" ]]; then
    log_error "Tailscale auth key required. Use --authkey=tskey-auth-xxx"
    log_info "Generate a key at: https://login.tailscale.com/admin/settings/keys"
    exit 1
fi

# Detect OS
OS=$(uname -s)
ARCH=$(uname -m)

log_info "Detected OS: $OS, Architecture: $ARCH"

install_linux() {
    log_info "Installing Tailscale on Linux..."

    # Use official install script
    if ! command -v tailscale &> /dev/null; then
        curl -fsSL https://tailscale.com/install.sh | sh
    else
        log_info "Tailscale already installed"
    fi

    # Enable and start tailscaled service
    sudo systemctl enable tailscaled
    sudo systemctl start tailscaled
    sleep 2
}

install_macos() {
    log_info "Installing Tailscale on macOS..."

    # Check for Homebrew
    if ! command -v brew &> /dev/null; then
        log_error "Homebrew not found. Please install Homebrew first:"
        log_error '  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'
        exit 1
    fi

    # Install via Homebrew
    if ! command -v tailscale &> /dev/null; then
        brew install tailscale
    else
        log_info "Tailscale already installed"
    fi

    # Start the service
    brew services start tailscale 2>/dev/null || true
    sleep 2
}

# Install based on OS
case "$OS" in
    Linux)
        install_linux
        ;;
    Darwin)
        install_macos
        ;;
    *)
        log_error "Unsupported operating system: $OS"
        exit 1
        ;;
esac

# Authenticate with Tailscale
log_info "Authenticating with Tailscale..."

AUTH_ARGS="--authkey=$TAILSCALE_KEY"
if [[ -n "$HOSTNAME_SUFFIX" ]]; then
    AUTH_ARGS="$AUTH_ARGS --hostname=$HOSTNAME_SUFFIX"
fi

if [[ "$OS" == "Darwin" ]]; then
    sudo tailscale up $AUTH_ARGS
else
    tailscale up $AUTH_ARGS
fi

# Wait for connection
log_info "Waiting for Tailscale to connect..."
sleep 5

# Get Tailscale info
TAILSCALE_IP=$(tailscale ip -4 2>/dev/null || echo "Not available")
TAILSCALE_STATUS=$(tailscale status --json 2>/dev/null | jq -r '.BackendState' || echo "Unknown")
TAILSCALE_HOSTNAME=$(tailscale status --json 2>/dev/null | jq -r '.Self.DNSName' | sed 's/\.$//' || echo "Not available")

# Output results
echo ""
echo "=========================================="
echo -e "${GREEN}Tailscale Installation Complete!${NC}"
echo "=========================================="
echo ""
echo "Status: ${TAILSCALE_STATUS}"
echo "IPv4:   ${TAILSCALE_IP}"
echo "DNS:    ${TAILSCALE_HOSTNAME}"
echo ""
echo "Useful commands:"
echo "  tailscale status    - Show connected devices"
echo "  tailscale ping <ip> - Test connectivity"
echo "  tailscale netcheck  - Network diagnostics"
echo ""
echo "Admin console: https://login.tailscale.com/admin"
echo "=========================================="
