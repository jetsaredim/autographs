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
  bootstrap_script     = file("${path.root}/../../deploy/scripts/bootstrap-runtime.sh")
  deploy_user          = var.runtime_deploy_user
  deploy_path          = var.runtime_deploy_path
  assign_public_ip     = var.assign_public_ip
  tags                 = local.tags
}
