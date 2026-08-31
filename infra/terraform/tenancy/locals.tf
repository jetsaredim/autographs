locals {
  tags = {
    project     = var.project_name
    environment = var.environment
    managed_by  = "terraform"
    owner       = var.owner_email != "" ? var.owner_email : "unset"
  }

  runtime_controller_secret_names = {
    admin_password_hash       = "${var.name_prefix}-admin-password-hash"
    oracle_db_password        = "${var.name_prefix}-oracle-db-password"
    oracle_db_wallet_password = "${var.name_prefix}-oracle-db-wallet-password"
  }
}
