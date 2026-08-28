use std::{fs, path::PathBuf};

#[test]
fn caddy_static_routes_serve_admin_and_current_static_release() {
    let caddyfile = read_repo("deploy/ansible/roles/autographs_deploy/files/Caddyfile");
    let caddy_quadlet =
        read_repo("deploy/ansible/roles/autographs_deploy/templates/autographs-caddy.container.j2");
    let deploy_tasks = read_repo("deploy/ansible/roles/autographs_deploy/tasks/main.yml");
    let deploy_defaults = read_repo("deploy/ansible/roles/autographs_deploy/defaults/main.yml");

    assert!(caddyfile.contains("@operator path /api/operator /api/operator/*"));
    assert!(caddyfile.contains("respond @operator 404"));
    assert!(caddyfile.contains("handle /admin/api/*"));
    assert!(caddyfile.contains("reverse_proxy autographs-controller:8080"));
    assert!(caddyfile.contains("handle_path /admin/*"));
    assert!(caddyfile.contains("root * /srv/autographs/static/current/admin"));
    assert!(caddyfile.contains("http://:8081"));
    assert!(caddyfile.contains("root * /srv/autographs/static/current"));
    assert!(caddyfile.contains("file_server"));
    assert!(caddyfile.matches("Cache-Control \"no-store\"").count() >= 3);
    assert!(caddyfile.contains("@staticMedia path /media/*"));
    assert!(caddyfile.contains("Cache-Control \"public, max-age=86400\""));
    assert!(caddyfile.contains("@staticAssets path /assets/*"));
    assert!(caddyfile.contains("Cache-Control \"public, max-age=3600\""));
    assert!(caddyfile.contains("@staticDocuments path / /index.html /404.html"));
    assert!(caddyfile.contains("Cache-Control \"public, max-age=60, must-revalidate\""));
    assert!(!caddyfile.contains("reverse_proxy autographs-app:3000"));

    assert!(caddy_quadlet.contains("Volume=autographs-static.volume:/srv/autographs/static:ro"));
    assert!(caddy_quadlet.contains("Image=docker.io/library/caddy:2-alpine"));
    assert!(!caddy_quadlet.contains("/usr/share/caddy/admin"));
    assert!(!caddy_quadlet.contains("autographs-app.service"));
    assert!(
        caddy_quadlet
            .contains("PublishPort=127.0.0.1:{{ autographs_deploy_candidate_preview_port }}:8081")
    );
    assert!(
        !caddy_quadlet.contains("PublishPort={{ autographs_deploy_candidate_preview_port }}:8081")
    );

    assert!(deploy_tasks.contains("Require promoted static release before Caddy cutover"));
    assert!(deploy_tasks.contains("current/manifest.json"));
    assert!(deploy_tasks.contains("Check promoted admin shell artifacts"));
    assert!(deploy_tasks.contains("Read promoted static release manifest"));
    assert!(deploy_tasks.contains("admin/index.html"));
    assert!(deploy_tasks.contains("admin/admin.js"));
    assert!(deploy_tasks.contains("admin/admin.css"));
    assert!(deploy_tasks.contains("| from_json"));
    assert!(deploy_tasks.contains("| difference("));
    assert!(deploy_tasks.contains("| intersect("));
    assert!(
        deploy_tasks.contains("legacy migration releases are accepted")
            || deploy_tasks.contains("legacy promoted release")
    );
    assert!(!deploy_tasks.contains("Remove staged admin shell before restaging"));
    assert!(!deploy_tasks.contains("Copy admin shell into promoted static release"));
    assert!(deploy_tasks.contains("Stop and disable retired Next.js app service"));
    assert!(deploy_tasks.contains("Remove retired Next.js app quadlet"));
    assert!(deploy_tasks.contains("Remove retired Next.js app container"));
    assert!(!deploy_tasks.contains("src: autographs-app.container.j2"));
    assert!(!deploy_tasks.contains("autographs_app_image"));
    assert!(
        deploy_tasks.contains(
            "http://127.0.0.1:{{ autographs_deploy_candidate_preview_port }}/manifest.json"
        )
    );
    assert!(deploy_tasks.contains("Verify Caddy admin shell route"));
    assert!(deploy_tasks.contains("ansible.builtin.command:"));
    assert!(deploy_defaults.contains("autographs_deploy_https_port: 443"));
    assert!(deploy_defaults.contains("  - curl"));
    assert!(deploy_tasks.contains(
        "\"{{ autographs_deploy_domain }}:{{ autographs_deploy_https_port }}:127.0.0.1\""
    ));
    assert!(deploy_tasks.contains(
        "\"https://{{ autographs_deploy_domain }}:{{ autographs_deploy_https_port }}/admin/\""
    ));
    assert!(deploy_tasks.contains("until: autographs_deploy_admin_shell_check.rc == 0"));
    assert!(!deploy_tasks.contains("url: \"https://127.0.0.1/admin/\""));
    assert!(!deploy_tasks.contains("Host: \"{{ autographs_deploy_domain }}\""));
}

#[test]
fn cdn_cache_contract_matches_caddy_origin_headers() {
    let caddyfile = read_repo("deploy/ansible/roles/autographs_deploy/files/Caddyfile");
    let contract = read_repo("docs/cdn-cache-contract.md");
    let deployment_runbook = read_repo("docs/deployment-runbook.md");

    assert!(contract.contains("Bypass admin and API"));
    assert!(contract.contains("Respect rollback-sensitive public documents"));
    assert!(contract.contains("Cache fingerprinted media and assets"));
    assert!(contract.contains("/admin*"));
    assert!(contract.contains("/admin/api/*"));
    assert!(contract.contains("/media/*"));
    assert!(contract.contains("/assets/*"));
    assert!(contract.contains("/collection/*"));
    assert!(contract.contains("/items/*"));
    assert!(contract.contains("/architecture/*"));
    assert!(contract.contains("/data/*"));
    assert!(contract.contains("/manifest.json"));
    assert!(contract.contains("rollback"));
    assert!(contract.contains("purge"));
    assert!(contract.contains("fingerprinted media URLs"));

    assert!(caddyfile.matches("Cache-Control \"no-store\"").count() >= 3);
    assert!(caddyfile.contains("handle /admin/api/*"));
    assert!(caddyfile.contains("@staticMedia path /media/*"));
    assert!(caddyfile.contains("Cache-Control \"public, max-age=86400\""));
    let static_asset_matchers = caddyfile
        .lines()
        .filter(|line| line.contains("@staticAssets path"))
        .collect::<Vec<_>>();
    assert_eq!(static_asset_matchers.len(), 2);
    assert!(
        static_asset_matchers
            .iter()
            .all(|line| line.contains("/assets/* /favicon.ico /icon.png"))
    );
    assert!(
        static_asset_matchers
            .iter()
            .all(|line| !line.contains("/architecture"))
    );
    assert!(caddyfile.contains("@staticDocuments path / /index.html /404.html"));
    assert!(caddyfile.contains("/collection/* /items/* /architecture/* /data/* /manifest.json"));
    assert!(caddyfile.contains("Cache-Control \"public, max-age=60, must-revalidate\""));

    assert!(
        deployment_runbook
            .contains("Public assets such as `/assets/*`, `/favicon.ico`, and `/icon.png`:")
    );
    let stale_architecture_asset_phrase = ["and the architecture", "SVG"].join(" ");
    assert!(!deployment_runbook.contains(&stale_architecture_asset_phrase));
    assert!(deployment_runbook.contains("`/architecture/*`, `/data/*`, `/manifest.json`"));
}

#[test]
fn controller_dockerfile_copies_compile_time_static_assets() {
    let dockerfile = read_repo("controller/Dockerfile");
    let dockerignore = read_repo(".dockerignore");
    let gitignore = read_repo(".gitignore");
    let smoke_dockerfile = read_repo("controller/Dockerfile.smoke");
    let static_smoke_dockerfile = read_repo("controller/Dockerfile.static-smoke");

    assert!(dockerfile.contains("COPY controller/src ./src"));
    assert!(dockerfile.contains("COPY controller/db ./db"));
    assert!(dockerfile.contains("COPY controller/static-public/assets ./static-public/assets"));
    assert!(
        dockerfile.contains("COPY controller/static-public/templates ./static-public/templates")
    );
    assert!(dockerfile
        .contains("COPY controller/static-public/data/not-found-quotes.json ./static-public/data/not-found-quotes.json"));
    assert!(!dockerfile.contains("COPY controller/static-public ./static-public"));
    assert!(dockerfile.contains("COPY controller/static-admin ./static-admin"));
    assert!(dockerfile.contains("cargo build --release --features production-persistence"));
    assert!(dockerfile.contains("COPY controller/static-admin /opt/autographs/static-admin"));
    assert!(dockerfile.contains("ca-certificates-2025.2.80_v9.0.305-102.el10_1"));
    assert!(!dockerfile.contains("oracle-instantclient"));
    assert!(dockerignore.contains("controller/static-public/data/collection.json"));
    assert!(dockerignore.contains("controller/static-public/data/facets.json"));
    assert!(dockerignore.contains("controller/static-public/data/items"));
    assert!(dockerignore.contains("controller/static-public/manifest.json"));
    assert!(dockerignore.contains("controller/static-public/items"));
    assert!(dockerignore.contains("controller/static-public/media"));
    assert!(gitignore.contains("controller/static-public/data/collection.json"));
    assert!(gitignore.contains("controller/static-public/data/facets.json"));
    assert!(gitignore.contains("controller/static-public/data/items/"));
    assert!(gitignore.contains("controller/static-public/manifest.json"));
    assert!(gitignore.contains("controller/static-public/items/*"));
    assert!(gitignore.contains("controller/static-public/media/*"));
    assert!(smoke_dockerfile.contains("COPY controller/static-admin ./static-admin"));
    assert!(static_smoke_dockerfile.contains("COPY controller/static-admin ./static-admin"));
    assert!(smoke_dockerfile.contains("microdnf install -y ca-certificates"));
    assert!(static_smoke_dockerfile.contains("microdnf install -y ca-certificates curl"));
    assert!(!smoke_dockerfile.contains("oracle-instantclient"));
    assert!(!static_smoke_dockerfile.contains("oracle-instantclient"));
}

#[test]
fn controller_quadlet_keeps_private_api_off_host_ports() {
    let controller_quadlet = read_repo(
        "deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2",
    );

    assert!(controller_quadlet.contains("Network=autographs.network"));
    assert!(
        controller_quadlet.contains("Volume=autographs-static.volume:/var/lib/autographs/static")
    );
    assert!(!controller_quadlet.contains("PublishPort="));
}

#[test]
fn controller_runtime_excludes_deploy_oci_credentials() {
    let controller_quadlet = read_repo(
        "deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2",
    );
    let app_env = read_repo("deploy/ansible/roles/autographs_deploy/templates/app.env.j2");
    let deploy_tasks = read_repo("deploy/ansible/roles/autographs_deploy/tasks/main.yml");
    let static_runtime_runbook = read_repo("docs/static-runtime-runbook.md");

    for runtime_contract in [&controller_quadlet, &app_env, &static_runtime_runbook] {
        assert!(!runtime_contract.contains("OCI_PRIVATE_KEY_PATH"));
        assert!(!runtime_contract.contains("/opt/autographs/secrets"));
    }
    assert!(!app_env.contains("OCI_CLI_USER_OCID"));
    assert!(!app_env.contains("OCI_TENANCY_OCID"));
    assert!(!app_env.contains("OCI_FINGERPRINT"));
    assert!(!static_runtime_runbook.contains("CANDIDATE_SECRETS_DIR"));
    assert!(deploy_tasks.contains("Remove legacy OCI private key from controller runtime"));
    assert!(!deploy_tasks.contains("- name: Install OCI private key"));
}

#[test]
fn deploy_wires_oracle_heartbeat_interval_override() {
    let deploy_workflow = read_repo(".github/workflows/deploy.yml");
    let app_env = read_repo("deploy/ansible/roles/autographs_deploy/templates/app.env.j2");
    let deploy_tasks = read_repo("deploy/ansible/roles/autographs_deploy/tasks/main.yml");

    assert!(deploy_workflow.contains(
        "AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS: ${{ vars.AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS || '86400' }}"
    ));
    assert!(deploy_workflow.contains(
        "--extra-vars autographs_oracle_heartbeat_interval_seconds=${{ env.AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS }}"
    ));
    assert!(app_env.contains(
        "AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS={{ autographs_oracle_heartbeat_interval_seconds | default(86400) }}"
    ));
    assert!(
        deploy_workflow
            .contains("ORACLE_DB_WALLET_PASSWORD: ${{ secrets.ORACLE_DB_WALLET_PASSWORD }}")
    );
    assert!(app_env.contains(
        "ORACLE_DB_WALLET_PASSWORD={{ '' if oracle_db_wallet_password_vault_secret_id | length > 0 else lookup('env', 'ORACLE_DB_WALLET_PASSWORD') | default('', true) }}"
    ));
    assert!(app_env.contains(
        "ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID={{ oracle_db_wallet_password_vault_secret_id }}"
    ));
    assert!(deploy_tasks.contains(
        "lookup('env', 'ORACLE_DB_WALLET_PASSWORD') | default('', true) | trim | length > 0"
    ));
}

#[test]
fn deploy_discovers_vault_secret_ids_without_github_variables() {
    let runtime_secrets = read_repo("infra/terraform/runtime_secrets.tf");
    let runtime_outputs = read_repo("infra/terraform/outputs.tf");
    let tenancy_iam = read_repo("infra/terraform/modules/iam/main.tf");
    let deploy_workflow = read_repo(".github/workflows/deploy.yml");

    assert!(runtime_secrets.contains("data \"oci_vault_secrets\" \"runtime_controller\""));
    assert!(runtime_secrets.contains("state          = \"ACTIVE\""));
    assert!(runtime_secrets.contains("env_name => one(secret_lookup.secrets[*].id)"));
    assert!(runtime_outputs.contains("output \"runtime_secret_id_env_vars\""));
    assert!(tenancy_iam.contains("to inspect secrets in compartment id"));

    for (env_name, output_name) in [
        (
            "AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID",
            "admin_password_hash_vault_secret_id",
        ),
        (
            "ORACLE_DB_PASSWORD_VAULT_SECRET_ID",
            "oracle_db_password_vault_secret_id",
        ),
        (
            "ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID",
            "oracle_db_wallet_password_vault_secret_id",
        ),
    ] {
        assert!(deploy_workflow.contains(&format!("'.{env_name}'")));
        assert!(deploy_workflow.contains(&format!("steps.terraform_apply.outputs.{output_name}")));
        assert!(!deploy_workflow.contains(&format!("vars.{env_name}")));
    }
}

#[test]
fn deploy_tasks_require_hash_only_admin_credentials_and_fail_closed_without_vault_id() {
    let deploy_tasks = read_repo("deploy/ansible/roles/autographs_deploy/tasks/main.yml");
    let credential_tasks =
        read_repo("deploy/ansible/roles/autographs_deploy/tasks/admin_credentials.yml");
    let validation_playbook =
        read_repo("deploy/ansible/playbooks/deploy-admin-credentials-validate-test.yml");

    assert!(deploy_tasks.contains("ansible.builtin.include_tasks: admin_credentials.yml"));

    for expected in [
        "autographs_deploy_admin_password_resolved: \"\"",
        "''\n        if autographs_deploy_admin_password_hash_vault_secret_id_input | length > 0",
        "else autographs_deploy_admin_password_hash_existing",
        "AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID for deployed admin routes",
        "Production deployment is hash-only",
    ] {
        assert!(
            credential_tasks.contains(expected),
            "deploy tasks should enforce the hash-only authentication contract with {expected}"
        );
    }

    assert!(!credential_tasks.contains("lookup('password'"));
    assert!(!credential_tasks.contains("autographs_deploy_admin_password_input"));
    assert!(!credential_tasks.contains("autographs_deploy_admin_password_existing"));
    assert!(validation_playbook.contains("previous Vault deployment"));
    assert!(validation_playbook.contains("autographs_admin_missing_secret_failed_closed"));
}

fn read_repo(relative: &str) -> String {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("controller parent")
        .to_path_buf();
    fs::read_to_string(repo.join(relative)).expect("read repository artifact")
}
