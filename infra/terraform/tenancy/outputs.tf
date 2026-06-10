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

output "runtime_secret_reader_policy_name" {
  description = "OCI policy name allowing runtime instance principals to read Vault secret bundles."
  value       = module.iam.runtime_secret_reader_policy_name
}

output "admin_runtime_object_access_policy_name" {
  description = "OCI policy name allowing the admin runtime IAM user to access private media objects."
  value       = module.iam.admin_runtime_object_access_policy_name
}

output "deploy_group_id" {
  description = "OCI group OCID for GitHub deployment automation when created by this root."
  value       = module.iam.deploy_group_id
}

output "operator_group_id" {
  description = "OCI group OCID for human operators when created by this root."
  value       = module.iam.operator_group_id
}

output "admin_runtime_group_id" {
  description = "OCI group OCID granting the private admin runtime IAM user media object access."
  value       = module.iam.admin_runtime_group_id
}

output "deploy_user_id" {
  description = "OCI user OCID for GitHub deployment automation when created by this root."
  value       = module.iam.deploy_user_id
}

output "admin_runtime_user_id" {
  description = "OCI IAM user OCID whose Customer Secret credentials are used by the private admin runtime."
  value       = module.iam.admin_runtime_user_id
}

output "admin_runtime_user_name" {
  description = "OCI IAM user name whose Customer Secret credentials are used by the private admin runtime."
  value       = module.iam.admin_runtime_user_name
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
