resource "porkbun_dns_record" "autographs" {
  count = var.manage_autographs_dns ? 1 : 0

  domain  = var.autographs_dns_domain
  name    = var.autographs_dns_subdomain
  type    = "A"
  content = module.compute.public_ip
  ttl     = var.autographs_dns_ttl
}

import {
  to = porkbun_dns_record.autographs
  id = var.autographs_dns_record_id
}
