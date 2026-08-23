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

resource "oci_kms_vault" "runtime_secrets" {
  compartment_id = module.iam.compartment_ocid
  display_name   = "${var.name_prefix}-runtime-secrets-vault"
  vault_type     = "DEFAULT"

  freeform_tags = local.tags
}

resource "oci_kms_key" "runtime_secrets" {
  compartment_id      = module.iam.compartment_ocid
  display_name        = "${var.name_prefix}-runtime-secrets-key"
  management_endpoint = oci_kms_vault.runtime_secrets.management_endpoint
  protection_mode     = "SOFTWARE"

  key_shape {
    algorithm = "AES"
    length    = 32
  }

  freeform_tags = local.tags
}

resource "oci_vault_secret" "runtime_vault_proof" {
  compartment_id         = module.iam.compartment_ocid
  description            = "Disposable Autographs proof secret for runtime instance-principal Vault retrieval."
  enable_auto_generation = true
  key_id                 = oci_kms_key.runtime_secrets.id
  secret_name            = "autographs-runtime-vault-proof"
  vault_id               = oci_kms_vault.runtime_secrets.id

  secret_generation_context {
    generation_template = "SECRETS_DEFAULT_PASSWORD"
    generation_type     = "PASSPHRASE"
    passphrase_length   = 32
  }

  freeform_tags = local.tags
}

resource "oci_identity_policy" "runtime_secret_bundle_access" {
  provider       = oci.home
  compartment_id = var.parent_compartment_ocid
  name           = "${var.name_prefix}-runtime-secret-bundle-access-policy"
  description    = "Allows Autographs runtime instance principals to read approved OCI Vault secret bundles."
  statements = [
    "Allow dynamic-group id ${module.iam.runtime_dynamic_group_id} to read secret-bundles in compartment id ${module.iam.compartment_ocid} where target.secret.id = '${oci_vault_secret.runtime_vault_proof.id}'"
  ]

  freeform_tags = local.tags
}
