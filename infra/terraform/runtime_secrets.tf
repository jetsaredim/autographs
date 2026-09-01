locals {
  runtime_controller_secret_definitions = {
    admin_password_hash = {
      description   = "Autographs runtime admin password hash placeholder."
      secret_id_env = "AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID"
      secret_name   = "${var.name_prefix}-admin-password-hash"
    }
    oracle_db_password = {
      description   = "Autographs runtime Oracle database password placeholder."
      secret_id_env = "ORACLE_DB_PASSWORD_VAULT_SECRET_ID"
      secret_name   = "${var.name_prefix}-oracle-db-password"
    }
    oracle_db_wallet_password = {
      description   = "Autographs runtime Oracle wallet password placeholder."
      secret_id_env = "ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID"
      secret_name   = "${var.name_prefix}-oracle-db-wallet-password"
    }
  }
}

resource "oci_kms_vault" "runtime_secrets" {
  compartment_id = var.compartment_ocid
  display_name   = "${var.name_prefix}-runtime-secrets-vault"
  vault_type     = "DEFAULT"

  lifecycle {
    prevent_destroy = true
  }

  freeform_tags = local.tags
}

resource "oci_kms_key" "runtime_secrets" {
  compartment_id      = var.compartment_ocid
  display_name        = "${var.name_prefix}-runtime-secrets-key"
  management_endpoint = oci_kms_vault.runtime_secrets.management_endpoint
  protection_mode     = "SOFTWARE"

  key_shape {
    algorithm = "AES"
    length    = 32
  }

  lifecycle {
    prevent_destroy = true
  }

  freeform_tags = local.tags
}

resource "oci_vault_secret" "runtime" {
  for_each = local.runtime_controller_secret_definitions

  compartment_id         = var.compartment_ocid
  description            = each.value.description
  enable_auto_generation = false
  key_id                 = oci_kms_key.runtime_secrets.id
  secret_name            = each.value.secret_name
  vault_id               = oci_kms_vault.runtime_secrets.id

  secret_content {
    content_type = "BASE64"
    content      = base64encode("AUTOGRAPHS_UNCONFIGURED_${upper(each.key)}")
    name         = "terraform-bootstrap"
    stage        = "CURRENT"
  }

  lifecycle {
    prevent_destroy = true
    ignore_changes = [
      secret_content,
      secret_generation_context,
    ]
  }

  freeform_tags = local.tags
}

locals {
  runtime_secret_id_env_vars = {
    for name, definition in local.runtime_controller_secret_definitions :
    definition.secret_id_env => try(oci_vault_secret.runtime[name].id, null)
  }
}
