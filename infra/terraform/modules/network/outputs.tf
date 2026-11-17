output "vcn_id" {
  value = oci_core_vcn.main.id
}

output "public_subnet_id" {
  value = oci_core_subnet.public.id
}

output "runtime_nsg_id" {
  value = oci_core_network_security_group.runtime.id
}
