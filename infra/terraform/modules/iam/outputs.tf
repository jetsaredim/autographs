output "compartment_ocid" {
  value = local.compartment_ocid
}

output "deploy_policy_name" {
  value = oci_identity_policy.deploy.name
}

output "operator_policy_name" {
  value = oci_identity_policy.operator.name
}

output "runtime_object_access_policy_name" {
  value = oci_identity_policy.runtime_object_access.name
}

output "deploy_group_id" {
  value = oci_identity_group.deploy.id
}

output "operator_group_id" {
  value = oci_identity_group.operator.id
}

output "deploy_user_id" {
  value = oci_identity_user.deploy.id
}

output "runtime_dynamic_group_id" {
  value = oci_identity_dynamic_group.runtime_instances.id
}

output "runtime_dynamic_group_name" {
  value = var.runtime_dynamic_group_name
}

output "deploy_api_key_fingerprint" {
  value = var.deploy_user_api_public_key != "" ? oci_identity_api_key.deploy[0].fingerprint : null
}
