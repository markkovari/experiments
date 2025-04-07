terraform {

  backend "s3" {
    bucket                      = "terraform-state"
    key                         = "global/s3/terraform.tfstate"
    region                      = "us-east-1"
    endpoint                    = "https://markkovari-terraform-dev-state.ams3.digitaloceanspaces.com"
    skip_credentials_validation = true
    skip_metadata_api_check     = true
    force_path_style            = true
  }
}
