use std::fs;
use std::path::PathBuf;

#[test]
fn adb_password_uses_existing_runtime_vault_secret_ocid() {
    let runtime_secrets = read_repo("infra/terraform/runtime_secrets.tf");
    let root_main = read_repo("infra/terraform/main.tf");
    let root_variables = read_repo("infra/terraform/variables.tf");
    let module_main = read_repo("infra/terraform/modules/data_services/main.tf");
    let module_variables = read_repo("infra/terraform/modules/data_services/variables.tf");

    assert!(runtime_secrets.contains(
        "ORACLE_DB_PASSWORD_VAULT_SECRET_ID             = \"${var.name_prefix}-oracle-db-password\""
    ));
    assert!(runtime_secrets.contains("data \"oci_vault_secrets\" \"runtime_controller\""));
    assert!(runtime_secrets.contains("condition     = length(self.secrets) == 1"));
    assert!(runtime_secrets.contains("env_name => one(secret_lookup.secrets[*].id)"));
    assert!(root_main.contains(
        "autonomous_database_admin_password_secret_id    = local.runtime_secret_id_env_vars[\"ORACLE_DB_PASSWORD_VAULT_SECRET_ID\"]"
    ));
    assert!(module_variables.contains("variable \"autonomous_database_admin_password_secret_id\""));
    assert!(module_main.contains(
        "secret_id                   = var.autonomous_database_admin_password_secret_id"
    ));

    for source in [&runtime_secrets, &root_main, &root_variables, &module_main] {
        assert!(!source.contains("oci_vault_secret_bundle"));
        assert!(!source.contains("secret_version_number"));
        assert!(!source.contains("terraform_remote_state"));
    }
    assert!(!root_main.contains("modules/iam"));
    assert!(!root_main.contains("tenancy/"));
    assert!(!root_variables.contains("variable \"autonomous_database_admin_password\""));
    assert!(!module_main.contains("admin_password              ="));
}

#[test]
fn workflows_and_current_examples_do_not_accept_plaintext_adb_password() {
    for relative in [
        ".github/workflows/ci.yml",
        ".github/workflows/deploy.yml",
        ".github/.env.github.example",
        ".env.example",
        "infra/terraform/environments/prod/terraform.tfvars.example",
        "docs/configuration-contract.md",
        "docs/deployment-runbook.md",
    ] {
        let source = read_repo(relative);
        assert!(
            !source.contains("ADB_ADMIN_PASSWORD"),
            "{relative} must not reference the retired GitHub plaintext secret"
        );
        assert!(
            !source.contains("TF_VAR_autonomous_database_admin_password"),
            "{relative} must not accept the retired Terraform plaintext input"
        );
        assert!(
            !source.contains("vars.ORACLE_DB_PASSWORD_VAULT_SECRET_ID"),
            "{relative} must not duplicate the Terraform-owned secret OCID in GitHub Variables"
        );
    }
}

#[test]
fn deploy_identity_can_read_only_the_adb_password_secret_bundle() {
    let tenancy_main = read_repo("infra/terraform/tenancy/main.tf");
    let tenancy_outputs = read_repo("infra/terraform/tenancy/outputs.tf");

    assert!(tenancy_main.contains(
        "resource \"oci_identity_policy\" \"deploy_database_password_secret_bundle_access\""
    ));
    assert!(tenancy_main.contains(
        "Allow group id ${module.iam.deploy_group_id} to read secret-bundles in compartment id ${module.iam.compartment_ocid} where target.secret.id = '${oci_vault_secret.runtime[\"oracle_db_password\"].id}'"
    ));
    assert!(!tenancy_main.contains(
        "module.iam.deploy_group_id} to read secret-bundles in compartment id ${module.iam.compartment_ocid} where target.secret.id = '${oci_vault_secret.runtime[\"oracle_db_wallet_password\"].id}'"
    ));
    assert!(!tenancy_main.contains(
        "module.iam.deploy_group_id} to read secret-bundles in compartment id ${module.iam.compartment_ocid} where target.secret.id = '${oci_vault_secret.runtime[\"admin_password_hash\"].id}'"
    ));
    assert!(
        tenancy_outputs
            .contains("output \"deploy_database_password_secret_bundle_access_policy_name\"")
    );
}

fn read_repo(relative: &str) -> String {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("controller parent")
        .to_path_buf();
    fs::read_to_string(repo.join(relative)).expect("read repository artifact")
}
