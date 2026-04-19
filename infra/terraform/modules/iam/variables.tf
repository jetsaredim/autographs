variable "name_prefix" {
  type = string
}

variable "parent_compartment_ocid" {
  type = string
}

variable "create_compartment" {
  type = bool
}

variable "existing_compartment_ocid" {
  type = string
}

variable "compartment_description" {
  type = string
}

variable "deploy_group_name" {
  type = string
}

variable "operator_group_name" {
  type = string
}

variable "tags" {
  type = map(string)
}
