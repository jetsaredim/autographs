terraform {
  required_version = ">= 1.15.6, < 1.16.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 8.18"
    }
  }

  backend "oci" {}
}
