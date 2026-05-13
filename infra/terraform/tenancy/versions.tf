terraform {
  required_version = ">= 1.12.0, < 1.16.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 6.0"
    }
  }

  backend "oci" {}
}
