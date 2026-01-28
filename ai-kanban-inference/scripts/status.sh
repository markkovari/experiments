#!/bin/bash
#
# Status check script for the distributed AI coding agent system
# Checks connectivity and health of all components
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Status indicators
OK="${GREEN}✓${NC}"
FAIL="${RED}✗${NC}"
WARN="${YELLOW}!${NC}"
INFO="${BLUE}ℹ${NC}"

# Print header
print_header() {
    echo ""
    echo -e "${BOLD}${CYAN}$1${NC}"
    echo "─────────────────────────────────────────"
}

# Check if command exists
cmd_exists() {
    command -v "$1" &> /dev/null
}

# Check service status
check_service() {
    local name=$1
    local check_cmd=$2
    local status

    if eval "$check_cmd" &> /dev/null; then
        echo -e "  ${OK} ${name}"
        return 0
    else
        echo -e "  ${FAIL} ${name}"
        return 1
    fi
}

# Main status check
main() {
    echo ""
    echo -e "${BOLD}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}║      Distributed AI Coding Agent - Status Check          ║${NC}"
    echo -e "${BOLD}╚══════════════════════════════════════════════════════════╝${NC}"

    local errors=0

    #───────────────────────────────────────────────────────────────
    print_header "🔧 Prerequisites"
    #───────────────────────────────────────────────────────────────

    check_service "Tailscale CLI" "cmd_exists tailscale" || ((errors++))
    check_service "Nomad CLI" "cmd_exists nomad" || ((errors++))
    check_service "Consul CLI" "cmd_exists consul" || ((errors++))
    check_service "Node.js" "cmd_exists node" || ((errors++))
    check_service "jq" "cmd_exists jq" || ((errors++))

    #───────────────────────────────────────────────────────────────
    print_header "🌐 Tailscale Network"
    #───────────────────────────────────────────────────────────────

    if cmd_exists tailscale; then
        local ts_status
        ts_status=$(tailscale status --json 2>/dev/null || echo "{}")

        local backend_state
        backend_state=$(echo "$ts_status" | jq -r '.BackendState // "Unknown"')

        if [[ "$backend_state" == "Running" ]]; then
            echo -e "  ${OK} Tailscale daemon running"

            local ts_ip
            ts_ip=$(tailscale ip -4 2>/dev/null || echo "N/A")
            echo -e "  ${INFO} IPv4: ${ts_ip}"

            local ts_hostname
            ts_hostname=$(echo "$ts_status" | jq -r '.Self.DNSName // "N/A"' | sed 's/\.$//')
            echo -e "  ${INFO} Hostname: ${ts_hostname}"

            # Count peers
            local peer_count
            peer_count=$(echo "$ts_status" | jq '.Peer | length // 0')
            echo -e "  ${INFO} Connected peers: ${peer_count}"
        else
            echo -e "  ${FAIL} Tailscale not running (state: ${backend_state})"
            ((errors++))
        fi
    else
        echo -e "  ${FAIL} Tailscale not installed"
        ((errors++))
    fi

    #───────────────────────────────────────────────────────────────
    print_header "📦 Consul Cluster"
    #───────────────────────────────────────────────────────────────

    if cmd_exists consul; then
        if consul members &> /dev/null; then
            echo -e "  ${OK} Consul agent running"

            # Get member info
            local members
            members=$(consul members 2>/dev/null || echo "")

            local server_count client_count
            server_count=$(echo "$members" | grep -c "server" || echo "0")
            client_count=$(echo "$members" | grep -c "client" || echo "0")

            echo -e "  ${INFO} Servers: ${server_count}"
            echo -e "  ${INFO} Clients: ${client_count}"

            # List services
            local services
            services=$(consul catalog services 2>/dev/null | wc -l | tr -d ' ')
            echo -e "  ${INFO} Registered services: ${services}"
        else
            echo -e "  ${FAIL} Consul agent not running or unreachable"
            ((errors++))
        fi
    else
        echo -e "  ${WARN} Consul CLI not installed"
    fi

    #───────────────────────────────────────────────────────────────
    print_header "🚀 Nomad Cluster"
    #───────────────────────────────────────────────────────────────

    if cmd_exists nomad; then
        # Check if we can reach Nomad
        if nomad server members &> /dev/null; then
            echo -e "  ${OK} Nomad server reachable"

            # Server info
            local server_count
            server_count=$(nomad server members 2>/dev/null | tail -n +2 | wc -l | tr -d ' ')
            echo -e "  ${INFO} Servers: ${server_count}"

            # Client info
            local nodes
            nodes=$(nomad node status 2>/dev/null | tail -n +2 || echo "")
            local total_nodes ready_nodes
            total_nodes=$(echo "$nodes" | wc -l | tr -d ' ')
            ready_nodes=$(echo "$nodes" | grep -c "ready" || echo "0")

            echo -e "  ${INFO} Client nodes: ${ready_nodes}/${total_nodes} ready"

            # Job info
            local jobs running_jobs
            jobs=$(nomad job status 2>/dev/null | tail -n +2 || echo "")
            running_jobs=$(echo "$jobs" | grep -c "running" || echo "0")
            echo -e "  ${INFO} Running jobs: ${running_jobs}"

        elif nomad node status &> /dev/null; then
            echo -e "  ${OK} Nomad client running (server unreachable)"
            ((errors++))
        else
            echo -e "  ${FAIL} Nomad not running or unreachable"
            echo -e "  ${INFO} Try: export NOMAD_ADDR=http://<server>:4646"
            ((errors++))
        fi
    else
        echo -e "  ${WARN} Nomad CLI not installed"
    fi

    #───────────────────────────────────────────────────────────────
    print_header "🤖 Ollama Service"
    #───────────────────────────────────────────────────────────────

    # Try local first
    local ollama_host="${OLLAMA_HOST:-localhost:11434}"
    local ollama_url="http://${ollama_host}"

    if curl -s "${ollama_url}/api/tags" &> /dev/null; then
        echo -e "  ${OK} Ollama API responding at ${ollama_host}"

        # Get model info
        local models_json
        models_json=$(curl -s "${ollama_url}/api/tags" 2>/dev/null || echo '{"models":[]}')

        local model_count
        model_count=$(echo "$models_json" | jq '.models | length')
        echo -e "  ${INFO} Loaded models: ${model_count}"

        # List models
        if [[ "$model_count" -gt 0 ]]; then
            echo -e "  ${INFO} Available models:"
            echo "$models_json" | jq -r '.models[].name' | while read -r model; do
                local size
                size=$(echo "$models_json" | jq -r ".models[] | select(.name==\"$model\") | .size" | numfmt --to=iec 2>/dev/null || echo "?")
                echo "       • ${model} (${size})"
            done
        fi
    else
        echo -e "  ${FAIL} Ollama not responding at ${ollama_host}"
        echo -e "  ${INFO} Start with: ollama serve"
        ((errors++))
    fi

    # Check via Consul service discovery
    if cmd_exists consul && consul catalog services 2>/dev/null | grep -q "ollama"; then
        local ollama_consul
        ollama_consul=$(consul catalog nodes -service=ollama 2>/dev/null | tail -n +2 | head -1)
        if [[ -n "$ollama_consul" ]]; then
            echo -e "  ${OK} Registered in Consul"
        fi
    fi

    #───────────────────────────────────────────────────────────────
    print_header "🧠 Claude-Flow Service"
    #───────────────────────────────────────────────────────────────

    # Try to find Claude-Flow via Consul
    local cf_host=""
    if cmd_exists consul && consul catalog services 2>/dev/null | grep -q "claude-flow"; then
        cf_host=$(consul catalog nodes -service=claude-flow 2>/dev/null | tail -n +2 | awk '{print $2}' | head -1)
    fi

    # Fallback to localhost
    cf_host="${cf_host:-localhost}"
    local cf_url="http://${cf_host}:8080"

    if curl -s "${cf_url}/health" &> /dev/null; then
        echo -e "  ${OK} Claude-Flow responding at ${cf_host}:8080"

        # Try to get status
        local cf_status
        cf_status=$(curl -s "${cf_url}/api/status" 2>/dev/null || echo "{}")
        if [[ -n "$cf_status" && "$cf_status" != "{}" ]]; then
            local agent_count
            agent_count=$(echo "$cf_status" | jq '.agents.active // 0' 2>/dev/null || echo "?")
            echo -e "  ${INFO} Active agents: ${agent_count}"
        fi
    else
        echo -e "  ${WARN} Claude-Flow not responding at ${cf_host}:8080"
        echo -e "  ${INFO} May not be deployed yet"
    fi

    # Check Nomad job status
    if cmd_exists nomad && nomad job status claude-flow &> /dev/null; then
        local job_status
        job_status=$(nomad job status claude-flow 2>/dev/null | grep "Status" | head -1 | awk '{print $3}')
        echo -e "  ${INFO} Nomad job status: ${job_status}"
    fi

    #───────────────────────────────────────────────────────────────
    print_header "📊 Resource Usage"
    #───────────────────────────────────────────────────────────────

    # Memory
    if [[ "$(uname)" == "Darwin" ]]; then
        local mem_used mem_total
        mem_total=$(sysctl -n hw.memsize | awk '{print $1/1024/1024/1024}')
        # Get memory pressure
        local mem_pressure
        mem_pressure=$(memory_pressure 2>/dev/null | grep "System-wide memory free percentage" | awk '{print $5}' || echo "?")
        echo -e "  ${INFO} Total memory: ${mem_total:.0f} GB"
        echo -e "  ${INFO} Memory free: ${mem_pressure}"
    else
        local mem_info
        mem_info=$(free -h 2>/dev/null | grep Mem || echo "")
        if [[ -n "$mem_info" ]]; then
            echo -e "  ${INFO} Memory: ${mem_info}"
        fi
    fi

    # Disk for Ollama models
    if [[ -d "$HOME/.ollama/models" ]]; then
        local ollama_size
        ollama_size=$(du -sh "$HOME/.ollama/models" 2>/dev/null | cut -f1)
        echo -e "  ${INFO} Ollama models: ${ollama_size}"
    fi

    #───────────────────────────────────────────────────────────────
    print_header "🔗 Quick Links"
    #───────────────────────────────────────────────────────────────

    local ts_hostname
    ts_hostname=$(tailscale status --json 2>/dev/null | jq -r '.Self.DNSName // ""' | sed 's/\.$//' || echo "localhost")

    echo -e "  Claude-Flow UI:  http://${ts_hostname}:8080"
    echo -e "  Nomad UI:        http://${ts_hostname}:4646"
    echo -e "  Consul UI:       http://${ts_hostname}:8500"
    echo -e "  Ollama API:      http://${ts_hostname}:11434"

    #───────────────────────────────────────────────────────────────
    # Summary
    #───────────────────────────────────────────────────────────────
    echo ""
    echo "─────────────────────────────────────────"
    if [[ $errors -eq 0 ]]; then
        echo -e "${GREEN}${BOLD}All systems operational!${NC}"
    else
        echo -e "${YELLOW}${BOLD}${errors} issue(s) detected${NC}"
        echo -e "Run with verbose output: $0 --verbose"
    fi
    echo ""

    return $errors
}

# Run main
main "$@"
