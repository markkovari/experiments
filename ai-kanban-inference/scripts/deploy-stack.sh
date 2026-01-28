#!/bin/bash
#
# Deploy all Nomad jobs to the cluster
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

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
NOMAD_DIR="$PROJECT_DIR/nomad"

# Check if Nomad is available
if ! command -v nomad &> /dev/null; then
    log_error "Nomad not found. Please install Nomad first."
    exit 1
fi

# Check Nomad connectivity
log_info "Checking Nomad server connectivity..."
if ! nomad server members &> /dev/null; then
    log_error "Cannot connect to Nomad server."
    log_error "Make sure the Nomad server is running and NOMAD_ADDR is set correctly."
    log_info "Try: export NOMAD_ADDR=http://<rpi5-tailscale-ip>:4646"
    exit 1
fi

log_info "Connected to Nomad cluster"
nomad server members

echo ""
log_info "Checking Nomad client nodes..."
nomad node status

echo ""

# Deploy Ollama registration first (so it's available for other services)
log_info "Deploying Ollama service registration..."
if [[ -f "$NOMAD_DIR/ollama-service.nomad" ]]; then
    nomad job plan "$NOMAD_DIR/ollama-service.nomad" || true
    nomad job run "$NOMAD_DIR/ollama-service.nomad"
    log_info "Ollama service job submitted"
else
    log_warn "ollama-service.nomad not found, skipping"
fi

# Wait for Ollama registration
log_info "Waiting for Ollama service to register..."
sleep 5

# Deploy Claude-Flow
log_info "Deploying Claude-Flow..."
if [[ -f "$NOMAD_DIR/claude-flow.nomad" ]]; then
    nomad job plan "$NOMAD_DIR/claude-flow.nomad" || true
    nomad job run "$NOMAD_DIR/claude-flow.nomad"
    log_info "Claude-Flow job submitted"
else
    log_warn "claude-flow.nomad not found, skipping"
fi

# Wait for services to start
log_info "Waiting for services to become healthy..."
sleep 10

# Check job status
echo ""
echo "=========================================="
echo -e "${GREEN}Deployment Status${NC}"
echo "=========================================="
echo ""

log_info "Nomad Jobs:"
nomad job status

echo ""

# Check Consul services if available
if command -v consul &> /dev/null; then
    log_info "Consul Services:"
    consul catalog services 2>/dev/null || log_warn "Could not query Consul services"
fi

echo ""

# Get service URLs
log_info "Retrieving service endpoints..."

# Try to get Claude-Flow URL from Consul
CLAUDE_FLOW_URL=""
if command -v consul &> /dev/null; then
    CLAUDE_FLOW_HOST=$(consul catalog nodes -service=claude-flow 2>/dev/null | tail -n +2 | awk '{print $2}' | head -1)
    if [[ -n "$CLAUDE_FLOW_HOST" ]]; then
        CLAUDE_FLOW_URL="http://${CLAUDE_FLOW_HOST}:8080"
    fi
fi

# Try to get Ollama URL from Consul
OLLAMA_URL=""
if command -v consul &> /dev/null; then
    OLLAMA_HOST=$(consul catalog nodes -service=ollama 2>/dev/null | tail -n +2 | awk '{print $2}' | head -1)
    if [[ -n "$OLLAMA_HOST" ]]; then
        OLLAMA_URL="http://${OLLAMA_HOST}:11434"
    fi
fi

echo ""
echo "=========================================="
echo -e "${GREEN}Stack Deployed Successfully!${NC}"
echo "=========================================="
echo ""
echo "Service Endpoints:"
if [[ -n "$CLAUDE_FLOW_URL" ]]; then
    echo "  Claude-Flow UI: $CLAUDE_FLOW_URL"
else
    echo "  Claude-Flow UI: (check nomad job status claude-flow)"
fi
if [[ -n "$OLLAMA_URL" ]]; then
    echo "  Ollama API:     $OLLAMA_URL"
else
    echo "  Ollama API:     (check nomad job status ollama-registration)"
fi
echo ""
echo "Management UIs:"
echo "  Nomad:  http://<control-plane>:4646"
echo "  Consul: http://<control-plane>:8500"
echo ""
echo "Useful commands:"
echo "  nomad job status                 - List all jobs"
echo "  nomad alloc logs <alloc-id>      - View job logs"
echo "  consul catalog services          - List registered services"
echo "  consul members                   - View cluster members"
echo "=========================================="
