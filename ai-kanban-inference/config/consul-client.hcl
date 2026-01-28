# Consul Client Configuration for MacBook Pro M1 MAX (Data Plane)
#
# Consul client for service registration and health checking
#
# Usage: Place in /etc/consul.d/consul.hcl

datacenter = "home"
data_dir   = "/opt/consul/data"
log_level  = "INFO"

# Node name (set during installation)
node_name = "macbook-data"

# Bind to Tailscale interface (utun* on macOS)
bind_addr = "{{ GetInterfaceIP \"utun\" }}"

# Client address - allow local connections
client_addr = "127.0.0.1"

# Client mode (not a server)
server = false

# Join the Consul server on RPi (replace with actual Tailscale hostname)
retry_join = ["rpi5-control.tailnet-name.ts.net"]

# Retry settings
retry_interval = "30s"
retry_max      = 0  # Retry indefinitely

# Ports configuration
ports {
  http  = 8500
  https = -1
  grpc  = 8502
}

# DNS configuration
dns_config {
  allow_stale = true
  max_stale   = "87600h"
}

# Service definitions can be added here or in separate files
# Example: Register Ollama as a service
# services {
#   name = "ollama"
#   port = 11434
#   check {
#     http     = "http://localhost:11434/api/tags"
#     interval = "30s"
#     timeout  = "5s"
#   }
# }

# Leave configuration
leave_on_terminate = true
skip_leave_on_interrupt = false

# ACL configuration (match server)
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
