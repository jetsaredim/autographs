terraform {
  required_version = ">= 1.16.3, < 1.17.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 9.2"
    }
  }

  backend "oci" {}
}
