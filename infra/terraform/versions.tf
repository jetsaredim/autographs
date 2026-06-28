terraform {
  required_version = ">= 1.15.7, < 1.16.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 8.20"
    }

    porkbun = {
      source  = "cullenmcdermott/porkbun"
      version = "~> 0.3"
    }
  }

  backend "oci" {}
}
