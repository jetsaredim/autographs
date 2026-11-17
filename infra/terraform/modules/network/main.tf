moved {
  from = oci_core_vcn.main[0]
  to   = oci_core_vcn.main
}

moved {
  from = oci_core_internet_gateway.main[0]
  to   = oci_core_internet_gateway.main
}

moved {
  from = oci_core_route_table.public[0]
  to   = oci_core_route_table.public
}

moved {
  from = oci_core_network_security_group.runtime[0]
  to   = oci_core_network_security_group.runtime
}

moved {
  from = oci_core_network_security_group_security_rule.egress_all[0]
  to   = oci_core_network_security_group_security_rule.egress_all
}

moved {
  from = oci_core_subnet.public[0]
  to   = oci_core_subnet.public
}

resource "oci_core_vcn" "main" {
  compartment_id = var.compartment_id
  cidr_block     = var.vcn_cidr_block
  display_name   = "${var.name_prefix}-vcn"
  dns_label      = var.project_name

  freeform_tags = var.tags
}

resource "oci_core_internet_gateway" "main" {
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "${var.name_prefix}-igw"
  enabled        = true

  freeform_tags = var.tags
}

resource "oci_core_route_table" "public" {
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "${var.name_prefix}-public-rt"

  route_rules {
    network_entity_id = oci_core_internet_gateway.main.id
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
  }

  freeform_tags = var.tags
}

resource "oci_core_network_security_group" "runtime" {
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.main.id
  display_name   = "${var.name_prefix}-runtime-nsg"

  freeform_tags = var.tags
}

resource "oci_core_network_security_group_security_rule" "ssh_ingress" {
  for_each                  = toset(var.ssh_ingress_cidrs)
  network_security_group_id = oci_core_network_security_group.runtime.id
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
  for_each                  = toset(var.http_ingress_cidrs)
  network_security_group_id = oci_core_network_security_group.runtime.id
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
  for_each                  = toset(var.https_ingress_cidrs)
  network_security_group_id = oci_core_network_security_group.runtime.id
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
  network_security_group_id = oci_core_network_security_group.runtime.id
  direction                 = "EGRESS"
  protocol                  = "all"
  destination               = "0.0.0.0/0"
  destination_type          = "CIDR_BLOCK"
}

resource "oci_core_subnet" "public" {
  compartment_id             = var.compartment_id
  vcn_id                     = oci_core_vcn.main.id
  cidr_block                 = var.public_subnet_cidr_block
  display_name               = "${var.name_prefix}-public-subnet"
  dns_label                  = "public"
  route_table_id             = oci_core_route_table.public.id
  prohibit_public_ip_on_vnic = false

  freeform_tags = var.tags
}
