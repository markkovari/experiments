terraform {
  required_providers {
    jetstream = {
      source  = "nats-io/jetstream"
      version = "0.2.1"
    }
  }
}
provider "jetstream" {
  servers = "localhost:4222"
}

resource "jetstream_kv_bucket" "user_notifications" {
  name        = "user_notifications"
  description = "Notifications for users"
}
