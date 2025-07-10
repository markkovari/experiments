terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }

  backend "s3" {

    # Deactivate a few AWS-specific checks
    skip_region_validation = true
    skip_s3_checksum       = true
    # Replace with your desired Space name
    bucket = "do-devops2-bucket"

    key = "environments/dev/digitalocean_project.tfstate"
    # This key specifies the path and filename of your state file within the bucket

    # Replace with your Space region endpoint. 
    # Example: nyc3.digitaloceanspaces.com, fra1.digitaloceanspaces.com
    endpoints = {
      s3 = "https://do-devops2-bucket.fra1.digitaloceanspaces.com"
    }
    workspace_key_prefix = "envs/dev"
    # As DigitalOcean Spaces uses an S3-compatible API, 
    # you might need to skip some AWS-specific checks.
    skip_credentials_validation = true
    skip_requesting_account_id  = true
    skip_metadata_api_check     = true

    # DigitalOcean Spaces does not use AWS regions directly for the backend.
    # However, the S3 backend requires a 'region' argument.
    # You can set it to any valid AWS region, as it will be ignored by DO Spaces.
    region         = "us-east-1"
    use_path_style = true
    use_lockfile   = true
  }
}

# Configure the DigitalOcean Provider
provider "digitalocean" {
  # The token will be picked up from the environment variable DIGITALOCEAN_TOKEN or DO_TOKEN
  # token = var.do_token # If you prefer to explicitly define it as a variable
}






# Define a DigitalOcean Project
resource "digitalocean_project" "my_project" {
  name        = "My Awesome Terraform Project"
  description = "A project to manage my DigitalOcean resources with Terraform."
  purpose     = "Web Application"
  environment = "Development"
  # You can associate existing resources here using their URNs if needed, e.g.:
  # resources   = [digitalocean_droplet.example.urn] 
}

/*
# Example of a resource you might add to the project later
resource "digitalocean_droplet" "example" {
  name   = "my-example-droplet"
  size   = "s-1vcpu-1gb"
  image  = "ubuntu-22-04-x64"
  region = "nyc3"
}
*/

output "project_id" {
  description = "The ID of the created DigitalOcean Project"
  value       = digitalocean_project.my_project.id
}

output "project_urn" {
  description = "The URN of the created DigitalOcean Project"
  value       = digitalocean_project.my_project.urn
}
