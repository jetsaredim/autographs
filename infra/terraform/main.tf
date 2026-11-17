module "network" {
  source = "./modules/network"

  name_prefix              = local.name_prefix
  project_name             = var.project_name
  compartment_id           = local.compartment_ocid
  vcn_cidr_block           = var.vcn_cidr_block
  public_subnet_cidr_block = var.public_subnet_cidr_block
  ssh_ingress_cidrs        = var.ssh_ingress_cidrs
  http_ingress_cidrs       = var.http_ingress_cidrs
  https_ingress_cidrs      = var.https_ingress_cidrs
  tags                     = local.tags
}

module "compute" {
  source = "./modules/compute"

  name_prefix          = local.name_prefix
  project_name         = var.project_name
  compartment_id       = local.compartment_ocid
  availability_domain  = var.availability_domain
  subnet_id            = module.network.public_subnet_id
  nsg_ids              = module.network.runtime_nsg_id != null ? [module.network.runtime_nsg_id] : []
  shape                = var.runtime_shape
  ocpus                = var.runtime_ocpus
  memory_in_gbs        = var.runtime_memory_gbs
  boot_volume_size_gbs = var.runtime_boot_volume_size_gbs
  image_ocid           = local.runtime_image_ocid
  ssh_public_keys      = var.runtime_ssh_public_keys
  tags                 = local.tags
}

module "data_services" {
  source = "./modules/data_services"

  create_autonomous_database                   = var.create_autonomous_database
  runtime_secret_values_ready                  = local.runtime_secret_values_ready
  autonomous_database_name                     = local.autonomous_database_name
  autonomous_database_display_name             = local.autonomous_database_display_name
  autonomous_database_admin_password_secret_id = local.autonomous_database_admin_password_secret_id
  media_bucket_namespace                       = var.media_bucket_namespace
  media_bucket_name                            = local.media_bucket_name
  compartment_id                               = local.compartment_ocid
  tags                                         = local.tags
}
