variable "compartment_id" {
  description = "Compartment OCID that owns data services."
  type        = string
}

variable "create_autonomous_database" {
  description = "Whether to create the Oracle Autonomous Database metadata store."
  type        = bool
}

variable "autonomous_database_name" {
  description = "Oracle Autonomous Database DB name."
  type        = string
}

variable "autonomous_database_display_name" {
  description = "Oracle Autonomous Database display name."
  type        = string
}

variable "autonomous_database_admin_password" {
  description = "Initial ADMIN password for the Autonomous Database."
  type        = string
  sensitive   = true
}

variable "autonomous_database_is_free_tier" {
  description = "Whether to request Oracle Always Free sizing."
  type        = bool
}

variable "autonomous_database_db_workload" {
  description = "Autonomous Database workload type."
  type        = string
}

variable "autonomous_database_license_model" {
  description = "Autonomous Database license model."
  type        = string
}

variable "autonomous_database_data_storage_size_in_tbs" {
  description = "Autonomous Database storage size in TB."
  type        = number
}

variable "create_media_bucket" {
  description = "Whether to create the private Object Storage media bucket."
  type        = bool
}

variable "media_bucket_namespace" {
  description = "Object Storage namespace for the media bucket."
  type        = string
}

variable "media_bucket_name" {
  description = "Private Object Storage media bucket name."
  type        = string
}

variable "media_bucket_versioning" {
  description = "Object Storage versioning mode."
  type        = string
}

variable "tags" {
  description = "Freeform tags applied to data services."
  type        = map(string)
}
