locals {
  runtime_controller_secret_definitions = {
    admin_password_hash = {
      description   = "Autographs runtime admin password hash placeholder."
      secret_id_env = "AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID"
      secret_name   = "${var.name_prefix}-admin-password-hash"
    }
    oracle_db_password = {
      description   = "Autographs runtime OCI-generated Oracle database password."
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

data "oci_vault_secrets" "oracle_db_password" {
  count          = var.create_autonomous_database ? 1 : 0
  compartment_id = var.compartment_ocid
  vault_id       = oci_kms_vault.runtime_secrets.id
  name           = local.runtime_controller_secret_definitions.oracle_db_password.secret_name
}

data "oci_vault_secrets" "runtime_readiness" {
  compartment_id = var.compartment_ocid
  vault_id       = oci_kms_vault.runtime_secrets.id
}

locals {
  autonomous_database_admin_password_secret_id = var.create_autonomous_database ? one(data.oci_vault_secrets.oracle_db_password[0].secrets).id : null
  runtime_secret_metadata_by_name = {
    for secret in data.oci_vault_secrets.runtime_readiness.secrets :
    secret.secret_name => secret
    if contains([
      for definition in values(local.runtime_controller_secret_definitions) :
      definition.secret_name
    ], secret.secret_name)
  }
}

resource "oci_vault_secret" "runtime" {
  for_each = local.runtime_controller_secret_definitions

  compartment_id         = var.compartment_ocid
  description            = each.value.description
  enable_auto_generation = each.key == "oracle_db_password"
  key_id                 = oci_kms_key.runtime_secrets.id
  secret_name            = each.value.secret_name
  vault_id               = oci_kms_vault.runtime_secrets.id

  dynamic "secret_content" {
    for_each = each.key == "oracle_db_password" ? [] : [each.key]
    content {
      content_type = "BASE64"
      content      = base64encode("AUTOGRAPHS_UNCONFIGURED_${upper(each.key)}")
      name         = "terraform-bootstrap"
      stage        = "CURRENT"
    }
  }

  dynamic "secret_generation_context" {
    for_each = each.key == "oracle_db_password" ? [each.key] : []
    content {
      generation_type     = "PASSPHRASE"
      generation_template = "DBAAS_DEFAULT_PASSWORD"
      passphrase_length   = 30
    }
  }

  dynamic "rotation_config" {
    for_each = each.key == "oracle_db_password" && var.create_autonomous_database ? [each.key] : []
    content {
      is_scheduled_rotation_enabled = true
      rotation_interval             = "P90D"

      target_system_details {
        target_system_type = "ADB"
        adb_id             = module.data_services.autonomous_database_id
      }
    }
  }

  lifecycle {
    prevent_destroy = true
    ignore_changes = [
      secret_content,
    ]
  }

  freeform_tags = local.tags
}

locals {
  runtime_secret_id_env_vars = {
    for name, definition in local.runtime_controller_secret_definitions :
    definition.secret_id_env => try(oci_vault_secret.runtime[name].id, null)
  }
  runtime_secret_values_ready = alltrue([
    for name, definition in local.runtime_controller_secret_definitions :
    name == "oracle_db_password" ?
    try(tonumber(local.runtime_secret_metadata_by_name[definition.secret_name].current_version_number), 0) >= 1 :
    try(tonumber(local.runtime_secret_metadata_by_name[definition.secret_name].current_version_number), 0) > 1
  ])
}
