terraform {
  required_version = ">= 1.15.8, < 1.16.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 8.24"
    }
  }

  backend "oci" {}
}
