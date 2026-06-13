variable "compartment_id" {
  type = string
}

variable "namespace" {
  type = string
}

variable "bucket_name" {
  type = string
}

variable "storage_tier" {
  type = string
}

variable "tags" {
  type = map(string)
}
