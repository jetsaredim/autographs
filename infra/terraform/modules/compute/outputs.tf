output "instance_id" {
  value = var.create_instance ? oci_core_instance.runtime[0].id : null
}

output "public_ip" {
  value = var.create_instance ? oci_core_instance.runtime[0].public_ip : null
}

output "private_ip" {
  value = var.create_instance ? oci_core_instance.runtime[0].private_ip : null
}
