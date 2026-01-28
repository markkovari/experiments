# Nomad Client Configuration for MacBook Pro M1 MAX (Data Plane)
#
# This configuration runs Nomad in client-only mode:
# - Executes workloads scheduled by the server
# - Provides GPU/ML inference capabilities via Ollama
#
# Usage: Place in /etc/nomad.d/nomad.hcl

datacenter = "home"
data_dir   = "/opt/nomad/data"
log_level  = "INFO"

# Bind to Tailscale interface (utun* on macOS)
# Note: macOS uses "utun" prefix for Tailscale
bind_addr = "{{ GetInterfaceIP \"utun\" }}"

# Advertise addresses
advertise {
  http = "{{ GetInterfaceIP \"utun\" }}:4646"
  rpc  = "{{ GetInterfaceIP \"utun\" }}:4647"
  serf = "{{ GetInterfaceIP \"utun\" }}:4648"
}

# Client-only mode (no server)
client {
  enabled    = true
  node_class = "data-plane"

  # Server addresses (replace with actual RPi Tailscale hostname)
  servers = ["rpi5-control.tailnet-name.ts.net:4647"]

  # Node metadata for constraints and affinity rules
  meta {
    role      = "data-plane"
    arch      = "arm64"
    platform  = "macos"
    gpu       = "apple-silicon"
    gpu_cores = "32"      # M1 MAX GPU cores
    memory    = "96gb"
    ollama    = "true"
  }

  # Host volume for Ollama models (read-only access from jobs)
  host_volume "ollama-models" {
    path      = "/Users/REPLACE_USERNAME/.ollama/models"
    read_only = true
  }

  # Host volume for shared workspaces (if needed)
  host_volume "workspaces" {
    path      = "/Users/REPLACE_USERNAME/workspaces"
    read_only = false
  }

  # Resource reservation for macOS and Ollama
  reserved {
    cpu    = 2000    # Reserve 2 GHz for OS + Ollama overhead
    memory = 16384   # Reserve 16 GB for OS + Ollama models
  }

  # Network configuration
  network_interface = "utun"  # Tailscale interface
}

# Consul integration
consul {
  address = "127.0.0.1:8500"

  client_service_name = "nomad-client"
  auto_advertise      = true
  client_auto_join    = true
}

# Enable raw_exec driver for running native processes
# This is required for running Ollama and Claude-Flow commands
plugin "raw_exec" {
  config {
    enabled = true
  }
}

# ACL configuration (match server settings)
acl {
  enabled = false
}

# Telemetry
telemetry {
  collection_interval        = "10s"
  disable_hostname           = false
  prometheus_metrics         = true
  publish_allocation_metrics = true
  publish_node_metrics       = true
}

# Client-specific limits
limits {
  https_handshake_timeout   = "5s"
  http_max_conns_per_client = 200
  rpc_handshake_timeout     = "5s"
}
