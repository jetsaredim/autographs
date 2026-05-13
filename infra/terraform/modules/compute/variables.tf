variable "create_instance" {
  type = bool
}

variable "name_prefix" {
  type = string
}

variable "compartment_id" {
  type = string
}

variable "availability_domain" {
  type = string
}

variable "subnet_id" {
  type = string
}

variable "nsg_ids" {
  type = list(string)
}

variable "shape" {
  type = string
}

variable "ocpus" {
  type = number
}

variable "memory_in_gbs" {
  type = number
}

variable "boot_volume_size_gbs" {
  type = number
}

variable "image_ocid" {
  type = string
}

variable "ssh_public_keys" {
  type = list(string)
}

variable "bootstrap_script" {
  type = string
}

variable "deploy_user" {
  type = string
}

variable "deploy_path" {
  type = string
}

variable "assign_public_ip" {
  type = bool
}

variable "tags" {
  type = map(string)
}
