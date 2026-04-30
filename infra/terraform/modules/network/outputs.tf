output "vcn_id" {
  value = var.create_network ? oci_core_vcn.main[0].id : null
}

output "public_subnet_id" {
  value = var.create_network ? oci_core_subnet.public[0].id : null
}

output "runtime_nsg_id" {
  value = var.create_network ? oci_core_network_security_group.runtime[0].id : null
}
