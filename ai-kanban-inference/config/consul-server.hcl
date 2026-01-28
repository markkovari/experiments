# Consul Server Configuration for Raspberry Pi 5 (Control Plane)
#
# Single-node Consul server for service discovery and health checking
#
# Usage: Place in /etc/consul.d/consul.hcl

datacenter = "home"
data_dir   = "/opt/consul/data"
log_level  = "INFO"

# Node name (set during installation)
node_name = "rpi5-control"

# Bind to Tailscale interface
bind_addr = "{{ GetInterfaceIP \"tailscale0\" }}"

# Client address - allow connections from any interface
client_addr = "0.0.0.0"

# Server mode configuration
server           = true
bootstrap_expect = 1  # Single-node cluster

# UI configuration
ui_config {
  enabled = true
}

# Service mesh (Connect)
connect {
  enabled = true
}

# Ports configuration
ports {
  http  = 8500
  https = -1     # Disable HTTPS (using Tailscale for encryption)
  grpc  = 8502
  dns   = 8600
}

# DNS configuration for service discovery
dns_config {
  allow_stale         = true
  max_stale           = "87600h"  # Allow very stale reads (single node)
  node_ttl            = "30s"
  service_ttl {
    "*" = "30s"
  }
  enable_truncate     = true
  only_passing        = false
}

# Performance tuning for Raspberry Pi
performance {
  raft_multiplier = 5  # Increase for slower hardware
}

# Leave configuration
leave_on_terminate = true
skip_leave_on_interrupt = false

# Retry join (empty for bootstrap, will be populated for multi-node)
retry_join = []

# ACL configuration (disabled for simplicity)
acl {
  enabled        = false
  default_policy = "allow"
}

# Telemetry
telemetry {
  prometheus_retention_time = "60s"
  disable_hostname          = false
}

# Logging
log_json = false
enable_syslog = false
