# Distributed AI Coding Agent System

A distributed AI-powered coding agent system using Claude-Flow, Ollama, and HashiCorp Nomad.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Tailscale Network                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────┐    ┌─────────────────────────────┐│
│  │   Raspberry Pi 5        │    │   MacBook Pro M1 MAX        ││
│  │   (Control Plane)       │    │   (Data Plane)              ││
│  │                         │    │                             ││
│  │  • Nomad Server         │    │  • Nomad Client             ││
│  │  • Consul Server        │◄───┤  • Consul Client            ││
│  │  • Claude-Flow UI       │    │  • Ollama (native)          ││
│  │                         │    │  • Agent Workers            ││
│  └─────────────────────────┘    └─────────────────────────────┘│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Components

| Component | Location | Purpose |
|-----------|----------|---------|
| **Nomad Server** | RPi5 | Schedules and orchestrates workloads |
| **Consul Server** | RPi5 | Service discovery and health checking |
| **Claude-Flow** | RPi5 | Multi-agent orchestration UI and API |
| **Nomad Client** | MacBook | Executes agent tasks |
| **Consul Client** | MacBook | Registers local services |
| **Ollama** | MacBook | Local LLM inference (GPU accelerated) |

## Pre-installed Models

All models run locally on Apple Silicon with 96GB unified memory:

| Model | Size | Use Case |
|-------|------|----------|
| `codellama:34b` | ~19GB | Code-focused tasks |
| `deepseek-coder:33b` | ~18GB | Coding and debugging |
| `llama3.2:70b` | ~40GB | Complex reasoning |
| `qwen2.5-coder:32b` | ~18GB | Fast coding tasks |

## Quick Start

### Prerequisites

- Raspberry Pi 5 (4GB+ RAM) with Raspberry Pi OS
- MacBook Pro M1 MAX (or similar Apple Silicon with 64GB+ RAM)
- Tailscale account ([sign up free](https://tailscale.com))

### 1. Generate Tailscale Auth Key

1. Go to [Tailscale Admin Console](https://login.tailscale.com/admin/settings/keys)
2. Generate a new auth key (reusable recommended)
3. Save the key: `tskey-auth-xxxxxxxx`

### 2. Setup Control Plane (Raspberry Pi 5)

SSH into your Raspberry Pi and run:

```bash
# Clone the repository
git clone https://github.com/markkovari/ai-kanban-inference.git
cd ai-kanban-inference

# Run setup script as root
sudo ./scripts/setup-rpi-control-plane.sh --tailscale-key=tskey-auth-xxx
```

Note the Nomad server address from the output (e.g., `rpi5-control.tailnet-name.ts.net`).

### 3. Setup Data Plane (MacBook)

On your MacBook:

```bash
# Clone the repository
git clone https://github.com/markkovari/ai-kanban-inference.git
cd ai-kanban-inference

# Run setup script
./scripts/setup-mac-data-plane.sh \
  --tailscale-key=tskey-auth-xxx \
  --nomad-server=rpi5-control.tailnet-name.ts.net
```

### 4. Pull AI Models

```bash
# Pull all models (this takes a while for large models)
./scripts/pull-models.sh

# Or skip the 70B model for faster setup
./scripts/pull-models.sh --skip-large

# Include optional smaller models for faster inference
./scripts/pull-models.sh --include-optional
```

### 5. Deploy the Stack

```bash
# Set Nomad address
export NOMAD_ADDR=http://rpi5-control.tailnet-name.ts.net:4646

# Deploy all services
./scripts/deploy-stack.sh
```

### 6. Verify Installation

```bash
# Check cluster status
./scripts/status.sh

# Or manually:
tailscale status          # Both nodes connected
consul members            # Both nodes healthy
nomad node status         # Both nodes ready
curl http://macbook:11434/api/tags  # Ollama responding
```

### 7. Access the UI

Open in your browser:
- **Claude-Flow UI**: `http://rpi5-control.tailnet-name.ts.net:8080`
- **Nomad UI**: `http://rpi5-control.tailnet-name.ts.net:4646`
- **Consul UI**: `http://rpi5-control.tailnet-name.ts.net:8500`

## Directory Structure

```
ai-kanban-inference/
├── scripts/
│   ├── setup-rpi-control-plane.sh  # Bootstrap RPi5 control plane
│   ├── setup-mac-data-plane.sh     # Bootstrap Mac data plane
│   ├── install-tailscale.sh        # Cross-platform Tailscale installer
│   ├── deploy-stack.sh             # Deploy Nomad jobs
│   ├── pull-models.sh              # Pull Ollama models
│   └── status.sh                   # Check cluster status
├── nomad/
│   ├── claude-flow.nomad           # Claude-Flow service job
│   ├── ollama-service.nomad        # Ollama service registration
│   └── agent-task.nomad.tpl        # Template for agent batch jobs
├── config/
│   ├── nomad-server.hcl            # Nomad server config (RPi)
│   ├── nomad-client-mac.hcl        # Nomad client config (Mac)
│   ├── consul-server.hcl           # Consul server config
│   ├── consul-client.hcl           # Consul client config
│   ├── claude-flow.json            # Claude-Flow configuration
│   ├── ollama.plist                # macOS LaunchAgent for Ollama
│   └── .env.example                # Environment variables template
└── docs/
    ├── architecture.md             # Detailed architecture docs
    └── architecture.mermaid        # Mermaid.js diagram source
```

## Agent Types

Claude-Flow provides 60+ specialized agent types. Key agents include:

| Agent | Model | Purpose |
|-------|-------|---------|
| `coder` | deepseek-coder:33b | Write and implement code |
| `reviewer` | codellama:34b | Review code for issues |
| `architect` | llama3.2:70b | Design system architecture |
| `debugger` | deepseek-coder:33b | Find and fix bugs |
| `documenter` | qwen2.5-coder:32b | Write documentation |
| `tester` | codellama:34b | Write and analyze tests |

## How It Works

### Task Execution Flow

1. **Create Task**: User creates a coding task in Claude-Flow UI
2. **Agent Selection**: Claude-Flow selects appropriate agent type
3. **Job Scheduling**: Nomad schedules a batch job on the Mac
4. **Isolated Execution**: Agent runs in isolated allocation with resource limits
5. **Local Inference**: Agent calls Ollama locally (no external API)
6. **Result Return**: Output returned to Claude-Flow
7. **Cleanup**: Nomad garbage collects completed job

### Isolation Model

Each agent task runs with:
- **Nomad Batch Job**: Separate job per task
- **Resource Limits**: CPU/memory enforced via cgroups
- **Ephemeral Workspace**: Fresh 2GB filesystem per task
- **Task Timeout**: Auto-kill after 30 minutes (configurable)
- **No Network Persistence**: Clean state between tasks

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and customize:

```bash
cp config/.env.example .env
```

Key variables:

| Variable | Description |
|----------|-------------|
| `TAILSCALE_KEY` | Tailscale auth key |
| `NOMAD_SERVER` | RPi Tailscale hostname |
| `OLLAMA_HOST` | Ollama bind address |
| `AGENT_TIMEOUT` | Default task timeout |

### Claude-Flow Configuration

Edit `config/claude-flow.json` to:
- Change default models per agent type
- Adjust temperature and token limits
- Configure swarm consensus settings
- Set persistence options

### Nomad Resources

Edit `nomad/agent-task.nomad.tpl` to adjust:
- CPU allocation (default: 4 cores)
- Memory limit (default: 8GB)
- Disk space (default: 2GB)
- Timeout (default: 30 minutes)

## Troubleshooting

### Tailscale not connecting

```bash
# Check Tailscale status
tailscale status

# Re-authenticate
sudo tailscale up --authkey=tskey-auth-xxx --reset
```

### Nomad client not joining

```bash
# Check Nomad logs
journalctl -u nomad -f          # Linux
tail -f /var/log/nomad.log      # macOS

# Verify connectivity
nc -zv rpi5-control.ts.net 4647
```

### Ollama not responding

```bash
# Check Ollama status
curl http://localhost:11434/api/tags

# Restart Ollama
launchctl unload ~/Library/LaunchAgents/com.ollama.ollama.plist
launchctl load -w ~/Library/LaunchAgents/com.ollama.ollama.plist

# Check logs
tail -f /tmp/ollama.log
```

### Claude-Flow not starting

```bash
# Check Claude-Flow job status
nomad job status claude-flow

# View logs
nomad alloc logs <alloc-id>

# Restart the job
nomad job stop claude-flow
nomad job run nomad/claude-flow.nomad
```

## Security Considerations

- **Tailscale**: All traffic encrypted via WireGuard
- **No Public Exposure**: Services only accessible via Tailscale
- **Local Inference**: No data sent to external APIs
- **ACLs Disabled**: Enable in production (see Nomad/Consul docs)

## Performance Tuning

### Ollama

```bash
# Increase parallel requests
export OLLAMA_NUM_PARALLEL=4

# Keep models loaded longer
export OLLAMA_KEEP_ALIVE=30m
```

### Nomad

Adjust in `nomad/agent-task.nomad.tpl`:
```hcl
resources {
  cpu    = 8000   # 8 cores for heavy tasks
  memory = 16384  # 16GB for large models
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License

MIT License - See LICENSE file for details.

## References

- [Claude-Flow](https://github.com/ruvnet/claude-flow) - Multi-agent orchestration
- [Ollama](https://ollama.com) - Local LLM inference
- [Nomad](https://www.nomadproject.io) - Workload orchestration
- [Consul](https://www.consul.io) - Service discovery
- [Tailscale](https://tailscale.com) - Mesh VPN
