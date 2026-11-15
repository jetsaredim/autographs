resource "oci_database_autonomous_database" "catalog" {
  count = var.create_autonomous_database ? 1 : 0

  compartment_id              = var.compartment_id
  db_name                     = var.autonomous_database_name
  display_name                = var.autonomous_database_display_name
  secret_id                   = var.autonomous_database_admin_password_secret_id
  db_workload                 = "OLTP"
  is_free_tier                = true
  is_mtls_connection_required = true
  license_model               = "LICENSE_INCLUDED"
  data_storage_size_in_tbs    = 1
  freeform_tags               = var.tags

  lifecycle {
    precondition {
      condition     = !var.create_autonomous_database || var.runtime_secret_values_ready
      error_message = "All managed runtime secrets must have a non-bootstrap CURRENT version before create_autonomous_database can create or update ADB."
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
  versioning     = "Enabled"
  freeform_tags  = var.tags

  lifecycle {
    precondition {
      condition     = !var.create_media_bucket || var.media_bucket_namespace != ""
      error_message = "media_bucket_namespace is required when create_media_bucket is true."
    }
  }
}
