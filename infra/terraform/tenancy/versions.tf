terraform {
  required_version = ">= 1.16.1, < 1.17.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 8.29"
    }
  }

  backend "oci" {}
}
