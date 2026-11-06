variable "cloudflare_account_id" {
  description = "Cloudflare account ID that will own the jetsaredim.net full zone."
  type        = string
}

variable "domain_name" {
  description = "Domain whose authoritative DNS moves to Cloudflare."
  type        = string
  default     = "jetsaredim.net"

  validation {
    condition     = var.domain_name == "jetsaredim.net"
    error_message = "This root is scoped to the jetsaredim.net domain."
  }
}

variable "dns_records" {
  description = "Complete reviewed Porkbun inventory except autographs; keys are stable Terraform labels."
  type = map(object({
    name     = string
    type     = string
    content  = string
    ttl      = number
    priority = optional(number)
  }))

  validation {
    condition = length(var.dns_records) > 0 && alltrue([
      for record in values(var.dns_records) :
      !(record.name == "autographs.jetsaredim.net" && record.type == "A") &&
      (record.name == "jetsaredim.net" || endswith(record.name, ".jetsaredim.net"))
      ]) && anytrue([
      for record in values(var.dns_records) :
      record.name == "jetsaredim.net" && record.type == "MX"
      ]) && anytrue([
      for record in values(var.dns_records) :
      record.name == "jetsaredim.net" && record.type == "TXT" && startswith(record.content, "v=spf1")
    ])
    error_message = "Provide a reviewed nonempty jetsaredim.net inventory with apex MX and SPF, excluding the app-owned autographs A record."
  }
}
