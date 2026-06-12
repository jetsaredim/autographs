resource "oci_identity_compartment" "project" {
  provider       = oci.home
  count          = var.create_compartment ? 1 : 0
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-cmp"
  description    = var.compartment_description
  enable_delete  = false

  freeform_tags = var.tags
}

resource "oci_identity_group" "deploy" {
  provider       = oci.home
  count          = var.create_deploy_group ? 1 : 0
  compartment_id = var.parent_compartment_ocid
  name           = var.deploy_group_name
  description    = "Least-privilege deployment automation group for Autographs."

  freeform_tags = var.tags
}

resource "oci_identity_group" "operator" {
  provider       = oci.home
  count          = var.create_operator_group ? 1 : 0
  compartment_id = var.parent_compartment_ocid
  name           = var.operator_group_name
  description    = "Human operator group for Autographs day-two management."

  freeform_tags = var.tags
}

resource "oci_identity_user" "deploy" {
  provider       = oci.home
  count          = var.create_deploy_user ? 1 : 0
  compartment_id = var.parent_compartment_ocid
  name           = var.deploy_user_name
  description    = var.deploy_user_description
  email          = var.deploy_user_email != "" ? var.deploy_user_email : null

  freeform_tags = var.tags
}

resource "oci_identity_user_group_membership" "deploy" {
  provider       = oci.home
  count          = var.create_deploy_user && var.create_deploy_group ? 1 : 0
  compartment_id = var.parent_compartment_ocid
  group_id       = oci_identity_group.deploy[0].id
  user_id        = oci_identity_user.deploy[0].id
}

resource "oci_identity_dynamic_group" "runtime_instances" {
  provider       = oci.home
  compartment_id = var.tenancy_ocid
  name           = var.runtime_dynamic_group_name
  description    = "Autographs runtime VM instances allowed to use instance principal authentication."
  matching_rule  = "ALL {instance.compartment.id = '${local.compartment_ocid}'}"

  freeform_tags = var.tags

  lifecycle {
    precondition {
      condition     = local.compartment_ocid != ""
      error_message = "Set existing_compartment_ocid when create_compartment is false, or keep create_compartment true so Terraform can manage the project compartment."
    }
  }
}

resource "oci_identity_api_key" "deploy" {
  provider  = oci.home
  count     = var.create_deploy_user && var.deploy_user_api_public_key != "" ? 1 : 0
  user_id   = oci_identity_user.deploy[0].id
  key_value = var.deploy_user_api_public_key
}

locals {
  compartment_ocid      = var.create_compartment ? oci_identity_compartment.project[0].id : var.existing_compartment_ocid
  deploy_group          = var.create_deploy_group ? "group id ${oci_identity_group.deploy[0].id}" : "group ${var.deploy_group_name}"
  operator_group        = var.create_operator_group ? "group id ${oci_identity_group.operator[0].id}" : "group ${var.operator_group_name}"
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
    "Allow ${local.runtime_dynamic_group} to read buckets in compartment id ${local.compartment_ocid}",
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

  lifecycle {
    precondition {
      condition     = local.compartment_ocid != ""
      error_message = "Set existing_compartment_ocid when create_compartment is false, or keep create_compartment true so Terraform can manage the project compartment."
    }
  }
}

resource "oci_identity_policy" "operator" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-operator-policy"
  description    = "Operator policy seam for human break-glass and day-two management."
  statements     = local.operator_policy_statements

  freeform_tags = var.tags

  lifecycle {
    precondition {
      condition     = local.compartment_ocid != ""
      error_message = "Set existing_compartment_ocid when create_compartment is false, or keep create_compartment true so Terraform can manage the project compartment."
    }
  }
}

resource "oci_identity_policy" "runtime_object_access" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-runtime-object-access-policy"
  description    = "Allows Autographs runtime instance principals to access private media objects."
  statements     = local.runtime_object_access_policy_statements

  freeform_tags = var.tags

  lifecycle {
    precondition {
      condition     = local.compartment_ocid != ""
      error_message = "Set existing_compartment_ocid when create_compartment is false, or keep create_compartment true so Terraform can manage the project compartment."
    }
  }
}
