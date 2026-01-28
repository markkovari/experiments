# Nomad Server Configuration for Raspberry Pi 5 (Control Plane)
#
# This configuration runs Nomad in both server and client mode:
# - Server: Schedules workloads across the cluster
# - Client: Can run lightweight control-plane tasks
#
# Usage: Place in /etc/nomad.d/nomad.hcl

datacenter = "home"
data_dir   = "/opt/nomad/data"
log_level  = "INFO"

# Bind to Tailscale interface for secure cluster communication
# The template function resolves the Tailscale IP at runtime
bind_addr = "{{ GetInterfaceIP \"tailscale0\" }}"

# Advertise addresses (replace with actual Tailscale IP during setup)
advertise {
  http = "{{ GetInterfaceIP \"tailscale0\" }}:4646"
  rpc  = "{{ GetInterfaceIP \"tailscale0\" }}:4647"
  serf = "{{ GetInterfaceIP \"tailscale0\" }}:4648"
}

# Server configuration
server {
  enabled          = true
  bootstrap_expect = 1  # Single-node cluster

  # Raft settings for single node
  raft_protocol = 3

  # Server join configuration for future expansion
  server_join {
    retry_max      = 3
    retry_interval = "15s"
  }

  # Garbage collection settings
  job_gc_interval           = "5m"
  job_gc_threshold          = "4h"
  eval_gc_threshold         = "1h"
  deployment_gc_threshold   = "1h"
  node_gc_threshold         = "24h"
}

# Client configuration (for control-plane tasks)
client {
  enabled    = true
  node_class = "control-plane"

  # Node metadata for constraints
  meta {
    role     = "control-plane"
    arch     = "arm64"
    platform = "raspberry-pi"
  }

  # Resource reservation for system
  reserved {
    cpu    = 200   # Reserve 200 MHz for OS
    memory = 512   # Reserve 512 MB for OS
  }
}

# Consul integration for service discovery
consul {
  address = "127.0.0.1:8500"

  # Service registration
  server_service_name = "nomad"
  client_service_name = "nomad-client"
  auto_advertise      = true

  # Health checks
  server_auto_join = true
  client_auto_join = true
}

# Enable raw_exec driver for running native processes
plugin "raw_exec" {
  config {
    enabled = true
  }
}

# ACL configuration (disabled for simplicity, enable in production)
acl {
  enabled = false
}

# Telemetry configuration
telemetry {
  collection_interval        = "10s"
  disable_hostname           = false
  prometheus_metrics         = true
  publish_allocation_metrics = true
  publish_node_metrics       = true
}

# Limits
limits {
  https_handshake_timeout   = "5s"
  http_max_conns_per_client = 200
  rpc_handshake_timeout     = "5s"
}
