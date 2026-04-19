resource "oci_core_vcn" "main" {
  count          = var.create_network ? 1 : 0
  compartment_id = var.compartment_id
  cidr_block     = var.vcn_cidr_block
  display_name   = "${var.name_prefix}-vcn"
  dns_label      = "autographs"

  freeform_tags = var.tags
}

resource "oci_core_internet_gateway" "main" {
  count          = var.create_network ? 1 : 0
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.main[0].id
  display_name   = "${var.name_prefix}-igw"
  enabled        = true

  freeform_tags = var.tags
}

resource "oci_core_route_table" "public" {
  count          = var.create_network ? 1 : 0
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.main[0].id
  display_name   = "${var.name_prefix}-public-rt"

  route_rules {
    network_entity_id = oci_core_internet_gateway.main[0].id
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
  }

  freeform_tags = var.tags
}

resource "oci_core_network_security_group" "runtime" {
  count          = var.create_network ? 1 : 0
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.main[0].id
  display_name   = "${var.name_prefix}-runtime-nsg"

  freeform_tags = var.tags
}

resource "oci_core_network_security_group_security_rule" "ssh_ingress" {
  for_each                  = var.create_network ? toset(var.ssh_ingress_cidrs) : toset([])
  network_security_group_id = oci_core_network_security_group.runtime[0].id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = each.value
  source_type               = "CIDR_BLOCK"

  tcp_options {
    destination_port_range {
      min = 22
      max = 22
    }
  }
}

resource "oci_core_network_security_group_security_rule" "http_ingress" {
  for_each                  = var.create_network ? toset(var.http_ingress_cidrs) : toset([])
  network_security_group_id = oci_core_network_security_group.runtime[0].id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = each.value
  source_type               = "CIDR_BLOCK"

  tcp_options {
    destination_port_range {
      min = 80
      max = 80
    }
  }
}

resource "oci_core_network_security_group_security_rule" "https_ingress" {
  for_each                  = var.create_network ? toset(var.https_ingress_cidrs) : toset([])
  network_security_group_id = oci_core_network_security_group.runtime[0].id
  direction                 = "INGRESS"
  protocol                  = "6"
  source                    = each.value
  source_type               = "CIDR_BLOCK"

  tcp_options {
    destination_port_range {
      min = 443
      max = 443
    }
  }
}

resource "oci_core_network_security_group_security_rule" "egress_all" {
  count                     = var.create_network ? 1 : 0
  network_security_group_id = oci_core_network_security_group.runtime[0].id
  direction                 = "EGRESS"
  protocol                  = "all"
  destination               = "0.0.0.0/0"
  destination_type          = "CIDR_BLOCK"
}

resource "oci_core_subnet" "public" {
  count                      = var.create_network ? 1 : 0
  compartment_id             = var.compartment_id
  vcn_id                     = oci_core_vcn.main[0].id
  cidr_block                 = var.public_subnet_cidr_block
  display_name               = "${var.name_prefix}-public-subnet"
  dns_label                  = "public"
  route_table_id             = oci_core_route_table.public[0].id
  prohibit_public_ip_on_vnic = false

  freeform_tags = var.tags
}
