use std::collections::HashSet;

use crate::oci_secrets::OciVaultSecretClient;

const SECRET_ENV_MAPPINGS: &[SecretEnvMapping] = &[
    SecretEnvMapping {
        value_env: "ORACLE_DB_PASSWORD",
        secret_id_env: "ORACLE_DB_PASSWORD_VAULT_SECRET_ID",
    },
    SecretEnvMapping {
        value_env: "ORACLE_DB_WALLET_PASSWORD",
        secret_id_env: "ORACLE_DB_WALLET_PASSWORD_VAULT_SECRET_ID",
    },
    SecretEnvMapping {
        value_env: "AUTOGRAPHS_ADMIN_PASSWORD_HASH",
        secret_id_env: "AUTOGRAPHS_ADMIN_PASSWORD_HASH_VAULT_SECRET_ID",
    },
    SecretEnvMapping {
        value_env: "AUTOGRAPHS_OPERATOR_API_TOKEN",
        secret_id_env: "AUTOGRAPHS_OPERATOR_API_TOKEN_VAULT_SECRET_ID",
    },
];

#[derive(Clone, Copy)]
struct SecretEnvMapping {
    value_env: &'static str,
    secret_id_env: &'static str,
}

pub async fn resolve_env_secret_references() -> Result<(), String> {
    let requested = requested_mappings();
    if requested.is_empty() {
        return Ok(());
    }

    tracing::info!(
        count = requested.len(),
        "resolving runtime secrets from OCI Vault"
    );
    let client = OciVaultSecretClient::new()?;
    let mut resolved_secret_ids = HashSet::new();
    for mapping in requested {
        let secret_id = env_value(mapping.secret_id_env).expect("requested mapping has secret id");
        let value = client.read_current_secret(&secret_id).await?;
        if value.trim().is_empty() {
            return Err(format!(
                "OCI Vault secret referenced by {} resolved to a blank value",
                mapping.secret_id_env
            ));
        }
        set_env_before_controller_threads(mapping.value_env, value);
        resolved_secret_ids.insert(mapping.secret_id_env);
    }
    tracing::info!(
        count = resolved_secret_ids.len(),
        "resolved runtime secret values from OCI Vault"
    );
    Ok(())
}

fn requested_mappings() -> Vec<SecretEnvMapping> {
    SECRET_ENV_MAPPINGS
        .iter()
        .copied()
        .filter(|mapping| {
            env_value(mapping.secret_id_env).is_some() && env_value(mapping.value_env).is_none()
        })
        .collect()
}

fn env_value(name: &str) -> Option<String> {
    // ast-grep-ignore: no-distributed-env-read
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn set_env_before_controller_threads(name: &str, value: String) {
    // SAFETY: `main` calls this during the single-threaded startup resolver,
    // before constructing the controller's multi-thread Tokio runtime or
    // spawning background heartbeat/listener tasks. Later controller code reads
    // these env vars through existing config paths but does not mutate them.
    unsafe {
        std::env::set_var(name, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_missing_values_with_secret_ids_are_requested() {
        let id_env = "ORACLE_DB_PASSWORD_VAULT_SECRET_ID";
        let value_env = "ORACLE_DB_PASSWORD";
        with_env_removed(id_env);
        with_env_removed(value_env);

        assert!(
            !requested_mappings()
                .iter()
                .any(|mapping| mapping.secret_id_env == id_env)
        );

        set_env_before_controller_threads(id_env, "ocid1.vaultsecret.example".to_owned());
        assert!(
            requested_mappings()
                .iter()
                .any(|mapping| mapping.secret_id_env == id_env)
        );

        set_env_before_controller_threads(value_env, "already-present".to_owned());
        assert!(
            !requested_mappings()
                .iter()
                .any(|mapping| mapping.secret_id_env == id_env)
        );

        with_env_removed(id_env);
        with_env_removed(value_env);
    }

    fn with_env_removed(name: &str) {
        // SAFETY: These unit tests run synchronously and clean up their own
        // process env keys; the test only uses task-specific env names already
        // owned by the controller configuration contract.
        unsafe {
            std::env::remove_var(name);
        }
    }
}
