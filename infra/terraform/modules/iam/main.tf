moved {
  from = oci_identity_group.deploy[0]
  to   = oci_identity_group.deploy
}

moved {
  from = oci_identity_group.operator[0]
  to   = oci_identity_group.operator
}

moved {
  from = oci_identity_user.deploy[0]
  to   = oci_identity_user.deploy
}

moved {
  from = oci_identity_user_group_membership.deploy[0]
  to   = oci_identity_user_group_membership.deploy
}

moved {
  from = oci_identity_compartment.project[0]
  to   = oci_identity_compartment.project
}

resource "oci_identity_compartment" "project" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-cmp"
  description    = var.compartment_description
  enable_delete  = false

  freeform_tags = var.tags
}

resource "oci_identity_group" "deploy" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = var.deploy_group_name
  description    = "Least-privilege deployment automation group for Autographs."

  freeform_tags = var.tags
}

resource "oci_identity_group" "operator" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = var.operator_group_name
  description    = "Human operator group for Autographs day-two management."

  freeform_tags = var.tags
}

resource "oci_identity_user" "deploy" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = var.deploy_user_name
  description    = var.deploy_user_description
  email          = var.deploy_user_email != "" ? var.deploy_user_email : null

  freeform_tags = var.tags
}

resource "oci_identity_user_group_membership" "deploy" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  group_id       = oci_identity_group.deploy.id
  user_id        = oci_identity_user.deploy.id
}

resource "oci_identity_dynamic_group" "runtime_instances" {
  provider       = oci.home
  compartment_id = var.tenancy_ocid
  name           = var.runtime_dynamic_group_name
  description    = "Autographs runtime VM instances allowed to use instance principal authentication."
  # Keep this scoped to the project compartment to avoid a circular dependency
  # between tenancy IAM and the runtime instance created by the deployment root.
  matching_rule = "ALL {instance.compartment.id = '${local.compartment_ocid}'}"

  freeform_tags = var.tags
}

resource "oci_identity_api_key" "deploy" {
  provider  = oci.home
  count     = var.deploy_user_api_public_key != "" ? 1 : 0
  user_id   = oci_identity_user.deploy.id
  key_value = var.deploy_user_api_public_key
}

locals {
  compartment_ocid      = oci_identity_compartment.project.id
  deploy_group          = "group id ${oci_identity_group.deploy.id}"
  operator_group        = "group id ${oci_identity_group.operator.id}"
  runtime_dynamic_group = "dynamic-group id ${oci_identity_dynamic_group.runtime_instances.id}"

  deploy_policy_statements = [
    "Allow ${local.deploy_group} to inspect compartments in tenancy",
    "Allow ${local.deploy_group} to read objectstorage-namespaces in tenancy",
    "Allow ${local.deploy_group} to manage instance-family in compartment id ${local.compartment_ocid}",
    "Allow ${local.deploy_group} to manage virtual-network-family in compartment id ${local.compartment_ocid}",
    "Allow ${local.deploy_group} to manage volume-family in compartment id ${local.compartment_ocid}",
    "Allow ${local.deploy_group} to manage autonomous-database-family in compartment id ${local.compartment_ocid}",
    "Allow ${local.deploy_group} to manage buckets in compartment id ${local.compartment_ocid}",
    "Allow ${local.deploy_group} to manage objects in compartment id ${local.compartment_ocid} where target.bucket.name = '${var.state_bucket_name}'"
  ]

  operator_policy_statements = [
    "Allow ${local.operator_group} to inspect compartments in tenancy",
    "Allow ${local.operator_group} to read objectstorage-namespaces in tenancy",
    "Allow ${local.operator_group} to inspect all-resources in compartment id ${local.compartment_ocid}",
    "Allow ${local.operator_group} to manage instance-family in compartment id ${local.compartment_ocid}",
    "Allow ${local.operator_group} to manage virtual-network-family in compartment id ${local.compartment_ocid}",
    "Allow ${local.operator_group} to manage volume-family in compartment id ${local.compartment_ocid}",
    "Allow ${local.operator_group} to manage autonomous-database-family in compartment id ${local.compartment_ocid}",
    "Allow ${local.operator_group} to manage buckets in compartment id ${local.compartment_ocid}",
    "Allow ${local.operator_group} to manage objects in compartment id ${local.compartment_ocid} where target.bucket.name = '${var.state_bucket_name}'",
    "Allow ${local.operator_group} to manage objects in compartment id ${local.compartment_ocid} where target.bucket.name = '${var.media_bucket_name}'"
  ]

  runtime_object_access_policy_statements = [
    "Allow ${local.runtime_dynamic_group} to read objectstorage-namespaces in tenancy",
    "Allow ${local.runtime_dynamic_group} to read buckets in compartment id ${local.compartment_ocid} where target.bucket.name = '${var.media_bucket_name}'",
    "Allow ${local.runtime_dynamic_group} to manage objects in compartment id ${local.compartment_ocid} where target.bucket.name = '${var.media_bucket_name}'"
  ]
}

resource "oci_identity_policy" "deploy" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-deploy-policy"
  description    = "Least-privilege policy seam for GitHub-driven deployment automation."
  statements     = local.deploy_policy_statements

  freeform_tags = var.tags
}

resource "oci_identity_policy" "operator" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-operator-policy"
  description    = "Operator policy seam for human break-glass and day-two management."
  statements     = local.operator_policy_statements

  freeform_tags = var.tags
}

resource "oci_identity_policy" "runtime_object_access" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-runtime-object-access-policy"
  description    = "Allows Autographs runtime instance principals to access private media objects."
  statements     = local.runtime_object_access_policy_statements

  freeform_tags = var.tags
}
