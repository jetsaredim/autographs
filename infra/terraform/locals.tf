locals {
  name_prefix                      = "${var.project_name}-${var.environment}"
  autonomous_database_name         = "${var.project_name}db"
  autonomous_database_display_name = "${local.name_prefix}-adb"
  media_bucket_name                = "${var.project_name}-media-${var.environment}"

  tags = {
    project     = var.project_name
    environment = var.environment
    managed_by  = "terraform"
    owner       = var.owner_email != "" ? var.owner_email : "unset"
  }

  runtime_image_ocid = var.runtime_image_ocid != "" ? var.runtime_image_ocid : try(data.oci_core_images.oracle_linux_10[0].images[0].id, "")
}
