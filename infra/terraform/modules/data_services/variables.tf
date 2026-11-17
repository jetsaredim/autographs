variable "compartment_id" {
  description = "Compartment OCID that owns data services."
  type        = string
}

variable "create_autonomous_database" {
  description = "Whether to create the Oracle Autonomous Database metadata store."
  type        = bool
}

variable "runtime_secret_values_ready" {
  description = "Whether the runtime Vault secrets have been populated with real out-of-band values."
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

variable "autonomous_database_admin_password_secret_id" {
  description = "OCI Vault secret OCID containing the Autonomous Database ADMIN password."
  type        = string
}

variable "media_bucket_namespace" {
  description = "Object Storage namespace for the media bucket."
  type        = string
}

variable "media_bucket_name" {
  description = "Private Object Storage media bucket name."
  type        = string
}

variable "tags" {
  description = "Freeform tags applied to data services."
  type        = map(string)
}
