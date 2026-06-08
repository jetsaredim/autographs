data "oci_objectstorage_namespace" "ns" {
  compartment_id = var.tenancy_ocid
}

module "iam" {
  source = "../modules/iam"

  providers = {
    oci.home = oci.home
  }

  name_prefix                    = var.name_prefix
  parent_compartment_ocid        = var.parent_compartment_ocid
  tenancy_ocid                   = var.tenancy_ocid
  create_compartment             = var.create_compartment
  existing_compartment_ocid      = var.existing_compartment_ocid
  compartment_description        = var.compartment_description
  deploy_group_name              = var.deploy_group_name
  create_deploy_group            = var.create_deploy_group
  operator_group_name            = var.operator_group_name
  create_operator_group          = var.create_operator_group
  create_deploy_user             = var.create_deploy_user
  runtime_dynamic_group_name     = var.runtime_dynamic_group_name
  runtime_instance_ocid          = var.runtime_instance_ocid
  admin_runtime_group_name       = var.admin_runtime_group_name
  admin_runtime_user_name        = var.admin_runtime_user_name
  admin_runtime_user_description = var.admin_runtime_user_description
  admin_runtime_user_email       = var.admin_runtime_user_email
  admin_access_key_secret_name   = var.admin_access_key_secret_name
  admin_secret_key_secret_name   = var.admin_secret_key_secret_name
  deploy_user_name               = var.deploy_user_name
  deploy_user_description        = var.deploy_user_description
  deploy_user_email              = var.deploy_user_email
  deploy_user_api_public_key     = var.deploy_user_api_public_key
  media_bucket_name              = var.media_bucket_name
  tags                           = local.tags
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
