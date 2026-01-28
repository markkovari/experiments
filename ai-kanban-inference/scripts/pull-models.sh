#!/bin/bash
#
# Pull all Ollama models for the distributed AI coding agent
# Run this on the MacBook (data-plane) after Ollama is installed
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_progress() { echo -e "${BLUE}[PULL]${NC} $1"; }

# Models to pull - optimized for 96GB RAM M1 MAX
# These models are selected for coding tasks and fit in memory
MODELS=(
    "codellama:34b"           # Code-focused, Meta
    "deepseek-coder:33b"      # Coding tasks, DeepSeek
    "llama3.2:70b"            # General purpose, Meta (if available, fallback to 3.1)
    "qwen2.5-coder:32b"       # Coding, Alibaba
)

# Optional smaller models for faster inference
OPTIONAL_MODELS=(
    "codellama:13b"           # Faster code model
    "qwen2.5-coder:7b"        # Fast coding
    "llama3.2:8b"             # Fast general purpose
)

# Parse arguments
INCLUDE_OPTIONAL=false
SKIP_LARGE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --include-optional)
            INCLUDE_OPTIONAL=true
            shift
            ;;
        --skip-large)
            SKIP_LARGE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--include-optional] [--skip-large]"
            echo ""
            echo "Options:"
            echo "  --include-optional  Also pull smaller/faster models"
            echo "  --skip-large        Skip 70B+ models"
            exit 0
            ;;
        *)
            shift
            ;;
    esac
done

# Check if Ollama is running
log_info "Checking Ollama service..."
if ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    log_error "Ollama is not running. Please start Ollama first:"
    log_error "  ollama serve"
    log_error "  # or"
    log_error "  launchctl load ~/Library/LaunchAgents/com.ollama.ollama.plist"
    exit 1
fi

log_info "Ollama is running"

# Get currently installed models
log_info "Checking installed models..."
INSTALLED=$(ollama list 2>/dev/null | tail -n +2 | awk '{print $1}' || echo "")
echo "$INSTALLED"
echo ""

# Function to check if model is installed
is_installed() {
    local model=$1
    echo "$INSTALLED" | grep -q "^${model}$" && return 0 || return 1
}

# Function to pull a model
pull_model() {
    local model=$1

    if is_installed "$model"; then
        log_info "Model $model is already installed, skipping"
        return 0
    fi

    log_progress "Pulling $model..."

    if ollama pull "$model"; then
        log_info "Successfully pulled $model"
        return 0
    else
        log_warn "Failed to pull $model"
        return 1
    fi
}

# Track results
PULLED=()
FAILED=()
SKIPPED=()

echo ""
echo "=========================================="
echo -e "${GREEN}Pulling AI Models${NC}"
echo "=========================================="
echo ""

# Pull main models
for model in "${MODELS[@]}"; do
    # Skip large models if requested
    if [[ "$SKIP_LARGE" == true ]] && [[ "$model" == *":70b"* ]]; then
        log_warn "Skipping large model: $model"
        SKIPPED+=("$model")
        continue
    fi

    if pull_model "$model"; then
        PULLED+=("$model")
    else
        FAILED+=("$model")

        # Try fallback for llama3.2:70b -> llama3.1:70b
        if [[ "$model" == "llama3.2:70b" ]]; then
            log_info "Trying fallback: llama3.1:70b"
            if pull_model "llama3.1:70b"; then
                PULLED+=("llama3.1:70b (fallback)")
            fi
        fi
    fi

    echo ""
done

# Pull optional models if requested
if [[ "$INCLUDE_OPTIONAL" == true ]]; then
    echo ""
    log_info "Pulling optional models..."
    echo ""

    for model in "${OPTIONAL_MODELS[@]}"; do
        if pull_model "$model"; then
            PULLED+=("$model")
        else
            FAILED+=("$model")
        fi
        echo ""
    done
fi

# Summary
echo ""
echo "=========================================="
echo -e "${GREEN}Model Pull Complete${NC}"
echo "=========================================="
echo ""

if [[ ${#PULLED[@]} -gt 0 ]]; then
    echo "Successfully pulled:"
    for model in "${PULLED[@]}"; do
        echo "  ✓ $model"
    done
fi

if [[ ${#SKIPPED[@]} -gt 0 ]]; then
    echo ""
    echo "Skipped:"
    for model in "${SKIPPED[@]}"; do
        echo "  - $model"
    done
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
    echo ""
    echo "Failed to pull:"
    for model in "${FAILED[@]}"; do
        echo "  ✗ $model"
    done
fi

echo ""
log_info "Currently installed models:"
ollama list

echo ""
echo "=========================================="
echo "Model storage: ~/.ollama/models"
echo "Disk usage:    $(du -sh ~/.ollama/models 2>/dev/null | cut -f1 || echo 'N/A')"
echo "=========================================="
