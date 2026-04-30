resource "oci_core_instance" "runtime" {
  count               = var.create_instance ? 1 : 0
  availability_domain = var.availability_domain
  compartment_id      = var.compartment_id
  display_name        = "${var.name_prefix}-vm"
  shape               = var.shape

  shape_config {
    ocpus         = var.ocpus
    memory_in_gbs = var.memory_in_gbs
  }

  create_vnic_details {
    subnet_id        = var.subnet_id
    assign_public_ip = var.assign_public_ip
    nsg_ids          = var.nsg_ids
    hostname_label   = "autographs"
  }

  source_details {
    source_type             = "image"
    source_id               = var.image_ocid
    boot_volume_size_in_gbs = var.boot_volume_size_gbs
  }

  metadata = {
    ssh_authorized_keys = join("\n", var.ssh_public_keys)
  }

  freeform_tags = var.tags

  lifecycle {
    precondition {
      condition     = !var.create_instance || var.availability_domain != ""
      error_message = "availability_domain must be set when create_instance is true."
    }

    precondition {
      condition     = !var.create_instance || var.subnet_id != null
      error_message = "subnet_id must be available when create_instance is true."
    }

    precondition {
      condition     = !var.create_instance || var.image_ocid != ""
      error_message = "Provide runtime_image_ocid or oracle_linux_image_ocids[var.region] before creating the runtime instance."
    }

    precondition {
      condition     = !var.create_instance || length(var.ssh_public_keys) > 0
      error_message = "At least one SSH public key is required when create_instance is true."
    }
  }
}
