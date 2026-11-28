variable "project_name" {
  description = "Project slug used in OCI display names."
  type        = string
  default     = "autographs"

  validation {
    condition     = can(regex("^[a-z][a-z0-9]{0,11}$", var.project_name))
    error_message = "project_name must be lowercase alphanumeric, start with a letter, and be at most 12 characters so the derived Oracle DB name stays within 14 characters."
  }
}

variable "environment" {
  description = "Environment name for this root."
  type        = string
  default     = "prod"
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

variable "bootstrap_runtime_secrets" {
  description = "Use only for the targeted Vault bootstrap to omit the ADB rotation configuration."
  type        = bool
  default     = false
}

variable "media_bucket_namespace" {
  description = "OCI Object Storage namespace for the private autograph media bucket."
  type        = string
  default     = ""
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

variable "autographs_dns_ttl" {
  description = "TTL for the Porkbun backup A record and the Cloudflare A record in DNS-only mode."
  type        = number
  default     = 600

  validation {
    condition     = var.autographs_dns_ttl >= 600 && var.autographs_dns_ttl <= 86400
    error_message = "autographs_dns_ttl must be between 600 and 86400 seconds to work with both DNS providers."
  }
}

variable "autographs_dns_record_id" {
  description = "Existing Porkbun DNS record ID for autographs.jetsaredim.net."
  type        = string
}

variable "cloudflare_mode" {
  description = "Cloudflare migration stage: off, DNS-only, or proxied."
  type        = string
  default     = "proxied"

  validation {
    condition     = contains(["off", "dns_only", "proxied"], var.cloudflare_mode)
    error_message = "cloudflare_mode must be off, dns_only, or proxied."
  }
}

variable "cloudflare_account_id" {
  description = "Cloudflare account ID for the jetsaredim.net zone."
  type        = string
  default     = ""

  validation {
    condition     = var.cloudflare_mode == "off" || can(regex("^[0-9a-f]{32}$", var.cloudflare_account_id))
    error_message = "Set cloudflare_account_id to the 32-character account ID when Cloudflare mode is dns_only or proxied."
  }
}
