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

variable "home_region" {
  description = "OCI home region for IAM and tenancy-scoped resources."
  type        = string
  default     = "us-ashburn-1"
}

variable "parent_compartment_ocid" {
  description = "Compartment that owns the project compartment and policies. Usually the tenancy OCID."
  type        = string
}

variable "existing_compartment_ocid" {
  description = "Existing project compartment OCID to use when create_compartment is false."
  type        = string
  default     = ""
}

variable "create_compartment" {
  description = "Whether Terraform should create the project compartment."
  type        = bool
  default     = true
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
  description = "Existing OCI group name used by GitHub deploy automation."
  type        = string
  default     = "autographs-deployers"
}

variable "operator_group_name" {
  description = "Existing OCI group name used by the human operator."
  type        = string
  default     = "autographs-operators"
}

variable "create_state_bucket" {
  description = "Whether Terraform should create the remote state bucket."
  type        = bool
  default     = true
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

variable "state_object_key" {
  description = "Object key name for the environment state file inside the backend bucket."
  type        = string
  default     = "envs/prod/terraform.tfstate"
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

variable "assign_public_ip" {
  description = "Whether the runtime VNIC should receive a public IP."
  type        = bool
  default     = true
}
