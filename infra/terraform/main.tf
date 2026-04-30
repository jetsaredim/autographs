data "oci_objectstorage_namespace" "ns" {
  compartment_id = var.tenancy_ocid
}

module "iam" {
  source = "./modules/iam"

  providers = {
    oci.home = oci.home
  }

  name_prefix               = var.name_prefix
  parent_compartment_ocid   = var.parent_compartment_ocid
  create_compartment        = var.create_compartment
  existing_compartment_ocid = var.existing_compartment_ocid
  compartment_description   = var.compartment_description
  deploy_group_name         = var.deploy_group_name
  operator_group_name       = var.operator_group_name
  tags                      = local.tags
}

module "state_bucket" {
  source = "./modules/state_bucket"

  create_bucket  = var.create_state_bucket
  compartment_id = module.iam.compartment_ocid
  namespace      = data.oci_objectstorage_namespace.ns.namespace
  bucket_name    = var.state_bucket_name
  storage_tier   = var.state_bucket_storage_tier
  tags           = local.tags
}

module "network" {
  source = "./modules/network"

  create_network           = var.create_network
  name_prefix              = var.name_prefix
  compartment_id           = module.iam.compartment_ocid
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
  compartment_id       = module.iam.compartment_ocid
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
