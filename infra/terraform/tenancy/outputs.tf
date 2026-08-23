output "compartment_ocid" {
  description = "OCI compartment OCID that owns the project resources."
  value       = module.iam.compartment_ocid
}

output "deploy_policy_name" {
  description = "OCI policy name intended for routine deployment automation."
  value       = module.iam.deploy_policy_name
}

output "operator_policy_name" {
  description = "OCI policy name intended for the human operator."
  value       = module.iam.operator_policy_name
}

output "runtime_object_access_policy_name" {
  description = "OCI policy name allowing runtime instance principals to access private media objects."
  value       = module.iam.runtime_object_access_policy_name
}

output "runtime_secret_bundle_access_policy_name" {
  description = "OCI policy name allowing runtime instance principals to read approved Vault secret bundles."
  value       = module.iam.runtime_secret_bundle_access_policy_name
}

output "runtime_secrets_vault_id" {
  description = "OCI Vault OCID for Autographs runtime secret bundles."
  value       = oci_kms_vault.runtime_secrets.id
}

output "runtime_secrets_key_id" {
  description = "OCI KMS key OCID used by Autographs runtime Vault secrets."
  value       = oci_kms_key.runtime_secrets.id
}

output "runtime_vault_proof_secret_id" {
  description = "Disposable proof secret OCID for runtime instance-principal Vault retrieval."
  value       = oci_vault_secret.runtime_vault_proof.id
}

output "deploy_group_id" {
  description = "OCI group OCID for GitHub deployment automation when created by this root."
  value       = module.iam.deploy_group_id
}

output "operator_group_id" {
  description = "OCI group OCID for human operators when created by this root."
  value       = module.iam.operator_group_id
}

output "deploy_user_id" {
  description = "OCI user OCID for GitHub deployment automation when created by this root."
  value       = module.iam.deploy_user_id
}

output "runtime_dynamic_group_id" {
  description = "OCI dynamic group OCID for Autographs runtime VM instance principals."
  value       = module.iam.runtime_dynamic_group_id
}

output "runtime_dynamic_group_name" {
  description = "OCI dynamic group name for Autographs runtime VM instance principals."
  value       = module.iam.runtime_dynamic_group_name
}

output "deploy_api_key_fingerprint" {
  description = "API signing key fingerprint when deploy_user_api_public_key is attached."
  value       = module.iam.deploy_api_key_fingerprint
}

output "state_bucket_name" {
  description = "Object Storage bucket name used for Terraform remote state."
  value       = module.state_bucket.bucket_name
}

output "object_storage_namespace" {
  description = "OCI Object Storage namespace for the tenancy."
  value       = data.oci_objectstorage_namespace.ns.namespace
}

output "app_state_backend_key" {
  description = "Backend key for the runtime/app state object."
  value       = "envs/prod/terraform.tfstate"
}

output "tenancy_state_backend_key" {
  description = "Backend key for the tenancy bootstrap state object."
  value       = "envs/prod/tenancy-bootstrap.tfstate"
}
