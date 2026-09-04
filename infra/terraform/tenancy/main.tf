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
  tenancy_ocid               = var.tenancy_ocid
  compartment_description    = var.compartment_description
  deploy_group_name          = var.deploy_group_name
  operator_group_name        = var.operator_group_name
  runtime_dynamic_group_name = var.runtime_dynamic_group_name
  deploy_user_name           = var.deploy_user_name
  deploy_user_description    = var.deploy_user_description
  deploy_user_email          = var.deploy_user_email
  deploy_user_api_public_key = var.deploy_user_api_public_key
  media_bucket_name          = var.media_bucket_name
  state_bucket_name          = var.state_bucket_name
  tags                       = local.tags
}

module "state_bucket" {
  source = "../modules/state_bucket"

  compartment_id = module.iam.compartment_ocid
  namespace      = data.oci_objectstorage_namespace.ns.namespace
  bucket_name    = var.state_bucket_name
  storage_tier   = var.state_bucket_storage_tier
  tags           = local.tags
}

resource "oci_identity_policy" "runtime_secret_bundle_access" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-runtime-secret-bundle-access-policy"
  description    = "Allows Autographs runtime instance principals to read Terraform-managed OCI Vault secret bundles."
  statements = [
    for secret_name in values(local.runtime_controller_secret_names) :
    "Allow dynamic-group id ${module.iam.runtime_dynamic_group_id} to read secret-bundles in compartment id ${module.iam.compartment_ocid} where target.secret.name = '${secret_name}'"
  ]

  freeform_tags = local.tags
}

resource "oci_identity_policy" "database_secret_access" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-database-secret-access-policy"
  description    = "Allows project Vault secret resource principals to read the generated Autonomous Database password secret during coordinated rotation."
  statements = [
    "Allow any-user to read secret-family in compartment id ${module.iam.compartment_ocid} where all {request.principal.type = 'vaultsecret', request.principal.compartment.id = '${module.iam.compartment_ocid}', target.secret.name = '${local.runtime_controller_secret_names.oracle_db_password}'}"
  ]

  freeform_tags = local.tags
}
