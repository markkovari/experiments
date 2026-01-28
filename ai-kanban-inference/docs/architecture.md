# Architecture Documentation

## Overview

This document describes the architecture of the distributed AI coding agent system. The system is designed to run AI-powered coding agents locally, without relying on external cloud APIs, while maintaining scalability and isolation.

## Design Principles

1. **Local-First**: All AI inference runs on local hardware
2. **Secure by Default**: Tailscale provides encrypted communication
3. **Lightweight**: Nomad uses ~50MB RAM vs Kubernetes' 500MB+
4. **Isolated Execution**: Each agent task runs in its own allocation
5. **Self-Healing**: Automatic restarts and health checks

## System Components

### Control Plane (Raspberry Pi 5)

The control plane handles orchestration and provides the user interface.

| Component | Port | Purpose |
|-----------|------|---------|
| Nomad Server | 4646, 4647, 4648 | Workload scheduling |
| Consul Server | 8500, 8600 | Service discovery |
| Claude-Flow | 8080, 8081 | Agent orchestration UI |

**Resource Requirements:**
- CPU: 1+ GHz (quad-core recommended)
- RAM: 2GB minimum, 4GB recommended
- Storage: 16GB+ microSD or SSD

### Data Plane (MacBook Pro M1 MAX)

The data plane executes AI workloads with GPU acceleration.

| Component | Port | Purpose |
|-----------|------|---------|
| Nomad Client | 4646-4648 | Task execution |
| Consul Client | 8500 | Service registration |
| Ollama | 11434 | LLM inference |

**Resource Requirements:**
- CPU: Apple Silicon (M1/M2/M3)
- RAM: 64GB minimum for 34B models, 96GB for 70B
- Storage: 200GB+ for model storage

## Network Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Tailscale Mesh VPN                       │
│                   (WireGuard encrypted)                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   100.x.x.1                              100.x.x.2          │
│   ┌───────────┐                         ┌───────────┐       │
│   │   RPi5    │◄───── Tailscale ───────►│  MacBook  │       │
│   │           │       Network           │           │       │
│   └───────────┘                         └───────────┘       │
│                                                             │
│   DNS: rpi5-control.tailnet.ts.net      macbook-data.ts.net │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Port Matrix

| Service | Protocol | Port | Access |
|---------|----------|------|--------|
| Nomad HTTP | TCP | 4646 | Tailscale only |
| Nomad RPC | TCP | 4647 | Tailscale only |
| Nomad Serf | TCP/UDP | 4648 | Tailscale only |
| Consul HTTP | TCP | 8500 | Tailscale only |
| Consul DNS | TCP/UDP | 8600 | Tailscale only |
| Consul gRPC | TCP | 8502 | Tailscale only |
| Claude-Flow | TCP | 8080 | Tailscale only |
| Claude-Flow MCP | TCP | 8081 | Tailscale only |
| Ollama | TCP | 11434 | Tailscale only |

## Service Discovery

Consul provides DNS-based service discovery:

```
ollama.service.consul      → MacBook Ollama API
claude-flow.service.consul → RPi Claude-Flow API
nomad.service.consul       → Nomad server
```

### Health Checks

| Service | Check Type | Interval | Endpoint |
|---------|------------|----------|----------|
| Ollama | HTTP | 30s | `/api/tags` |
| Claude-Flow | HTTP | 10s | `/health` |
| Nomad | TCP | 10s | Port 4646 |

## Task Execution Model

### Agent Task Lifecycle

```
1. Task Created
   └─► Claude-Flow receives task request

2. Agent Selection
   └─► Select agent type (coder, reviewer, etc.)
   └─► Choose appropriate model

3. Job Submission
   └─► Generate Nomad job spec from template
   └─► Submit batch job to Nomad

4. Scheduling
   └─► Nomad evaluates constraints
   └─► Selects data-plane node
   └─► Creates allocation

5. Execution
   └─► Agent runs in isolated allocation
   └─► Calls local Ollama for inference
   └─► Writes output to allocation directory

6. Completion
   └─► Results returned to Claude-Flow
   └─► Job marked complete
   └─► Allocation garbage collected
```

### Resource Isolation

Each agent task gets:

| Resource | Default | Configurable |
|----------|---------|--------------|
| CPU | 4 cores | Yes |
| Memory | 8 GB | Yes |
| Disk | 2 GB ephemeral | Yes |
| Timeout | 30 minutes | Yes |
| Network | Host network | No |

### Scheduling Constraints

```hcl
# Agent tasks run on data-plane
constraint {
  attribute = "${meta.role}"
  value     = "data-plane"
}

# Prefer nodes with more memory
affinity {
  attribute = "${meta.memory}"
  value     = "96gb"
  weight    = 100
}
```

## Model Selection

### Available Models

| Model | Parameters | VRAM | Use Case |
|-------|------------|------|----------|
| codellama:34b | 34B | ~19GB | Code generation |
| deepseek-coder:33b | 33B | ~18GB | Code/debug |
| llama3.2:70b | 70B | ~40GB | Reasoning |
| qwen2.5-coder:32b | 32B | ~18GB | Fast coding |

### Model Routing

Claude-Flow routes tasks to models based on agent type:

```json
{
  "coder": "deepseek-coder:33b",
  "reviewer": "codellama:34b",
  "architect": "llama3.2:70b",
  "debugger": "deepseek-coder:33b",
  "documenter": "qwen2.5-coder:32b"
}
```

## Failure Handling

### Automatic Recovery

| Failure | Response |
|---------|----------|
| Agent task crash | Restart up to 2 times |
| Ollama timeout | Retry up to 3 times |
| Node disconnect | Reschedule to healthy node |
| Consul failure | Nomad continues with cached data |

### Circuit Breaker

Claude-Flow implements circuit breaker pattern:
- Opens after 5 consecutive failures
- Half-open after 30 seconds
- Closes after 3 successful requests

## Security Model

### Network Security

- **Tailscale**: WireGuard encryption for all traffic
- **No Public Exposure**: Services only accessible via Tailscale
- **Firewall**: All non-Tailscale ports blocked

### Execution Security

- **Isolated Allocations**: Each task runs separately
- **Resource Limits**: Prevents resource exhaustion
- **Ephemeral Storage**: No persistent state between tasks
- **No External Network**: Tasks can only reach local Ollama

### Data Security

- **Local Inference**: No data sent to external APIs
- **No Logging of Prompts**: Sensitive data stays local
- **Encrypted Storage**: Tailscale encrypts in transit

## Scaling Considerations

### Horizontal Scaling

Add more data-plane nodes:

1. Run `setup-mac-data-plane.sh` on new machine
2. Point to existing Nomad server
3. Node automatically joins cluster
4. Workloads distribute across nodes

### Vertical Scaling

For larger models (70B+):
- Increase `OLLAMA_NUM_PARALLEL` for concurrent requests
- Adjust `memory` in agent-task template
- Use `OLLAMA_KEEP_ALIVE` to keep models loaded

### Multi-Region

For geographically distributed teams:
- Deploy control plane per region
- Use Nomad federation
- Configure Consul WAN gossip

## Monitoring

### Metrics Endpoints

| Service | Endpoint | Format |
|---------|----------|--------|
| Nomad | `:4646/v1/metrics` | Prometheus |
| Consul | `:8500/v1/agent/metrics` | Prometheus |
| Claude-Flow | `:9090/metrics` | Prometheus |

### Key Metrics

- `nomad_client_allocs_running`: Active allocations
- `consul_health_service_status`: Service health
- `ollama_requests_total`: Inference requests
- `claude_flow_tasks_completed`: Completed tasks

## Backup and Recovery

### What to Backup

| Component | Path | Frequency |
|-----------|------|-----------|
| Nomad state | `/opt/nomad/data` | Daily |
| Consul state | `/opt/consul/data` | Daily |
| Claude-Flow DB | `/opt/claude-flow/data` | Hourly |
| Ollama models | `~/.ollama/models` | Weekly |

### Recovery Procedure

1. Restore Consul data first (foundation for service discovery)
2. Restore Nomad data (workload state)
3. Restore Claude-Flow database (task history)
4. Verify Ollama models present
5. Restart all services
6. Run `./scripts/status.sh` to verify

## Future Enhancements

- [ ] ACL/RBAC for multi-user access
- [ ] Vault integration for secrets
- [ ] Prometheus/Grafana monitoring stack
- [ ] Automated model updates
- [ ] Multi-region federation
- [ ] GPU scheduling for multi-GPU nodes
