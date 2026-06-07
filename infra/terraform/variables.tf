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
  description = "OCI provider auth mode. Use SecurityToken for local OCI CLI session auth, or APIKey for GitHub/deploy API signing keys."
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
  description = "OCI region where runtime resources live."
  type        = string
  default     = "us-ashburn-1"
}

variable "owner_email" {
  description = "Optional owner tag value for OCI tagging."
  type        = string
  default     = ""
}

variable "compartment_ocid" {
  description = "Project compartment OCID produced by the tenancy bootstrap root."
  type        = string
}

variable "create_network" {
  description = "Whether Terraform should create the VCN and subnet baseline."
  type        = bool
  default     = true
}

variable "vcn_cidr_block" {
  description = "CIDR block for the project VCN."
  type        = string
  default     = "10.42.0.0/16"
}

variable "public_subnet_cidr_block" {
  description = "CIDR block for the public runtime subnet."
  type        = string
  default     = "10.42.10.0/24"
}

variable "ssh_ingress_cidrs" {
  description = "CIDRs allowed to SSH to the runtime VM."
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "http_ingress_cidrs" {
  description = "CIDRs allowed to reach the Caddy reverse proxy over HTTP."
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "https_ingress_cidrs" {
  description = "CIDRs allowed to reach the Caddy reverse proxy over HTTPS."
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "create_runtime_instance" {
  description = "Whether Terraform should create the OCI VM used for the Caddy to Next.js runtime."
  type        = bool
  default     = true
}

variable "availability_domain" {
  description = "Availability domain name for the runtime VM."
  type        = string
  default     = ""
}

variable "runtime_shape" {
  description = "OCI shape for the runtime VM."
  type        = string
  default     = "VM.Standard.E2.1.Micro"
}

variable "runtime_ocpus" {
  description = "OCPU count for the flex runtime shape."
  type        = number
  default     = 1
}

variable "runtime_memory_gbs" {
  description = "Memory in GB for the flex runtime shape."
  type        = number
  default     = 1
}

variable "runtime_boot_volume_size_gbs" {
  description = "Boot volume size for the runtime VM."
  type        = number
  default     = 50
}

variable "runtime_image_ocid" {
  description = "Custom image OCID for the runtime VM. Leave empty to use the latest Oracle Linux 10 image compatible with runtime_shape."
  type        = string
  default     = ""
}

variable "runtime_ssh_public_keys" {
  description = "SSH public keys injected into the runtime instance."
  type        = list(string)
  default     = []
}

variable "runtime_deploy_user" {
  description = "OS user that receives deployment files and runs container deployment commands."
  type        = string
  default     = "opc"

  validation {
    condition     = can(regex("^[A-Za-z_][A-Za-z0-9_-]*$", var.runtime_deploy_user))
    error_message = "runtime_deploy_user must be a Linux username containing only letters, numbers, underscores, and hyphens, and must not start with a number."
  }
}

variable "runtime_deploy_path" {
  description = "Absolute path on the runtime VM used by Ansible for quadlet, Caddy, wallet, and secret deployment files."
  type        = string
  default     = "/opt/autographs"

  validation {
    condition     = can(regex("^/opt/autographs(/[A-Za-z0-9_-][A-Za-z0-9._-]*)*$", var.runtime_deploy_path))
    error_message = "runtime_deploy_path must be /opt/autographs or a safe child path under /opt/autographs without dot-only path segments."
  }
}

variable "assign_public_ip" {
  description = "Whether the runtime VNIC should receive a public IP."
  type        = bool
  default     = true
}

variable "create_autonomous_database" {
  description = "Whether Terraform should create the Oracle Autonomous Database Free metadata store."
  type        = bool
  default     = false
}

variable "autonomous_database_name" {
  description = "Oracle Autonomous Database DB name. Keep this short and alphanumeric for Oracle service constraints."
  type        = string
  default     = "autographsdb"

  validation {
    condition     = can(regex("^[A-Za-z][A-Za-z0-9]{0,13}$", var.autonomous_database_name))
    error_message = "autonomous_database_name must start with a letter, contain only letters and numbers, and be at most 14 characters."
  }
}

variable "autonomous_database_display_name" {
  description = "Display name for the Oracle Autonomous Database metadata store."
  type        = string
  default     = "autographs-prod-adb"
}

variable "autonomous_database_admin_password" {
  description = "Initial ADMIN password for the Oracle Autonomous Database. Required only when create_autonomous_database is true."
  type        = string
  sensitive   = true
  default     = ""
}

variable "autonomous_database_is_free_tier" {
  description = "Whether the Autonomous Database should use Oracle Always Free sizing."
  type        = bool
  default     = true
}

variable "autonomous_database_is_mtls_connection_required" {
  description = "Whether the Autonomous Database should require mTLS wallet authentication."
  type        = bool
  default     = true
}

variable "autonomous_database_db_workload" {
  description = "Autonomous Database workload type."
  type        = string
  default     = "OLTP"
}

variable "autonomous_database_license_model" {
  description = "Autonomous Database license model."
  type        = string
  default     = "LICENSE_INCLUDED"
}

variable "autonomous_database_data_storage_size_in_tbs" {
  description = "Autonomous Database storage size in TB."
  type        = number
  default     = 1
}

variable "create_media_bucket" {
  description = "Whether Terraform should create the private Object Storage bucket for autograph images."
  type        = bool
  default     = false
}

variable "media_bucket_namespace" {
  description = "OCI Object Storage namespace for the private autograph media bucket."
  type        = string
  default     = ""
}

variable "media_bucket_name" {
  description = "Name of the private Object Storage bucket for autograph images."
  type        = string
  default     = "autographs-media-prod"
}

variable "media_bucket_versioning" {
  description = "Object Storage versioning mode for the private media bucket."
  type        = string
  default     = "Enabled"

  validation {
    condition     = contains(["Enabled", "Disabled"], var.media_bucket_versioning)
    error_message = "media_bucket_versioning must be Enabled or Disabled."
  }
}

variable "controller_vault_name" {
  description = "Display name for the OCI Vault used by the private controller."
  type        = string
  default     = "autographs-controller-vault"
}

variable "controller_vault_type" {
  description = "OCI Vault type for controller runtime secrets."
  type        = string
  default     = "DEFAULT"

  validation {
    condition     = contains(["DEFAULT", "VIRTUAL_PRIVATE"], var.controller_vault_type)
    error_message = "controller_vault_type must be DEFAULT or VIRTUAL_PRIVATE."
  }
}

variable "controller_vault_key_name" {
  description = "Display name for the OCI Vault key used to encrypt controller runtime secrets."
  type        = string
  default     = "autographs-controller-secrets-key"
}

variable "controller_s3_access_key_secret_name" {
  description = "OCI Vault secret name for the controller OCI S3 access key."
  type        = string
  default     = "autographs-controller-s3-access-key"
}

variable "controller_s3_secret_key_secret_name" {
  description = "OCI Vault secret name for the controller OCI S3 secret key."
  type        = string
  default     = "autographs-controller-s3-secret-key"
}

variable "porkbun_api_key" {
  description = "Porkbun API key used to manage DNS records."
  type        = string
  sensitive   = true
  default     = ""
}

variable "porkbun_secret_key" {
  description = "Porkbun secret API key used to manage DNS records."
  type        = string
  sensitive   = true
  default     = ""
}

variable "autographs_dns_domain" {
  description = "Porkbun domain that owns the autographs DNS record."
  type        = string
  default     = "jetsaredim.net"
}

variable "autographs_dns_subdomain" {
  description = "Subdomain for the autographs app DNS record."
  type        = string
  default     = "autographs"
}

variable "autographs_dns_ttl" {
  description = "TTL for the autographs DNS record."
  type        = number
  default     = 300
}

variable "autographs_dns_record_id" {
  description = "Existing Porkbun DNS record ID for autographs.jetsaredim.net."
  type        = string
}
