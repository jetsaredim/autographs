use crate::{config::RuntimeSecretOverrides, oci_secrets::OciVaultSecretClient};

const SECRET_ENV_MAPPINGS: &[SecretEnvMapping] = &[
    SecretEnvMapping {
        secret_id_env: "ORACLE_DB_PASSWORD_VAULT_SECRET_ID",
        target: SecretTarget::OracleDbPassword,
    },
    SecretEnvMapping {
        secret_id_env: "ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID",
        target: SecretTarget::OracleDbWalletPassword,
    },
    SecretEnvMapping {
        secret_id_env: "AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID",
        target: SecretTarget::AdminPasswordHash,
    },
];

#[derive(Clone, Copy)]
struct SecretEnvMapping {
    secret_id_env: &'static str,
    target: SecretTarget,
}

#[derive(Clone, Copy)]
enum SecretTarget {
    OracleDbPassword,
    OracleDbWalletPassword,
    AdminPasswordHash,
}

pub async fn resolve_secret_references() -> Result<RuntimeSecretOverrides, String> {
    let requested = requested_mappings();
    if requested.is_empty() {
        return Ok(RuntimeSecretOverrides::default());
    }

    tracing::info!(
        count = requested.len(),
        "resolving runtime secrets from OCI Vault"
    );
    let client = OciVaultSecretClient::new()?;
    let mut resolved = Vec::with_capacity(requested.len());
    for mapping in requested {
        let secret_id = env_value(mapping.secret_id_env).expect("requested mapping has secret id");
        let value = client.read_current_secret(&secret_id).await?;
        resolved.push((mapping, value));
    }
    let resolved_count = resolved.len();
    let overrides = resolved_overrides(resolved)?;
    tracing::info!(
        count = resolved_count,
        "resolved runtime secret values from OCI Vault"
    );
    Ok(overrides)
}

fn requested_mappings() -> Vec<SecretEnvMapping> {
    requested_mappings_from(env_value)
}

fn requested_mappings_from(
    mut lookup: impl FnMut(&str) -> Option<String>,
) -> Vec<SecretEnvMapping> {
    SECRET_ENV_MAPPINGS
        .iter()
        .copied()
        .filter(|mapping| lookup(mapping.secret_id_env).is_some())
        .collect()
}

fn env_value(name: &str) -> Option<String> {
    // ast-grep-ignore: no-distributed-env-read
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn resolved_overrides(
    resolved: Vec<(SecretEnvMapping, String)>,
) -> Result<RuntimeSecretOverrides, String> {
    let mut overrides = RuntimeSecretOverrides::default();
    for (mapping, value) in resolved {
        if value.trim().is_empty() {
            return Err(format!(
                "OCI Vault secret referenced by {} resolved to a blank value",
                mapping.secret_id_env
            ));
        }
        match mapping.target {
            SecretTarget::OracleDbPassword => overrides.oracle_db_password = Some(value),
            SecretTarget::OracleDbWalletPassword => {
                overrides.oracle_db_wallet_password = Some(value)
            }
            SecretTarget::AdminPasswordHash => overrides.admin_password_hash = Some(value),
        }
    }
    Ok(overrides)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ControllerConfig;
    use std::collections::HashMap;

    #[test]
    fn secret_ids_request_vault_resolution_even_when_direct_values_exist() {
        let id_env = "ORACLE_DB_PASSWORD_VAULT_SECRET_ID";
        let mut env = HashMap::new();

        assert!(
            !requested_mappings_from(|name| env.get(name).cloned())
                .iter()
                .any(|mapping| mapping.secret_id_env == id_env)
        );

        env.insert(id_env, "ocid1.vaultsecret.example".to_owned());
        assert!(
            requested_mappings_from(|name| env.get(name).cloned())
                .iter()
                .any(|mapping| mapping.secret_id_env == id_env)
        );

        env.insert("ORACLE_DB_PASSWORD", "already-present".to_owned());
        assert!(
            requested_mappings_from(|name| env.get(name).cloned())
                .iter()
                .any(|mapping| mapping.secret_id_env == id_env)
        );
    }

    #[test]
    fn resolved_secrets_flow_into_controller_config_without_environment_mutation() {
        let mut config = ControllerConfig::for_test(true);
        config.oracle_user = Some("ADMIN".to_owned());
        config.oracle_password = Some("direct-database-password".to_owned());
        config.oracle_connect_string = Some("autographsdb_medium".to_owned());
        config.oracle_wallet_password = Some("direct-wallet-password".to_owned());
        config.admin_password_hash = Some("direct-admin-hash".to_owned());

        let overrides = resolved_overrides(vec![
            (
                SECRET_ENV_MAPPINGS[0],
                "resolved-database-password".to_owned(),
            ),
            (
                SECRET_ENV_MAPPINGS[1],
                "resolved-wallet-password".to_owned(),
            ),
            (SECRET_ENV_MAPPINGS[2], "resolved-admin-hash".to_owned()),
        ])
        .expect("build typed secret overrides");

        config.apply_secret_overrides(overrides);

        assert_eq!(
            config.oracle_password.as_deref(),
            Some("resolved-database-password")
        );
        assert_eq!(
            config.oracle_wallet_password.as_deref(),
            Some("resolved-wallet-password")
        );
        assert_eq!(
            config.admin_password_hash.as_deref(),
            Some("resolved-admin-hash")
        );
        assert!(config.oracle_configured);
    }
}
