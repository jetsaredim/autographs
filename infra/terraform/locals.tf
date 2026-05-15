locals {
  tags = {
    project     = var.project_name
    environment = var.environment
    managed_by  = "terraform"
    owner       = var.owner_email != "" ? var.owner_email : "unset"
  }

  runtime_image_ocid = var.runtime_image_ocid != "" ? var.runtime_image_ocid : lookup(var.oracle_linux_image_ocids, var.region, "")
  autonomous_database_runtime_whitelisted_ips = (
    var.autonomous_database_allow_runtime_public_ip && module.compute.public_ip != null
    ? [module.compute.public_ip]
    : []
  )
  autonomous_database_whitelisted_ips = distinct(concat(var.autonomous_database_whitelisted_ips, local.autonomous_database_runtime_whitelisted_ips))
}
