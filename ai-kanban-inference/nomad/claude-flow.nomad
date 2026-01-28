job "claude-flow" {
  datacenters = ["home"]
  type        = "service"

  # Run on control-plane (Raspberry Pi)
  constraint {
    attribute = "${meta.role}"
    value     = "control-plane"
  }

  group "server" {
    count = 1

    network {
      port "http" {
        static = 8080
      }
      port "mcp" {
        static = 8081
      }
    }

    # Restart policy
    restart {
      attempts = 3
      interval = "10m"
      delay    = "15s"
      mode     = "delay"
    }

    # Update strategy
    update {
      max_parallel     = 1
      min_healthy_time = "30s"
      healthy_deadline = "5m"
      auto_revert      = true
    }

    task "claude-flow" {
      driver = "raw_exec"

      config {
        command = "/usr/bin/npx"
        args    = [
          "claude-flow@alpha",
          "server",
          "--port", "${NOMAD_PORT_http}",
          "--mcp-port", "${NOMAD_PORT_mcp}"
        ]
      }

      env {
        # Ollama endpoint via Consul service discovery
        OLLAMA_HOST        = "http://ollama.service.consul:11434"

        # Claude-Flow configuration
        CLAUDE_FLOW_CONFIG = "/opt/claude-flow/config.json"

        # Node.js settings
        NODE_ENV           = "production"
        NODE_OPTIONS       = "--max-old-space-size=256"

        # Logging
        LOG_LEVEL          = "info"
      }

      resources {
        cpu    = 500   # 500 MHz
        memory = 256   # 256 MB
      }

      # Health check for HTTP endpoint
      service {
        name = "claude-flow"
        port = "http"
        tags = ["ui", "api", "agents"]

        check {
          type     = "http"
          path     = "/health"
          interval = "10s"
          timeout  = "2s"
        }

        check {
          type     = "http"
          path     = "/api/status"
          interval = "30s"
          timeout  = "5s"
        }
      }

      # MCP (Model Context Protocol) endpoint
      service {
        name = "claude-flow-mcp"
        port = "mcp"
        tags = ["mcp", "protocol"]

        check {
          type     = "tcp"
          interval = "30s"
          timeout  = "5s"
        }
      }

      # Graceful shutdown
      kill_timeout = "30s"

      # Logging configuration
      logs {
        max_files     = 5
        max_file_size = 10
      }
    }
  }
}
