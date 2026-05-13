output "compartment_ocid" {
  value = local.compartment_ocid
}

output "deploy_policy_name" {
  value = oci_identity_policy.deploy.name
}

output "operator_policy_name" {
  value = oci_identity_policy.operator.name
}

output "deploy_group_id" {
  value = var.create_deploy_group ? oci_identity_group.deploy[0].id : null
}

output "operator_group_id" {
  value = var.create_operator_group ? oci_identity_group.operator[0].id : null
}

output "deploy_user_id" {
  value = var.create_deploy_user ? oci_identity_user.deploy[0].id : null
}

output "deploy_api_key_fingerprint" {
  value = var.create_deploy_user && var.deploy_user_api_public_key != "" ? oci_identity_api_key.deploy[0].fingerprint : null
}
