resource "oci_objectstorage_bucket" "this" {
  count          = var.create_bucket ? 1 : 0
  compartment_id = var.compartment_id
  namespace      = var.namespace
  name           = var.bucket_name
  access_type    = "NoPublicAccess"
  storage_tier   = var.storage_tier
  versioning     = "Enabled"
  auto_tiering   = "Disabled"

  freeform_tags = var.tags
}
