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
  description = "CIDRs allowed to reach nginx over HTTP."
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "https_ingress_cidrs" {
  description = "CIDRs allowed to reach nginx over HTTPS."
  type        = list(string)
  default     = ["0.0.0.0/0"]
}

variable "create_runtime_instance" {
  description = "Whether Terraform should create the OCI VM used for the nginx to Next.js runtime."
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
  description = "Custom image OCID for the runtime VM. Leave empty to use the per-region map."
  type        = string
  default     = ""
}

variable "oracle_linux_image_ocids" {
  description = "Per-region Oracle Linux image OCIDs used when runtime_image_ocid is empty."
  type        = map(string)
  default     = {}
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
  description = "Absolute path on the runtime VM used for compose and nginx deployment files."
  type        = string
  default     = "/opt/autographs"

  validation {
    condition     = can(regex("^/[A-Za-z0-9._/-]+$", var.runtime_deploy_path))
    error_message = "runtime_deploy_path must be an absolute path containing only letters, numbers, dots, underscores, hyphens, and slashes."
  }
}

variable "assign_public_ip" {
  description = "Whether the runtime VNIC should receive a public IP."
  type        = bool
  default     = true
}
