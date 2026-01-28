# Architecture Diagrams

This document contains Mermaid.js diagrams visualizing the system architecture.

## System Overview

```mermaid
%%{init: {'theme': 'base', 'themeVariables': { 'primaryColor': '#4f46e5', 'primaryTextColor': '#fff', 'primaryBorderColor': '#3730a3', 'lineColor': '#6366f1', 'secondaryColor': '#f0fdf4', 'tertiaryColor': '#fef3c7'}}}%%

graph TB
    subgraph TAILSCALE["🌐 Tailscale Mesh VPN"]
        direction TB

        subgraph RPI["🍓 Raspberry Pi 5 - Control Plane"]
            direction TB
            NOMAD_SERVER["📦 Nomad Server<br/>:4646-4648"]
            CONSUL_SERVER["🔍 Consul Server<br/>:8500"]
            CLAUDE_FLOW["🧠 Claude-Flow<br/>:8080"]

            NOMAD_SERVER <--> CONSUL_SERVER
            CLAUDE_FLOW --> CONSUL_SERVER
            CLAUDE_FLOW --> NOMAD_SERVER
        end

        subgraph MAC["💻 MacBook Pro M1 MAX - Data Plane"]
            direction TB
            NOMAD_CLIENT["📦 Nomad Client"]
            CONSUL_CLIENT["🔍 Consul Client"]
            OLLAMA["🤖 Ollama<br/>:11434"]
            AGENTS["⚡ Agent Workers"]

            NOMAD_CLIENT <--> CONSUL_CLIENT
            AGENTS --> OLLAMA
            NOMAD_CLIENT --> AGENTS
        end

        NOMAD_SERVER <--"RPC"--> NOMAD_CLIENT
        CONSUL_SERVER <--"Gossip"--> CONSUL_CLIENT
        CLAUDE_FLOW --"Service Discovery"--> OLLAMA
    end

    USER["👤 User"] --> CLAUDE_FLOW
```

## Task Execution Flow

```mermaid
sequenceDiagram
    autonumber
    participant U as 👤 User
    participant CF as 🧠 Claude-Flow
    participant N as 📦 Nomad
    participant A as ⚡ Agent
    participant O as 🤖 Ollama

    U->>CF: Create coding task
    CF->>CF: Select agent type
    CF->>N: Submit batch job

    N->>N: Evaluate constraints
    N->>A: Create allocation

    loop Inference Loop
        A->>O: Send prompt
        O->>O: Run inference
        O->>A: Return completion
    end

    A->>N: Task complete
    N->>CF: Return results
    CF->>U: Display output

    N->>N: Garbage collect allocation
```

## Network Topology

```mermaid
graph LR
    subgraph Internet
        TS[("☁️ Tailscale<br/>Coordination")]
    end

    subgraph Home["🏠 Home Network"]
        subgraph VLAN1["Control VLAN"]
            RPI["🍓 RPi5<br/>100.x.x.1"]
        end

        subgraph VLAN2["Compute VLAN"]
            MAC["💻 MacBook<br/>100.x.x.2"]
        end
    end

    RPI <--"WireGuard<br/>Encrypted"--> TS
    MAC <--"WireGuard<br/>Encrypted"--> TS
    RPI <--"Direct<br/>Connection"--> MAC

    style TS fill:#0ea5e9,stroke:#0284c7,color:#fff
    style RPI fill:#4f46e5,stroke:#3730a3,color:#fff
    style MAC fill:#059669,stroke:#047857,color:#fff
```

## Service Discovery

```mermaid
graph TB
    subgraph Consul["🔍 Consul Service Mesh"]
        direction LR

        subgraph Services["Registered Services"]
            S1["ollama.service.consul<br/>→ 100.x.x.2:11434"]
            S2["claude-flow.service.consul<br/>→ 100.x.x.1:8080"]
            S3["nomad.service.consul<br/>→ 100.x.x.1:4646"]
        end

        subgraph Health["Health Checks"]
            H1["✓ HTTP /api/tags"]
            H2["✓ HTTP /health"]
            H3["✓ TCP :4646"]
        end

        S1 --- H1
        S2 --- H2
        S3 --- H3
    end

    CF["Claude-Flow"] --"DNS Query"--> Consul
    CF --"ollama.service.consul"--> OLLAMA["Ollama"]
```

## Agent Types & Models

```mermaid
graph LR
    subgraph Agents["🤖 Agent Types"]
        A1["👨‍💻 Coder"]
        A2["🔍 Reviewer"]
        A3["🏗️ Architect"]
        A4["🐛 Debugger"]
        A5["📝 Documenter"]
        A6["🧪 Tester"]
    end

    subgraph Models["🧠 LLM Models"]
        M1["deepseek-coder:33b"]
        M2["codellama:34b"]
        M3["llama3.2:70b"]
        M4["qwen2.5-coder:32b"]
    end

    A1 --> M1
    A2 --> M2
    A3 --> M3
    A4 --> M1
    A5 --> M4
    A6 --> M2

    style A1 fill:#4f46e5,color:#fff
    style A2 fill:#059669,color:#fff
    style A3 fill:#dc2626,color:#fff
    style A4 fill:#ea580c,color:#fff
    style A5 fill:#0ea5e9,color:#fff
    style A6 fill:#8b5cf6,color:#fff
```

## Resource Allocation

```mermaid
pie showData
    title "MacBook M1 MAX Resource Distribution (96GB)"
    "OS & System" : 8
    "Ollama Overhead" : 8
    "Model in Memory" : 40
    "Agent Tasks (4x)" : 32
    "Reserved" : 8
```

## Nomad Job States

```mermaid
stateDiagram-v2
    [*] --> Pending: Job Submitted

    Pending --> Running: Allocation Created
    Running --> Complete: Task Finished
    Running --> Failed: Task Error

    Failed --> Pending: Retry (max 2)
    Failed --> Dead: Max Retries

    Complete --> Dead: GC After 4h
    Dead --> [*]

    note right of Running
        Agent executing
        Calling Ollama
    end note

    note right of Complete
        Results returned
        to Claude-Flow
    end note
```

## Deployment Pipeline

```mermaid
flowchart LR
    subgraph Setup["📦 Initial Setup"]
        direction TB
        S1["1. Generate<br/>Tailscale Key"]
        S2["2. Setup RPi<br/>Control Plane"]
        S3["3. Setup Mac<br/>Data Plane"]
        S1 --> S2 --> S3
    end

    subgraph Deploy["🚀 Deployment"]
        direction TB
        D1["4. Pull<br/>Models"]
        D2["5. Deploy<br/>Stack"]
        D3["6. Verify<br/>Status"]
        D1 --> D2 --> D3
    end

    subgraph Use["✨ Usage"]
        direction TB
        U1["7. Access<br/>Claude-Flow UI"]
        U2["8. Create<br/>Tasks"]
        U3["9. Review<br/>Results"]
        U1 --> U2 --> U3
    end

    Setup --> Deploy --> Use
```

## High Availability (Future)

```mermaid
graph TB
    subgraph Region1["🌍 Region 1"]
        RPI1["RPi5 Primary"]
        MAC1["MacBook 1"]
    end

    subgraph Region2["🌍 Region 2"]
        RPI2["RPi5 Secondary"]
        MAC2["MacBook 2"]
    end

    RPI1 <--"Raft<br/>Consensus"--> RPI2
    MAC1 <--"Workload<br/>Distribution"--> MAC2

    LB["Load Balancer"] --> RPI1
    LB --> RPI2

    style LB fill:#f59e0b,stroke:#d97706,color:#fff
```

## Component Interactions Matrix

```mermaid
graph TD
    subgraph Legend
        L1["→ HTTP/API"]
        L2["⇢ RPC"]
        L3["⇠ Gossip"]
    end

    subgraph Matrix["Interaction Matrix"]
        CF["Claude-Flow"]
        NS["Nomad Server"]
        NC["Nomad Client"]
        CS["Consul Server"]
        CC["Consul Client"]
        OL["Ollama"]
        AG["Agents"]

        CF -->|HTTP| NS
        CF -->|DNS| CS
        NS <-->|RPC| NC
        CS <-->|Gossip| CC
        NC -->|exec| AG
        AG -->|HTTP| OL
        NC -->|register| CC
    end
```

## Viewing These Diagrams

These diagrams use [Mermaid.js](https://mermaid.js.org/) syntax. You can view them:

1. **GitHub**: Renders automatically in markdown files
2. **VS Code**: Install "Markdown Preview Mermaid Support" extension
3. **Online**: Paste into [Mermaid Live Editor](https://mermaid.live/)
4. **CLI**: Use `mmdc` (mermaid-cli) to generate images:
   ```bash
   npm install -g @mermaid-js/mermaid-cli
   mmdc -i docs/diagrams.md -o docs/diagrams.png
   ```
