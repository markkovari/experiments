#!/bin/bash
#
# Cross-Platform Data Plane Setup Script
# Supports: macOS (Apple Silicon/Intel) and Linux
# Skips Tailscale reconfiguration (assumes already connected)
# Installs: Nomad Client, Consul Client, Ollama
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
NOMAD_SERVER="malna.tail3a9c.ts.net"
DATACENTER="home"
NOMAD_VERSION="1.7.5"
CONSUL_VERSION="1.18.0"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --nomad-server=*)
            NOMAD_SERVER="${1#*=}"
            shift
            ;;
        --datacenter=*)
            DATACENTER="${1#*=}"
            shift
            ;;
        --help)
            echo "Usage: $0 [--nomad-server=malna.tail3a9c.ts.net] [--datacenter=home]"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

log_info "Starting Data Plane setup (Tailscale already configured)..."

# Detect OS and architecture
OS_TYPE=$(uname -s)
ARCH=$(uname -m)

case "$OS_TYPE" in
    Darwin*)
        OS="darwin"
        log_info "Detected macOS"
        ;;
    Linux*)
        OS="linux"
        log_info "Detected Linux"
        ;;
    *)
        log_error "Unsupported OS: $OS_TYPE"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_TYPE="amd64"
        ;;
    arm64|aarch64)
        ARCH_TYPE="arm64"
        ;;
    *)
        log_error "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

log_info "Platform: ${OS}_${ARCH_TYPE}"

# Check if running as root - we'll check Tailscale as regular user first
if [[ $EUID -eq 0 ]]; then
    log_error "Do not run this script with sudo!"
    log_error "The script will prompt for sudo when needed."
    exit 1
fi

# Set Tailscale path
if [ "$OS" = "darwin" ]; then
    TAILSCALE_BIN="/opt/homebrew/bin/tailscale"
    if [ ! -x "$TAILSCALE_BIN" ]; then
        TAILSCALE_BIN="tailscale"  # Fallback to PATH
    fi
else
    TAILSCALE_BIN="tailscale"
fi

# Check Tailscale is running (as regular user before any sudo operations)
log_info "Verifying Tailscale connection..."
if ! $TAILSCALE_BIN status &> /dev/null; then
    log_error "Tailscale is not running. Please start Tailscale first."
    exit 1
fi

# Get Tailscale info
TAILSCALE_IP=$($TAILSCALE_BIN ip -4)
TAILSCALE_HOSTNAME=$($TAILSCALE_BIN status --json | jq -r '.Self.DNSName' | sed 's/\.$//')
log_info "Tailscale IP: $TAILSCALE_IP"
log_info "Tailscale hostname: $TAILSCALE_HOSTNAME"

# 1. Install dependencies based on OS
if [ "$OS" = "darwin" ]; then
    # macOS - use Homebrew
    log_info "Checking for Homebrew..."
    if ! command -v brew &> /dev/null; then
        log_info "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi

    # 2. Install Node.js
    log_info "Installing Node.js..."
    if ! command -v node &> /dev/null || [[ $(node -v | cut -d'.' -f1 | tr -d 'v') -lt 20 ]]; then
        brew install node@20
        brew link --overwrite node@20
    fi
    node -v

    # 3. Install jq if not present
    if ! command -v jq &> /dev/null; then
        brew install jq
    fi
else
    # Linux - use apt-get
    log_info "Updating system packages..."
    sudo apt-get update

    log_info "Installing required packages..."
    sudo apt-get install -y curl wget unzip jq git

    # Install Node.js 20+
    log_info "Installing Node.js 20..."
    if ! command -v node &> /dev/null || [[ $(node -v | cut -d'.' -f1 | tr -d 'v') -lt 20 ]]; then
        curl -fsSL https://deb.nodesource.com/setup_20.x | sudo bash -
        sudo apt-get install -y nodejs
    fi
    node -v
    npm -v
fi

# 4. Install Ollama
log_info "Installing Ollama..."
if ! command -v ollama &> /dev/null; then
    if [ "$OS" = "darwin" ]; then
        log_info "Installing Ollama via Homebrew..."
        brew install ollama
    else
        log_info "Installing Ollama via install script..."
        curl -fsSL https://ollama.com/install.sh | sh
    fi
else
    log_info "Ollama already installed"
fi

# 5. Create directories
log_info "Creating directories..."
sudo mkdir -p /opt/nomad/data
sudo mkdir -p /opt/consul/data
sudo mkdir -p /etc/nomad.d
sudo mkdir -p /etc/consul.d
sudo chown -R $(whoami) /opt/nomad /opt/consul

# 6. Configure Ollama service
log_info "Configuring Ollama service..."

if [ "$OS" = "darwin" ]; then
    # macOS - use LaunchAgent
    mkdir -p ~/Library/LaunchAgents

    cat > ~/Library/LaunchAgents/com.ollama.ollama.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.ollama.ollama</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/ollama</string>
        <string>serve</string>
    </array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>OLLAMA_HOST</key>
        <string>0.0.0.0:11434</string>
        <key>OLLAMA_ORIGINS</key>
        <string>*</string>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/ollama.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/ollama.err</string>
</dict>
</plist>
EOF

    # Unload and reload Ollama service
    launchctl unload ~/Library/LaunchAgents/com.ollama.ollama.plist 2>/dev/null || true
    launchctl load -w ~/Library/LaunchAgents/com.ollama.ollama.plist
else
    # Linux - use systemd
    sudo tee /etc/systemd/system/ollama.service > /dev/null << 'EOF'
[Unit]
Description=Ollama Service
After=network-online.target

[Service]
ExecStart=/usr/local/bin/ollama serve
User=root
Group=root
Restart=always
RestartSec=3
Environment="OLLAMA_HOST=0.0.0.0:11434"
Environment="OLLAMA_ORIGINS=*"

[Install]
WantedBy=default.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable ollama
    sudo systemctl start ollama
fi

# Wait for Ollama to start
sleep 3
log_info "Ollama service started"

# 7. Download and install Nomad
log_info "Installing Nomad ${NOMAD_VERSION}..."
NOMAD_URL="https://releases.hashicorp.com/nomad/${NOMAD_VERSION}/nomad_${NOMAD_VERSION}_${OS}_${ARCH_TYPE}.zip"
curl -sL "$NOMAD_URL" -o /tmp/nomad.zip
sudo unzip -o /tmp/nomad.zip -d /usr/local/bin/
sudo chmod +x /usr/local/bin/nomad
rm /tmp/nomad.zip
nomad version

# 8. Download and install Consul
log_info "Installing Consul ${CONSUL_VERSION}..."
CONSUL_URL="https://releases.hashicorp.com/consul/${CONSUL_VERSION}/consul_${CONSUL_VERSION}_${OS}_${ARCH_TYPE}.zip"
curl -sL "$CONSUL_URL" -o /tmp/consul.zip
sudo unzip -o /tmp/consul.zip -d /usr/local/bin/
sudo chmod +x /usr/local/bin/consul
rm /tmp/consul.zip
consul version

# 9. Create Nomad client configuration
log_info "Configuring Nomad client..."
sudo tee /etc/nomad.d/nomad.hcl > /dev/null << EOF
datacenter = "${DATACENTER}"
data_dir   = "/opt/nomad/data"

bind_addr = "${TAILSCALE_IP}"

advertise {
  http = "${TAILSCALE_IP}:4646"
  rpc  = "${TAILSCALE_IP}:4647"
  serf = "${TAILSCALE_IP}:4648"
}

client {
  enabled    = true
  node_class = "data-plane"

  servers = ["${NOMAD_SERVER}:4647"]

  meta {
    role   = "data-plane"
    gpu    = "apple-silicon"
    memory = "96gb"
  }

  host_volume "ollama-models" {
    path      = "$HOME/.ollama/models"
    read_only = false
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

# 10. Create Consul client configuration
log_info "Configuring Consul client..."
sudo tee /etc/consul.d/consul.hcl > /dev/null << EOF
datacenter = "${DATACENTER}"
data_dir   = "/opt/consul/data"
log_level  = "INFO"

bind_addr   = "${TAILSCALE_IP}"
client_addr = "0.0.0.0"

server = false

retry_join = ["${NOMAD_SERVER}"]
EOF

# 11. Create and start Nomad service
if [ "$OS" = "darwin" ]; then
    # macOS - use LaunchDaemon
    log_info "Creating Nomad LaunchDaemon..."
    sudo tee /Library/LaunchDaemons/io.nomadproject.nomad.plist > /dev/null << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>io.nomadproject.nomad</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/nomad</string>
        <string>agent</string>
        <string>-config</string>
        <string>/etc/nomad.d</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/nomad.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/nomad.err</string>
</dict>
</plist>
EOF
else
    # Linux - use systemd
    log_info "Creating Nomad systemd service..."
    sudo tee /etc/systemd/system/nomad.service > /dev/null << 'EOF'
[Unit]
Description=Nomad
Documentation=https://www.nomadproject.io/docs/
Wants=network-online.target
After=network-online.target

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
fi

# 12. Create and start Consul service
if [ "$OS" = "darwin" ]; then
    # macOS - use LaunchDaemon
    log_info "Creating Consul LaunchDaemon..."
    sudo tee /Library/LaunchDaemons/io.consul.consul.plist > /dev/null << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>io.consul.consul</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/consul</string>
        <string>agent</string>
        <string>-config-dir=/etc/consul.d</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/consul.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/consul.err</string>
</dict>
</plist>
EOF
else
    # Linux - use systemd
    log_info "Creating Consul systemd service..."
    sudo tee /etc/systemd/system/consul.service > /dev/null << 'EOF'
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
fi

# 13. Load and start services
log_info "Starting services..."

if [ "$OS" = "darwin" ]; then
    # macOS - use launchctl
    # Stop existing services if running
    sudo launchctl unload /Library/LaunchDaemons/io.consul.consul.plist 2>/dev/null || true
    sudo launchctl unload /Library/LaunchDaemons/io.nomadproject.nomad.plist 2>/dev/null || true

    # Start Consul first
    sudo launchctl load -w /Library/LaunchDaemons/io.consul.consul.plist
    sleep 5

    # Start Nomad
    sudo launchctl load -w /Library/LaunchDaemons/io.nomadproject.nomad.plist
    sleep 5
else
    # Linux - use systemctl
    sudo systemctl daemon-reload
    sudo systemctl enable consul
    sudo systemctl enable nomad

    # Start Consul first
    sudo systemctl start consul
    sleep 5

    # Start Nomad
    sudo systemctl start nomad
    sleep 5
fi

# 14. Wait for services and verify
log_info "Waiting for services to initialize..."
sleep 10

# Check Ollama
OLLAMA_STATUS="Not Running"
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    OLLAMA_STATUS="Running"
fi

# Check Consul
CONSUL_STATUS="Not Running"
if consul members > /dev/null 2>&1; then
    CONSUL_STATUS="Running"
fi

# Check Nomad
NOMAD_STATUS="Not Running"
if nomad node status > /dev/null 2>&1; then
    NOMAD_STATUS="Running"
fi

# 15. Output status
echo ""
echo "=========================================="
echo -e "${GREEN}Data Plane Setup Complete!${NC}"
echo "=========================================="
echo ""
echo "Services Status:"
echo "  Ollama: ${OLLAMA_STATUS}"
echo "  Consul: ${CONSUL_STATUS}"
echo "  Nomad:  ${NOMAD_STATUS}"
echo ""
echo "Ollama API Endpoint:"
echo "  Local:     http://localhost:11434"
echo "  Tailscale: http://${TAILSCALE_HOSTNAME}:11434"
echo ""
echo "Tailscale Info:"
echo "  IP: ${TAILSCALE_IP}"
echo "  Hostname: ${TAILSCALE_HOSTNAME}"
echo ""
echo "Next Steps:"
echo "  1. Run ./scripts/pull-models.sh to download AI models"
echo "  2. Run ./scripts/deploy-stack.sh to deploy Nomad jobs"
echo "  3. Verify cluster: nomad node status"
echo "=========================================="
