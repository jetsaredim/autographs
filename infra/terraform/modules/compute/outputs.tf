output "instance_id" {
  value = oci_core_instance.runtime.id
}

output "public_ip" {
  value = oci_core_instance.runtime.public_ip
}

output "private_ip" {
  value = oci_core_instance.runtime.private_ip
}
