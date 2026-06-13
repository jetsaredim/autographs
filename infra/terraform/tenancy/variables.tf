variable "project_name" {
  description = "Project slug used in OCI display names."
  type        = string
  default     = "autographs"
}

variable "environment" {
  description = "Environment name for this root."
  type        = string
  default     = "prod"
}

variable "name_prefix" {
  description = "Prefix applied to OCI display names."
  type        = string
  default     = "autographs-prod"
}

variable "auth" {
  description = "OCI provider auth mode. Use SecurityToken for local OCI CLI session auth, or APIKey for deploy API signing keys."
  type        = string
  default     = "APIKey"

  validation {
    condition     = contains(["APIKey", "SecurityToken"], var.auth)
    error_message = "auth must be APIKey or SecurityToken."
  }
}

variable "config_file_profile" {
  description = "OCI CLI config profile used when auth is SecurityToken. Leave empty for APIKey auth."
  type        = string
  default     = ""
}

variable "tenancy_ocid" {
  description = "OCI tenancy OCID used by the Terraform provider."
  type        = string
}

variable "user_ocid" {
  description = "OCI user OCID for the Terraform/API-signing identity."
  type        = string
}

variable "fingerprint" {
  description = "Fingerprint for the OCI API signing key."
  type        = string
}

variable "private_key_path" {
  description = "Path to the OCI API signing private key on the executor machine."
  type        = string
}

variable "region" {
  description = "OCI region where the state bucket lives."
  type        = string
  default     = "us-ashburn-1"
}

variable "home_region" {
  description = "OCI home region for IAM and tenancy-scoped resources."
  type        = string
  default     = "us-ashburn-1"
}

variable "parent_compartment_ocid" {
  description = "Compartment that owns the project compartment and policies. Usually the tenancy OCID."
  type        = string
}

variable "compartment_description" {
  description = "Description for the project compartment."
  type        = string
  default     = "Personal autograph collection application resources"
}

variable "owner_email" {
  description = "Optional owner tag value for OCI tagging."
  type        = string
  default     = ""
}

variable "deploy_group_name" {
  description = "OCI group name used by GitHub deploy automation."
  type        = string
  default     = "autographs-deployers"
}

variable "operator_group_name" {
  description = "OCI group name used by the human operator."
  type        = string
  default     = "autographs-operators"
}

variable "runtime_dynamic_group_name" {
  description = "OCI dynamic group name used by Autographs runtime VM instance principals."
  type        = string
  default     = "autographs-runtime-instances"
}

variable "deploy_user_name" {
  description = "Name of the OCI user used by GitHub deployment automation."
  type        = string
  default     = "autographs-github-deploy"
}

variable "deploy_user_description" {
  description = "Description for the OCI deployment user."
  type        = string
  default     = "GitHub Actions deployment user for Autographs."
}

variable "deploy_user_email" {
  description = "Optional email address for the OCI deployment user."
  type        = string
  default     = ""
}

variable "deploy_user_api_public_key" {
  description = "Optional PEM public API key to attach to the deployment user. Generate and store the private key outside Terraform."
  type        = string
  default     = ""
}

variable "media_bucket_name" {
  description = "Private Object Storage bucket name that runtime instance principals can access."
  type        = string
  default     = "autographs-media-prod"
}

variable "state_bucket_name" {
  description = "Object Storage bucket name for Terraform state."
  type        = string
  default     = "autographs-tf-state"
}

variable "state_bucket_storage_tier" {
  description = "Storage tier for the Terraform state bucket."
  type        = string
  default     = "Standard"

  validation {
    condition     = contains(["Standard", "InfrequentAccess", "Archive"], var.state_bucket_storage_tier)
    error_message = "state_bucket_storage_tier must be Standard, InfrequentAccess, or Archive."
  }
}
