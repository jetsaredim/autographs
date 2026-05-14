output "autonomous_database_id" {
  description = "Oracle Autonomous Database OCID, when created."
  value       = var.create_autonomous_database ? oci_database_autonomous_database.catalog[0].id : null
}

output "autonomous_database_name" {
  description = "Oracle Autonomous Database DB name."
  value       = var.autonomous_database_name
}

output "media_bucket_name" {
  description = "Private Object Storage media bucket name."
  value       = var.media_bucket_name
}

output "media_bucket_namespace" {
  description = "Object Storage namespace for the media bucket."
  value       = var.media_bucket_namespace
}
