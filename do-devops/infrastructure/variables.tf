variable "do_token" {
  description = "Digital Ocean token"
  type        = string
  sensitive   = true
}


variable "do_region" {
  description = "digital ocean region"
  sensitive   = false
  validation {
    condition     = contains(["ams3"], var.do_region)
    error_message = "digital ocean region must be a valid do region"
  }
}

variable "do_spaces_access_key_id" {
  description = "digital ocean spaces access key"
  sensitive   = true
  validation {
    condition     = length(var.do_spaces_access_key_id) > 1
    error_message = "bucket access key must be at least one character long"
  }
}
variable "do_spaces_endpoint" {
  description = "digital ocean spaces access key"
  sensitive   = true
  validation {
    condition     = length(var.do_spaces_endpoint) > 1
    error_message = "bucket access key must be at least one character long"
  }
}



variable "do_spaces_secret_key" {
  description = "digital ocean spaces secret key"
  sensitive   = true
  validation {
    condition     = length(var.do_spaces_secret_key) > 1
    error_message = "bucket secret key must be at least one character long"
  }
}
