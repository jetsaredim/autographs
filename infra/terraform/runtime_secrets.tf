locals {
  runtime_controller_secret_names = {
    AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID = "${var.name_prefix}-admin-password-hash"
    ORACLE_DB_PASSWORD_VAULT_SECRET_ID             = "${var.name_prefix}-oracle-db-password"
    ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID      = "${var.name_prefix}-oracle-db-wallet-password"
  }
}

data "oci_vault_secrets" "runtime_controller" {
  for_each = local.runtime_controller_secret_names

  compartment_id = var.compartment_ocid
  name           = each.value
  state          = "ACTIVE"

  lifecycle {
    postcondition {
      condition     = length(self.secrets) == 1
      error_message = "Expected exactly one ACTIVE OCI Vault secret named ${each.value} in the runtime compartment."
    }
  }
}

locals {
  runtime_secret_id_env_vars = {
    for env_name, secret_lookup in data.oci_vault_secrets.runtime_controller :
    env_name => one(secret_lookup.secrets[*].id)
  }
}
