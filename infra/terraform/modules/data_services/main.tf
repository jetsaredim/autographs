moved {
  from = oci_database_autonomous_database.catalog[0]
  to   = oci_database_autonomous_database.catalog
}

resource "oci_database_autonomous_database" "catalog" {

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
    prevent_destroy = true
  }
}

moved {
  from = oci_objectstorage_bucket.media[0]
  to   = oci_objectstorage_bucket.media
}

resource "oci_objectstorage_bucket" "media" {

  compartment_id = var.compartment_id
  namespace      = var.media_bucket_namespace
  name           = var.media_bucket_name
  access_type    = "NoPublicAccess"
  storage_tier   = "Standard"
  versioning     = "Enabled"
  freeform_tags  = var.tags

  lifecycle {
    prevent_destroy = true

    precondition {
      condition     = var.media_bucket_namespace != ""
      error_message = "media_bucket_namespace is required for the private media bucket."
    }
  }
}
