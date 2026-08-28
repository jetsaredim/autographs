locals {
  tags = {
    project     = var.project_name
    environment = var.environment
    managed_by  = "terraform"
    owner       = var.owner_email != "" ? var.owner_email : "unset"
  }

  runtime_controller_secret_definitions = {
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
    admin_password_hash = {
      description   = "Autographs runtime admin password hash placeholder."
      secret_id_env = "AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID"
      secret_name   = "${var.name_prefix}-admin-password-hash"
    }
  }
}
