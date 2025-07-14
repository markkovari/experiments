terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "2.59.0"
    }
    google = {
      source  = "hashicorp/google"
      version = "6.43.0"
    }
  }
}

provider "digitalocean" {
  # Configuration options
}

provider "google" {
  # Configuration options
}
