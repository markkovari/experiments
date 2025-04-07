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

resource "digitalocean_app" "toto_app_backend" {
  project_id = digitalocean_project.my_faviourite_project.id
  spec {
    name   = "todo-app-backend"
    region = "ams"
    service {
      name            = "todo-app-backend"
      source_dir      = "todo-api"
      dockerfile_path = "todo-api/Dockerfile"
      git {
        repo_clone_url = "https://github.com/markkovari/ngrx-todo"
        branch         = "main"
      }
    }
  }
}

resource "digitalocean_app" "toto_app_frontend" {
  project_id = digitalocean_project.my_faviourite_project.id
  spec {
    name   = "static-site-example"
    region = "ams"

    static_site {
      name          = "todo-app-frontend"
      build_command = "npm run build"
      source_dir    = "todo-app"
      output_dir    = "/dist/todo-app/browser/"

      git {
        repo_clone_url = "https://github.com/markkovari/ngrx-todo"
        branch         = "main"
      }
    }
  }
}
