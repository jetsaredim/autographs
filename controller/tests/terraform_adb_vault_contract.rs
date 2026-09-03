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
    assert!(
        runtime_secrets.contains("enable_auto_generation = each.key == \"oracle_db_password\"")
    );
    assert!(runtime_secrets.contains("data \"oci_vault_secrets\" \"oracle_db_password\""));
    assert!(runtime_secrets.contains("data \"oci_vault_secrets\" \"runtime_readiness\""));
    assert!(runtime_secrets.contains("data \"oci_vault_secret\" \"runtime_readiness\""));
    assert!(runtime_secrets.contains("count          = var.create_autonomous_database ? 1 : 0"));
    assert!(runtime_secrets.contains(
        "for_each = var.create_autonomous_database ? local.runtime_controller_secret_definitions : {}"
    ));
    assert!(
        runtime_secrets.contains(
            "secret_id = local.runtime_secret_metadata_by_name[each.value.secret_name].id"
        )
    );
    assert!(runtime_secrets.contains("vault_id       = oci_kms_vault.runtime_secrets.id"));
    assert!(runtime_secrets.contains(
        "name           = local.runtime_controller_secret_definitions.oracle_db_password.secret_name"
    ));
    assert!(
        runtime_secrets.contains("one(data.oci_vault_secrets.oracle_db_password[0].secrets).id")
    );
    assert!(runtime_secrets.contains("dynamic \"secret_content\""));
    assert!(
        runtime_secrets.contains("for_each = each.key == \"oracle_db_password\" ? [] : [each.key]")
    );
    assert!(runtime_secrets.contains("dynamic \"secret_generation_context\""));
    assert!(
        runtime_secrets.contains("for_each = each.key == \"oracle_db_password\" ? [each.key] : []")
    );
    assert!(runtime_secrets.contains("generation_type     = \"PASSPHRASE\""));
    assert!(runtime_secrets.contains("generation_template = \"DBAAS_DEFAULT_PASSWORD\""));
    assert!(runtime_secrets.contains("passphrase_length   = 30"));
    assert!(runtime_secrets.contains("dynamic \"rotation_config\""));
    assert!(
        runtime_secrets
            .contains("each.key == \"oracle_db_password\" && var.create_autonomous_database")
    );
    assert!(runtime_secrets.contains("is_scheduled_rotation_enabled = true"));
    assert!(runtime_secrets.contains("rotation_interval             = \"P90D\""));
    assert!(runtime_secrets.contains("target_system_type = \"ADB\""));
    assert!(
        runtime_secrets
            .contains("adb_id             = module.data_services.autonomous_database_id")
    );
    assert!(runtime_secrets.contains("ignore_changes = [\n      secret_content,"));
    assert!(!tenancy_main.contains("resource \"oci_kms_vault\""));
    assert!(!tenancy_main.contains("resource \"oci_kms_key\""));
    assert!(!tenancy_main.contains("resource \"oci_vault_secret\""));
    assert!(root_main.contains(
        "autonomous_database_admin_password_secret_id    = local.autonomous_database_admin_password_secret_id"
    ));
    assert!(!root_main.contains("oci_vault_secret.runtime[\"oracle_db_password\"].id"));
    assert!(module_variables.contains("variable \"autonomous_database_admin_password_secret_id\""));
    assert!(module_variables.contains("variable \"runtime_secret_values_ready\""));
    assert!(module_main.contains(
        "secret_id                   = var.autonomous_database_admin_password_secret_id"
    ));
    assert!(module_main.contains(
        "condition     = !var.create_autonomous_database || var.runtime_secret_values_ready"
    ));
    assert!(runtime_outputs.contains("if local.runtime_secret_values_ready"));

    for source in [&runtime_secrets, &root_main, &root_variables, &module_main] {
        assert!(!source.contains("oci_vault_secret_bundle"));
        assert!(!source.contains("secret_version_number"));
        assert!(!source.contains("terraform_remote_state"));
    }
    assert!(!root_main.contains("modules/iam"));
    assert!(!root_main.contains("tenancy/"));
    assert!(!root_variables.contains("variable \"autonomous_database_admin_password\""));
    assert!(!root_variables.contains("variable \"runtime_secrets_ready\""));
    assert!(!root_variables.contains("variable \"runtime_secret_values_ready\""));
    assert!(!module_main.contains("admin_password              ="));
}

#[test]
fn fresh_runtime_bootstrap_fails_closed_until_secret_values_are_ready() {
    let runtime_secrets = read_repo("infra/terraform/runtime_secrets.tf");
    let variables = read_repo("infra/terraform/variables.tf");
    let example = read_repo("infra/terraform/environments/prod/terraform.tfvars.example");
    let ci = read_repo(".github/workflows/ci.yml");
    let deploy = read_repo(".github/workflows/deploy.yml");
    let bootstrap = read_repo("docs/oci-bootstrap.md");
    let runtime_secret_words = normalize_whitespace(&runtime_secrets);
    let bootstrap_words = normalize_whitespace(&bootstrap);

    assert!(runtime_secrets.contains("runtime_secret_values_ready = alltrue(["));
    assert!(
        runtime_secrets
            .contains("for name, definition in local.runtime_controller_secret_definitions")
    );
    assert!(runtime_secrets.contains("name == \"oracle_db_password\""));
    assert!(runtime_secret_words.contains(
        "name == \"oracle_db_password\" ? ( try(tonumber(data.oci_vault_secret.runtime_readiness[name].current_version_number), 0) > 1 || ( try(tonumber(data.oci_vault_secret.runtime_readiness[name].current_version_number), 0) >= 1 && try(data.oci_vault_secret.runtime_readiness[name].is_auto_generation_enabled, false) ) )"
    ));
    assert!(runtime_secret_words.contains(
        ") : try(tonumber(data.oci_vault_secret.runtime_readiness[name].current_version_number), 0) > 1"
    ));
    assert!(!runtime_secrets.contains(
        "local.runtime_secret_metadata_by_name[definition.secret_name].current_version_number"
    ));
    for source in [variables, example, ci, deploy.clone()] {
        assert!(!source.contains("runtime_secrets_ready"));
        assert!(!source.contains("OCI_RUNTIME_SECRETS_READY"));
    }
    assert!(deploy.contains("Oracle database password is OCI-generated"));
    assert!(deploy.contains("Populate only the Oracle wallet password and admin password hash"));
    assert!(!deploy.contains("Populate all three CURRENT values"));
    assert!(bootstrap_words.contains("Readiness derives from Vault version metadata"));
    assert!(bootstrap_words.contains("stops before Ansible"));
    assert!(
        bootstrap_words.contains("OCI generates a usable Oracle database password as version 1")
    );
    assert!(bootstrap_words.contains("Replace only those two manual secrets"));
    assert!(!bootstrap.contains("Replace all three `CURRENT` secret versions"));
}

#[test]
fn generated_database_password_rollout_is_content_free_and_fail_closed() {
    let contract = read_repo("docs/configuration-contract.md");
    let runbook = read_repo("docs/deployment-runbook.md");
    let contract_words = normalize_whitespace(&contract);
    let runbook_words = normalize_whitespace(&runbook);

    for required in [
        "`oracle_db_password` is generated and rotation-managed by OCI",
        "generated version 1 is usable",
        "must never be populated or overwritten out of band",
        "`oracle_db_wallet_password` and `admin_password_hash`",
        "`CURRENT` versions 2 or later",
        "scheduled `P90D` rotation",
    ] {
        assert!(
            contract_words.contains(required),
            "configuration contract must contain {required:?}"
        );
    }
    assert!(!contract_words.contains("Automatic OCI rotation remains disabled"));
    assert!(!contract_words.contains("Populate all three"));

    for required in [
        "stop if it proposes any replacement or destroy",
        "Terraform `apply` updates that configuration but does not",
        "invoke `RotateSecret`",
        "Never issue an on-demand rotation while rotation is pending or failed",
        "oci vault secret rotate",
        "--wait-for-state SUCCEEDED",
        "--max-wait-seconds 1200",
        "test \"$after_version\" -gt \"$before_version\"",
        "ActiveEnterTimestamp",
        "refreshed Oracle database credential",
        "Do not restart or bounce the controller",
    ] {
        assert!(
            runbook_words.contains(required),
            "deployment runbook must contain {required:?}"
        );
    }
    assert!(!runbook.contains("\noci secrets secret-bundle get"));
    assert!(!runbook.contains("populate the three `CURRENT` secret versions"));
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

fn normalize_whitespace(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}
