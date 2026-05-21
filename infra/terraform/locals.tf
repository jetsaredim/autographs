locals {
  tags = {
    project     = var.project_name
    environment = var.environment
    managed_by  = "terraform"
    owner       = var.owner_email != "" ? var.owner_email : "unset"
  }

  runtime_image_ocid = var.runtime_image_ocid != "" ? var.runtime_image_ocid : try(data.oci_core_images.oracle_linux_10[0].images[0].id, "")
}
