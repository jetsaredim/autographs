output "autonomous_database_id" {
  description = "Oracle Autonomous Database OCID, when created."
  value       = var.create_autonomous_database ? oci_database_autonomous_database.catalog[0].id : null
}

output "autonomous_database_name" {
  description = "Oracle Autonomous Database DB name."
  value       = var.autonomous_database_name
}

output "media_bucket_name" {
  description = "Private Object Storage media bucket name."
  value       = var.media_bucket_name
}

output "media_bucket_namespace" {
  description = "Object Storage namespace for the media bucket."
  value       = var.media_bucket_namespace
}

output "controller_vault_id" {
  description = "OCI Vault OCID used by the private controller for runtime secrets."
  value       = oci_kms_vault.controller.id
}

output "controller_vault_key_id" {
  description = "OCI Vault key OCID used to encrypt private controller runtime secrets."
  value       = oci_kms_key.controller.id
}

output "controller_s3_access_key_secret_name" {
  description = "OCI Vault secret name for the controller OCI S3 access key."
  value       = oci_vault_secret.controller_s3_access_key.secret_name
}

output "controller_s3_secret_key_secret_name" {
  description = "OCI Vault secret name for the controller OCI S3 secret key."
  value       = oci_vault_secret.controller_s3_secret_key.secret_name
}
