terraform {
  required_version = ">= 1.17.0, < 1.18.0"

  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 9.11"
    }

    porkbun = {
      source  = "cullenmcdermott/porkbun"
      version = "~> 0.3"
    }
  }

  backend "oci" {}
}
