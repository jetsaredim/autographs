resource "oci_database_autonomous_database" "catalog" {
  count = var.create_autonomous_database ? 1 : 0

  compartment_id              = var.compartment_id
  db_name                     = var.autonomous_database_name
  display_name                = var.autonomous_database_display_name
  admin_password              = var.autonomous_database_admin_password
  db_workload                 = var.autonomous_database_db_workload
  is_free_tier                = var.autonomous_database_is_free_tier
  is_mtls_connection_required = var.autonomous_database_is_mtls_connection_required
  license_model               = var.autonomous_database_license_model
  data_storage_size_in_tbs    = var.autonomous_database_data_storage_size_in_tbs
  freeform_tags               = var.tags

  lifecycle {
    precondition {
      condition     = !var.create_autonomous_database || var.autonomous_database_admin_password != ""
      error_message = "autonomous_database_admin_password is required when create_autonomous_database is true."
    }
  }
}

resource "oci_objectstorage_bucket" "media" {
  count = var.create_media_bucket ? 1 : 0

  compartment_id = var.compartment_id
  namespace      = var.media_bucket_namespace
  name           = var.media_bucket_name
  access_type    = "NoPublicAccess"
  storage_tier   = "Standard"
  versioning     = var.media_bucket_versioning
  freeform_tags  = var.tags

  lifecycle {
    precondition {
      condition     = !var.create_media_bucket || var.media_bucket_namespace != ""
      error_message = "media_bucket_namespace is required when create_media_bucket is true."
    }
  }
}

resource "oci_kms_vault" "admin" {
  compartment_id = var.compartment_id
  display_name   = var.admin_vault_name
  vault_type     = var.admin_vault_type
  freeform_tags  = var.tags
}

resource "oci_kms_key" "admin" {
  compartment_id      = var.compartment_id
  display_name        = var.admin_vault_key_name
  management_endpoint = oci_kms_vault.admin.management_endpoint
  protection_mode     = "SOFTWARE"
  freeform_tags       = var.tags

  key_shape {
    algorithm = "AES"
    length    = 32
  }
}

resource "oci_vault_secret" "admin_access_key" {
  compartment_id = var.compartment_id
  key_id         = oci_kms_key.admin.id
  secret_name    = var.admin_access_key_secret_name
  vault_id       = oci_kms_vault.admin.id
  description    = "Admin access key. Real value is operator-managed in OCI Vault."
  freeform_tags  = var.tags

  secret_content {
    content_type = "BASE64"
    content      = base64encode("replace-with-operator-managed-admin-access-key")
    name         = "terraform-placeholder"
    stage        = "CURRENT"
  }

  lifecycle {
    ignore_changes = [secret_content]
  }
}

resource "oci_vault_secret" "admin_secret_key" {
  compartment_id = var.compartment_id
  key_id         = oci_kms_key.admin.id
  secret_name    = var.admin_secret_key_secret_name
  vault_id       = oci_kms_vault.admin.id
  description    = "Admin secret key. Real value is operator-managed in OCI Vault."
  freeform_tags  = var.tags

  secret_content {
    content_type = "BASE64"
    content      = base64encode("replace-with-operator-managed-admin-secret-key")
    name         = "terraform-placeholder"
    stage        = "CURRENT"
  }

  lifecycle {
    ignore_changes = [secret_content]
  }
}
