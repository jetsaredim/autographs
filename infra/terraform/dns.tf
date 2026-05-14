resource "oci_dns_zone" "app" {
  count          = var.create_public_dns_zone ? 1 : 0
  compartment_id = var.compartment_ocid
  name           = var.public_dns_zone_name
  zone_type      = "PRIMARY"

  freeform_tags = local.tags
}

resource "oci_dns_rrset" "app_a" {
  count           = var.create_public_dns_zone ? 1 : 0
  zone_name_or_id = oci_dns_zone.app[0].id
  domain          = var.public_dns_record_name
  rtype           = "A"

  items {
    domain = var.public_dns_record_name
    rdata  = module.compute.public_ip
    rtype  = "A"
    ttl    = var.public_dns_record_ttl
  }

  lifecycle {
    precondition {
      condition     = module.compute.public_ip != null && module.compute.public_ip != ""
      error_message = "A runtime VM public IP is required before creating the public DNS A record."
    }

    precondition {
      condition     = var.public_dns_record_name == var.public_dns_zone_name || endswith(var.public_dns_record_name, ".${var.public_dns_zone_name}")
      error_message = "public_dns_record_name must be the public DNS zone apex or a subdomain of public_dns_zone_name."
    }
  }
}
