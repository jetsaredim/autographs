output "compartment_ocid" {
  description = "OCI compartment OCID that owns the project resources."
  value       = var.compartment_ocid
}

output "runtime_public_ip" {
  description = "Public IP address assigned to the OCI runtime VM."
  value       = module.compute.public_ip
}

output "runtime_private_ip" {
  description = "Private IP address assigned to the OCI runtime VM."
  value       = module.compute.private_ip
}

output "runtime_instance_id" {
  description = "OCI instance OCID for the runtime VM."
  value       = module.compute.instance_id
}

output "vcn_id" {
  description = "OCI VCN ID for the Phase 1 runtime network baseline."
  value       = module.network.vcn_id
}

output "public_subnet_id" {
  description = "OCI subnet ID used for the single Phase 1 public runtime subnet."
  value       = module.network.public_subnet_id
}

output "runtime_nsg_id" {
  description = "OCI network security group ID protecting the runtime VM."
  value       = module.network.runtime_nsg_id
}

output "autonomous_database_id" {
  description = "Oracle Autonomous Database OCID for the metadata store, when created."
  value       = module.data_services.autonomous_database_id
}

output "autonomous_database_name" {
  description = "Oracle Autonomous Database DB name used by wallet aliases and connection strings."
  value       = module.data_services.autonomous_database_name
}

output "media_bucket_name" {
  description = "Private OCI Object Storage bucket name for autograph images."
  value       = module.data_services.media_bucket_name
}

output "media_bucket_namespace" {
  description = "OCI Object Storage namespace for the private media bucket."
  value       = module.data_services.media_bucket_namespace
}

output "autographs_dns_fqdn" {
  description = "DNS name managed for the autographs runtime."
  value       = var.manage_autographs_dns ? "${var.autographs_dns_subdomain}.${var.autographs_dns_domain}" : ""
}
