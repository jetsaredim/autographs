output "bucket_name" {
  value = var.create_bucket ? oci_objectstorage_bucket.this[0].name : var.bucket_name
}
