# Registers the native Ollama service running on the MacBook
# as an external service in Consul for service discovery
job "ollama-registration" {
  datacenters = ["home"]
  type        = "service"

  # Run on data-plane (MacBook with Ollama)
  constraint {
    attribute = "${meta.role}"
    value     = "data-plane"
  }

  group "ollama" {
    count = 1

    # Keep this task running to maintain service registration
    restart {
      attempts = 10
      interval = "5m"
      delay    = "10s"
      mode     = "delay"
    }

    task "register" {
      driver = "raw_exec"

      # Simple keepalive process - the actual service registration
      # happens through the service block below
      config {
        command = "/bin/bash"
        args    = [
          "-c",
          "echo 'Ollama service registered'; while true; do sleep 3600; done"
        ]
      }

      # Register Ollama as a Consul service
      # This allows other services to discover Ollama via DNS:
      # ollama.service.consul:11434
      service {
        name = "ollama"
        port = 11434

        # Use the host's IP (Tailscale interface)
        address_mode = "host"

        tags = [
          "llm",
          "inference",
          "api",
          "gpu"
        ]

        meta {
          gpu     = "apple-silicon"
          memory  = "96gb"
          version = "latest"
        }

        # HTTP health check against Ollama API
        check {
          type     = "http"
          path     = "/api/tags"
          interval = "30s"
          timeout  = "5s"
        }

        # TCP health check as backup
        check {
          type     = "tcp"
          interval = "10s"
          timeout  = "2s"
        }
      }

      # Minimal resources since this is just for registration
      resources {
        cpu    = 10    # 10 MHz
        memory = 16    # 16 MB
      }
    }
  }
}
