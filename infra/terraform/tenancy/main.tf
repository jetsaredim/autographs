data "oci_objectstorage_namespace" "ns" {
  compartment_id = var.tenancy_ocid
}

module "iam" {
  source = "../modules/iam"

  providers = {
    oci.home = oci.home
  }

  name_prefix                = var.name_prefix
  parent_compartment_ocid    = var.parent_compartment_ocid
  create_compartment         = var.create_compartment
  existing_compartment_ocid  = var.existing_compartment_ocid
  compartment_description    = var.compartment_description
  deploy_group_name          = var.deploy_group_name
  create_deploy_group        = var.create_deploy_group
  operator_group_name        = var.operator_group_name
  create_operator_group      = var.create_operator_group
  create_deploy_user         = var.create_deploy_user
  deploy_user_name           = var.deploy_user_name
  deploy_user_description    = var.deploy_user_description
  deploy_user_email          = var.deploy_user_email
  deploy_user_api_public_key = var.deploy_user_api_public_key
  tags                       = local.tags
}

module "state_bucket" {
  source = "../modules/state_bucket"

  create_bucket  = var.create_state_bucket
  compartment_id = module.iam.compartment_ocid
  namespace      = data.oci_objectstorage_namespace.ns.namespace
  bucket_name    = var.state_bucket_name
  storage_tier   = var.state_bucket_storage_tier
  tags           = local.tags
}
