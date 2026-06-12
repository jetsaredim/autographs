variable "name_prefix" {
  type = string
}

variable "parent_compartment_ocid" {
  type = string
}

variable "tenancy_ocid" {
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

variable "media_bucket_name" {
  type = string
}

variable "state_bucket_name" {
  type = string
}

variable "tags" {
  type = map(string)
}
