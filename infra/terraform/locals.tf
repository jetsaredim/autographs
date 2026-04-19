locals {
  tags = {
    project     = var.project_name
    environment = var.environment
    managed_by  = "terraform"
    owner       = var.owner_email != "" ? var.owner_email : "unset"
  }

  runtime_image_ocid = var.runtime_image_ocid != "" ? var.runtime_image_ocid : lookup(var.oracle_linux_image_ocids, var.region, "")
}
