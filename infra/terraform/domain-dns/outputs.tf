output "cloudflare_zone_id" {
  description = "Zone ID to pass to the application runtime Terraform root."
  value       = cloudflare_zone.domain.id
}

output "cloudflare_nameservers" {
  description = "Assigned nameservers for the controlled Porkbun registrar handoff."
  value       = cloudflare_zone.domain.name_servers
}
