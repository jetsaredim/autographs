variable "name_prefix" {
  type = string
}

variable "parent_compartment_ocid" {
  type = string
}

variable "tenancy_ocid" {
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

variable "create_deploy_group" {
  type = bool
}

variable "operator_group_name" {
  type = string
}

variable "create_operator_group" {
  type = bool
}

variable "create_deploy_user" {
  type = bool
}

variable "runtime_dynamic_group_name" {
  type = string
}

variable "deploy_user_name" {
  type = string
}

variable "deploy_user_description" {
  type = string
}

variable "deploy_user_email" {
  type = string
}

variable "deploy_user_api_public_key" {
  type = string
}

variable "tags" {
  type = map(string)
}
