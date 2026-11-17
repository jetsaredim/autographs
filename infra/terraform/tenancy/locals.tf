locals {
  name_prefix                = "${var.project_name}-${var.environment}"
  media_bucket_name          = "${var.project_name}-media-${var.environment}"
  deploy_group_name          = "${var.project_name}-deployers"
  operator_group_name        = "${var.project_name}-operators"
  runtime_dynamic_group_name = "${var.project_name}-runtime-instances"
  deploy_user_name           = "${var.project_name}-github-deploy"

  tags = {
    project     = var.project_name
    environment = var.environment
    managed_by  = "terraform"
    owner       = var.owner_email != "" ? var.owner_email : "unset"
  }

  runtime_controller_secret_names = {
    admin_password_hash       = "${local.name_prefix}-admin-password-hash"
    oracle_db_password        = "${local.name_prefix}-oracle-db-password"
    oracle_db_wallet_password = "${local.name_prefix}-oracle-db-wallet-password"
  }
}
