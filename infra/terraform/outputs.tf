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

output "runtime_secret_id_env_vars" {
  description = "Controller environment variable names mapped to deployment-managed OCI Vault secret OCIDs after the operator marks all runtime secret values ready."
  value = {
    for env_name, secret_id in local.runtime_secret_id_env_vars : env_name => secret_id
    if var.runtime_secrets_ready
  }
}

output "runtime_secrets_vault_id" {
  description = "OCI Vault OCID for Autographs runtime secret bundles."
  value       = oci_kms_vault.runtime_secrets.id
}

output "runtime_secrets_key_id" {
  description = "OCI KMS key OCID used by Autographs runtime Vault secrets."
  value       = oci_kms_key.runtime_secrets.id
}

output "runtime_secret_ids" {
  description = "Deployment-managed Autographs runtime Vault secret OCIDs keyed by runtime secret name after the operator marks all runtime secret values ready."
  value = {
    for name, secret in oci_vault_secret.runtime : name => secret.id
    if var.runtime_secrets_ready
  }
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
  value       = "${var.autographs_dns_subdomain}.${var.autographs_dns_domain}"
}
