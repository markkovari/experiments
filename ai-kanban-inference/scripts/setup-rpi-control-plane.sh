#!/bin/bash
#
# Setup script for Raspberry Pi 5 as Control Plane
# Installs: Nomad Server, Consul Server, Claude-Flow, Tailscale
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
DATACENTER="home"
NOMAD_VERSION="1.7.5"
CONSUL_VERSION="1.18.0"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --tailscale-key=*)
            TAILSCALE_KEY="${1#*=}"
            shift
            ;;
        --datacenter=*)
            DATACENTER="${1#*=}"
            shift
            ;;
        --help)
            echo "Usage: $0 --tailscale-key=tskey-auth-xxx [--datacenter=home]"
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
    log_error "Tailscale auth key required. Use --tailscale-key=tskey-auth-xxx"
    exit 1
fi

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run as root"
    exit 1
fi

log_info "Starting Control Plane setup on Raspberry Pi 5..."

# 1. Update system packages
log_info "Updating system packages..."
apt-get update && apt-get upgrade -y

# 2. Install required packages
log_info "Installing required packages..."
apt-get install -y curl wget unzip jq git

# 3. Install Node.js 20+
log_info "Installing Node.js 20..."
if ! command -v node &> /dev/null || [[ $(node -v | cut -d'.' -f1 | tr -d 'v') -lt 20 ]]; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
    apt-get install -y nodejs
fi
node -v
npm -v

# 4. Install Tailscale
log_info "Installing Tailscale..."
if ! command -v tailscale &> /dev/null; then
    curl -fsSL https://tailscale.com/install.sh | sh
fi

# 5. Authenticate Tailscale
log_info "Authenticating Tailscale..."
tailscale up --authkey="$TAILSCALE_KEY" --hostname="rpi5-control"

# Wait for Tailscale to connect
sleep 5
TAILSCALE_IP=$(tailscale ip -4)
TAILSCALE_HOSTNAME=$(tailscale status --json | jq -r '.Self.DNSName' | sed 's/\.$//')
log_info "Tailscale IP: $TAILSCALE_IP"
log_info "Tailscale hostname: $TAILSCALE_HOSTNAME"

# 6. Download and install Nomad
log_info "Installing Nomad ${NOMAD_VERSION}..."
NOMAD_URL="https://releases.hashicorp.com/nomad/${NOMAD_VERSION}/nomad_${NOMAD_VERSION}_linux_arm64.zip"
wget -q "$NOMAD_URL" -O /tmp/nomad.zip
unzip -o /tmp/nomad.zip -d /usr/local/bin/
chmod +x /usr/local/bin/nomad
rm /tmp/nomad.zip
nomad version

# 7. Download and install Consul
log_info "Installing Consul ${CONSUL_VERSION}..."
CONSUL_URL="https://releases.hashicorp.com/consul/${CONSUL_VERSION}/consul_${CONSUL_VERSION}_linux_arm64.zip"
wget -q "$CONSUL_URL" -O /tmp/consul.zip
unzip -o /tmp/consul.zip -d /usr/local/bin/
chmod +x /usr/local/bin/consul
rm /tmp/consul.zip
consul version

# 8. Create directories
log_info "Creating data directories..."
mkdir -p /opt/nomad/data
mkdir -p /opt/consul/data
mkdir -p /opt/claude-flow/data
mkdir -p /etc/nomad.d
mkdir -p /etc/consul.d

# 9. Create Nomad server configuration
log_info "Configuring Nomad server..."
cat > /etc/nomad.d/nomad.hcl << EOF
datacenter = "${DATACENTER}"
data_dir   = "/opt/nomad/data"

bind_addr = "${TAILSCALE_IP}"

advertise {
  http = "${TAILSCALE_IP}:4646"
  rpc  = "${TAILSCALE_IP}:4647"
  serf = "${TAILSCALE_IP}:4648"
}

server {
  enabled          = true
  bootstrap_expect = 1

  # Server join configuration
  server_join {
    retry_max = 3
    retry_interval = "15s"
  }
}

client {
  enabled    = true
  node_class = "control-plane"

  meta {
    role = "control-plane"
  }
}

consul {
  address = "127.0.0.1:8500"
}

plugin "raw_exec" {
  config {
    enabled = true
  }
}
EOF

# 10. Create Consul server configuration
log_info "Configuring Consul server..."
cat > /etc/consul.d/consul.hcl << EOF
datacenter = "${DATACENTER}"
data_dir   = "/opt/consul/data"
log_level  = "INFO"

bind_addr   = "${TAILSCALE_IP}"
client_addr = "0.0.0.0"

server           = true
bootstrap_expect = 1

ui_config {
  enabled = true
}

connect {
  enabled = true
}

ports {
  grpc = 8502
}

retry_join = []
EOF

# 11. Create systemd service for Nomad
log_info "Creating Nomad systemd service..."
cat > /etc/systemd/system/nomad.service << 'EOF'
[Unit]
Description=Nomad
Documentation=https://www.nomadproject.io/docs/
Wants=network-online.target
After=network-online.target consul.service

[Service]
ExecReload=/bin/kill -HUP $MAINPID
ExecStart=/usr/local/bin/nomad agent -config /etc/nomad.d
KillMode=process
KillSignal=SIGINT
LimitNOFILE=65536
LimitNPROC=infinity
Restart=on-failure
RestartSec=2
TasksMax=infinity
OOMScoreAdjust=-1000

[Install]
WantedBy=multi-user.target
EOF

# 12. Create systemd service for Consul
log_info "Creating Consul systemd service..."
cat > /etc/systemd/system/consul.service << 'EOF'
[Unit]
Description=Consul
Documentation=https://www.consul.io/docs/
Wants=network-online.target
After=network-online.target

[Service]
ExecReload=/bin/kill -HUP $MAINPID
ExecStart=/usr/local/bin/consul agent -config-dir=/etc/consul.d
KillMode=process
KillSignal=SIGINT
LimitNOFILE=65536
Restart=on-failure
RestartSec=2
TasksMax=infinity

[Install]
WantedBy=multi-user.target
EOF

# 13. Install Claude-Flow
log_info "Installing Claude-Flow..."
npm install -g claude-flow@alpha || npm install -g @anthropic/claude-flow@latest || {
    log_warn "Claude-Flow package not found, will use npx"
}

# 14. Create Claude-Flow configuration
log_info "Creating Claude-Flow configuration..."
cat > /opt/claude-flow/config.json << EOF
{
  "providers": {
    "ollama": {
      "baseUrl": "http://ollama.service.consul:11434",
      "models": {
        "default": "codellama:34b",
        "code": "deepseek-coder:33b",
        "reasoning": "llama3.2:70b",
        "fast": "qwen2.5-coder:32b"
      }
    }
  },
  "agents": {
    "defaultProvider": "ollama",
    "swarm": {
      "enabled": true,
      "consensus": "raft"
    }
  },
  "persistence": {
    "type": "sqlite",
    "path": "/opt/claude-flow/data/state.db"
  },
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "mcpPort": 8081
  }
}
EOF

# 15. Create Claude-Flow systemd service
log_info "Creating Claude-Flow systemd service..."
cat > /etc/systemd/system/claude-flow.service << EOF
[Unit]
Description=Claude-Flow Agent Orchestrator
Documentation=https://github.com/ruvnet/claude-flow
After=network-online.target nomad.service consul.service
Wants=network-online.target

[Service]
Type=simple
User=root
Environment=NODE_ENV=production
Environment=CLAUDE_FLOW_CONFIG=/opt/claude-flow/config.json
Environment=OLLAMA_HOST=http://ollama.service.consul:11434
ExecStart=/usr/bin/npx claude-flow@alpha server --port 8080
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# 16. Reload systemd and enable services
log_info "Enabling and starting services..."
systemctl daemon-reload
systemctl enable consul
systemctl enable nomad
systemctl enable claude-flow

# Start services in order
systemctl start consul
sleep 5
systemctl start nomad
sleep 5
systemctl start claude-flow

# 17. Wait for services to be ready
log_info "Waiting for services to initialize..."
sleep 10

# 18. Output status and join information
echo ""
echo "=========================================="
echo -e "${GREEN}Control Plane Setup Complete!${NC}"
echo "=========================================="
echo ""
echo "Services Status:"
systemctl is-active consul && echo "  Consul: Running" || echo "  Consul: Not Running"
systemctl is-active nomad && echo "  Nomad: Running" || echo "  Nomad: Not Running"
systemctl is-active claude-flow && echo "  Claude-Flow: Running" || echo "  Claude-Flow: Not Running"
echo ""
echo "Access URLs (via Tailscale):"
echo "  Nomad UI:      http://${TAILSCALE_HOSTNAME}:4646"
echo "  Consul UI:     http://${TAILSCALE_HOSTNAME}:8500"
echo "  Claude-Flow:   http://${TAILSCALE_HOSTNAME}:8080"
echo ""
echo "Nomad Join Address (for data-plane nodes):"
echo "  ${TAILSCALE_HOSTNAME}:4647"
echo ""
echo "Tailscale Info:"
echo "  IP: ${TAILSCALE_IP}"
echo "  Hostname: ${TAILSCALE_HOSTNAME}"
echo ""
echo "Next Steps:"
echo "  1. Run setup-mac-data-plane.sh on your MacBook"
echo "  2. Use --nomad-server=${TAILSCALE_HOSTNAME} when joining"
echo "=========================================="
