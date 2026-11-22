terraform {
  required_version = ">= 1.17.2, < 1.18.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 9.14"
    }
  }

  backend "oci" {}
}
