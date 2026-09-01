use std::fs;
use std::path::PathBuf;

#[test]
fn deployment_root_owns_runtime_vault_resources_and_supplies_adb_secret_ocid() {
    let runtime_secrets = read_repo("infra/terraform/runtime_secrets.tf");
    let root_main = read_repo("infra/terraform/main.tf");
    let root_variables = read_repo("infra/terraform/variables.tf");
    let module_main = read_repo("infra/terraform/modules/data_services/main.tf");
    let module_variables = read_repo("infra/terraform/modules/data_services/variables.tf");
    let tenancy_main = read_repo("infra/terraform/tenancy/main.tf");
    let runtime_outputs = read_repo("infra/terraform/outputs.tf");

    assert!(runtime_secrets.contains("secret_id_env = \"ORACLE_DB_PASSWORD_VAULT_SECRET_ID\""));
    assert!(runtime_secrets.contains("resource \"oci_kms_vault\" \"runtime_secrets\""));
    assert!(runtime_secrets.contains("resource \"oci_kms_key\" \"runtime_secrets\""));
    assert!(runtime_secrets.contains("resource \"oci_vault_secret\" \"runtime\""));
    assert!(
        runtime_secrets
            .contains("definition.secret_id_env => try(oci_vault_secret.runtime[name].id, null)")
    );
    assert!(runtime_secrets.contains("prevent_destroy = true"));
    assert!(runtime_secrets.contains("enable_auto_generation = false"));
    assert!(!runtime_secrets.contains("enable_auto_generation,"));
    assert!(!runtime_secrets.contains("secret_generation_context,"));
    assert!(runtime_secrets.contains("ignore_changes = [\n      secret_content,"));
    assert!(!runtime_secrets.contains("data \"oci_vault_secrets\""));
    assert!(!tenancy_main.contains("resource \"oci_kms_vault\""));
    assert!(!tenancy_main.contains("resource \"oci_kms_key\""));
    assert!(!tenancy_main.contains("resource \"oci_vault_secret\""));
    assert!(root_main.contains(
        "autonomous_database_admin_password_secret_id    = local.runtime_secret_id_env_vars[\"ORACLE_DB_PASSWORD_VAULT_SECRET_ID\"]"
    ));
    assert!(module_variables.contains("variable \"autonomous_database_admin_password_secret_id\""));
    assert!(module_variables.contains("variable \"runtime_secrets_ready\""));
    assert!(module_main.contains(
        "secret_id                   = var.autonomous_database_admin_password_secret_id"
    ));
    assert!(
        module_main.contains(
            "condition     = !var.create_autonomous_database || var.runtime_secrets_ready"
        )
    );
    assert!(runtime_outputs.contains("if var.runtime_secrets_ready"));

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
fn fresh_runtime_bootstrap_fails_closed_until_secret_values_are_ready() {
    let variables = read_repo("infra/terraform/variables.tf");
    let example = read_repo("infra/terraform/environments/prod/terraform.tfvars.example");
    let ci = read_repo(".github/workflows/ci.yml");
    let deploy = read_repo(".github/workflows/deploy.yml");
    let bootstrap = read_repo("docs/oci-bootstrap.md");

    assert!(variables.contains("variable \"runtime_secrets_ready\""));
    assert!(example.contains("runtime_secrets_ready                         = false"));
    for workflow in [ci, deploy.clone()] {
        assert!(workflow.contains(
            "TF_VAR_runtime_secrets_ready: ${{ vars.OCI_RUNTIME_SECRETS_READY || 'false' }}"
        ));
    }
    assert!(deploy.contains("OCI_RUNTIME_SECRETS_READY must be true before deploy"));
    assert!(bootstrap.contains("`runtime_secrets_ready` and"));
    assert!(bootstrap.contains("stops before Ansible"));
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
fn deploy_identity_can_manage_regional_vault_resources_in_project_compartment() {
    let iam_main = read_repo("infra/terraform/modules/iam/main.tf");
    let tenancy_main = read_repo("infra/terraform/tenancy/main.tf");

    for resource_type in ["vaults", "keys", "secret-family"] {
        assert!(iam_main.contains(&format!(
            "Allow ${{local.deploy_group}} to manage {resource_type} in compartment id ${{local.compartment_ocid}}"
        )));
    }
    assert!(!iam_main.contains("to manage vaults in tenancy"));
    assert!(!iam_main.contains("to manage keys in tenancy"));
    assert!(!iam_main.contains("to manage secret-family in tenancy"));
    assert!(!tenancy_main.contains("deploy_database_password_secret_bundle_access"));
}

#[test]
fn runtime_bundle_policy_uses_stable_secret_names_instead_of_cross_state_ocids() {
    let tenancy_main = read_repo("infra/terraform/tenancy/main.tf");
    let tenancy_locals = read_repo("infra/terraform/tenancy/locals.tf");

    assert!(tenancy_main.contains("where target.secret.name = '${secret_name}'"));
    assert!(!tenancy_main.contains("where target.secret.id"));
    for suffix in [
        "admin-password-hash",
        "oracle-db-password",
        "oracle-db-wallet-password",
    ] {
        assert!(tenancy_locals.contains(&format!("${{var.name_prefix}}-{suffix}")));
    }
}

fn read_repo(relative: &str) -> String {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("controller parent")
        .to_path_buf();
    fs::read_to_string(repo.join(relative)).expect("read repository artifact")
}
