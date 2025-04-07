terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.50.0"
    }
  }

}

provider "digitalocean" {
  token             = var.do_token
  spaces_endpoint   = var.do_spaces_endpoint
  spaces_access_id  = var.do_spaces_access_key_id
  spaces_secret_key = var.do_spaces_secret_key
}

resource "digitalocean_project" "my_faviourite_project" {
  name       = "my_faviourite_project_dev"
  purpose    = "development"
  is_default = true
}

resource "digitalocean_app" "static-site-example" {
  project_id = digitalocean_project.my_faviourite_project.id
  spec {
    name   = "static-site-example"
    region = "ams"

    static_site {
      name          = "sample-jekyll"
      build_command = "bundle exec jekyll build -d ./public"
      output_dir    = "/public"

      git {
        repo_clone_url = "https://github.com/digitalocean/sample-jekyll.git"
        branch         = "main"
      }
    }
  }
}
