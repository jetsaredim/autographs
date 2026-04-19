variable "create_network" {
  type = bool
}

variable "name_prefix" {
  type = string
}

variable "compartment_id" {
  type = string
}

variable "vcn_cidr_block" {
  type = string
}

variable "public_subnet_cidr_block" {
  type = string
}

variable "ssh_ingress_cidrs" {
  type = list(string)
}

variable "http_ingress_cidrs" {
  type = list(string)
}

variable "https_ingress_cidrs" {
  type = list(string)
}

variable "tags" {
  type = map(string)
}
