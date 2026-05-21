data "oci_core_images" "oracle_linux_10" {
  count = var.create_runtime_instance && var.runtime_image_ocid == "" ? 1 : 0

  compartment_id           = var.compartment_ocid
  operating_system         = "Oracle Linux"
  operating_system_version = "10"
  shape                    = var.runtime_shape
  state                    = "AVAILABLE"
  sort_by                  = "TIMECREATED"
  sort_order               = "DESC"
}
