data "oci_identity_compartments" "project" {
  compartment_id = var.tenancy_ocid
  name           = "${var.name_prefix}-cmp"
  state          = "ACTIVE"
}

locals {
  compartment_ocid = one(data.oci_identity_compartments.project.compartments[*].id)
}
