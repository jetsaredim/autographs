output "compartment_ocid" {
  value = local.compartment_ocid
}

output "deploy_policy_name" {
  value = oci_identity_policy.deploy.name
}

output "operator_policy_name" {
  value = oci_identity_policy.operator.name
}
