terraform {
  required_version = "~> 1.11.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 6.0"
    }
  }

  backend "oci" {}
}
