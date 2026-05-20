module "network" {
  source = "./modules/network"

  create_network           = var.create_network
  name_prefix              = var.name_prefix
  compartment_id           = var.compartment_ocid
  vcn_cidr_block           = var.vcn_cidr_block
  public_subnet_cidr_block = var.public_subnet_cidr_block
  ssh_ingress_cidrs        = var.ssh_ingress_cidrs
  http_ingress_cidrs       = var.http_ingress_cidrs
  https_ingress_cidrs      = var.https_ingress_cidrs
  tags                     = local.tags
}

module "compute" {
  source = "./modules/compute"

  create_instance      = var.create_runtime_instance
  name_prefix          = var.name_prefix
  compartment_id       = var.compartment_ocid
  availability_domain  = var.availability_domain
  subnet_id            = module.network.public_subnet_id
  nsg_ids              = module.network.runtime_nsg_id != null ? [module.network.runtime_nsg_id] : []
  shape                = var.runtime_shape
  ocpus                = var.runtime_ocpus
  memory_in_gbs        = var.runtime_memory_gbs
  boot_volume_size_gbs = var.runtime_boot_volume_size_gbs
  image_ocid           = local.runtime_image_ocid
  ssh_public_keys      = var.runtime_ssh_public_keys
  assign_public_ip     = var.assign_public_ip
  tags                 = local.tags
}

module "data_services" {
  source = "./modules/data_services"

  create_autonomous_database                      = var.create_autonomous_database
  autonomous_database_name                        = var.autonomous_database_name
  autonomous_database_display_name                = var.autonomous_database_display_name
  autonomous_database_admin_password              = var.autonomous_database_admin_password
  autonomous_database_is_free_tier                = var.autonomous_database_is_free_tier
  autonomous_database_is_mtls_connection_required = var.autonomous_database_is_mtls_connection_required
  autonomous_database_db_workload                 = var.autonomous_database_db_workload
  autonomous_database_license_model               = var.autonomous_database_license_model
  autonomous_database_data_storage_size_in_tbs    = var.autonomous_database_data_storage_size_in_tbs
  create_media_bucket                             = var.create_media_bucket
  media_bucket_namespace                          = var.media_bucket_namespace
  media_bucket_name                               = var.media_bucket_name
  media_bucket_versioning                         = var.media_bucket_versioning
  compartment_id                                  = var.compartment_ocid
  tags                                            = local.tags
}
