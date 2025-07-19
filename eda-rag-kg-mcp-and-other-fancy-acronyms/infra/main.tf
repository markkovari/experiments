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
  # credentials = "/home/you/ngs_stream_admin.creds"
  # tls {
  # ca_file_data = "<Root CA PEM data>"
  #}
}

variable "messageRetentionTime" {
  type        = number
  default     = 60 * 60 * 24
  description = "Maximum time how much a message can live"
}


locals {
  mainSubjectName    = "LISTINGS"
  priorityCategories = ["low", "medium", "high"]
}


resource "jetstream_kv_bucket" "user_notifiations" {
  name           = "user_notifications"
  description    = "User notifications bucket"
  placement_tags = ["user", "notification"]

}
# resource "jetstream_stream" "LISTINGS" {
#   name     = local.mainSubjectName
#   subjects = [format("%s.*.*", local.mainSubjectName)]
#   storage  = "file"
#   max_age  = var.messageRetentionTime
# }


# resource "jetstream_consumer" "LISTINGS_CREATE" {
#   stream_id       = jetstream_stream.LISTINGS.id
#   ack_policy      = "explicit"
#   durable_name    = format("%s_create", local.mainSubjectName)
#   description     = "Consume the newly created listings"
#   deliver_all     = true
#   filter_subject  = format("%s.create", local.mainSubjectName)
#   sample_freq     = 100
#   max_ack_pending = 10
#   max_delivery    = 5
# }
