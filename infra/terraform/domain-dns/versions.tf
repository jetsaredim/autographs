terraform {
  required_version = ">= 1.17.1, < 1.18.0"

  required_providers {
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 5.28.0"
    }
  }

  backend "oci" {}
}
