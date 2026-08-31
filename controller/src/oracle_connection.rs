use std::sync::Arc;

use oracledb::{Config, Connection};

use crate::oracle_credentials::DatabaseCredentialProvider;

// Autonomous HIGH/MEDIUM services enable PDML, but controller writes are multi-statement OLTP.
const SESSION_INITIALIZATION_SQL: [&str; 1] = ["alter session disable parallel dml"];

pub struct OracleConnectionSettings {
    user: String,
    credential_provider: Arc<DatabaseCredentialProvider>,
    connect_string: String,
    wallet_dir: Option<String>,
    wallet_password: Option<String>,
}

impl OracleConnectionSettings {
    pub fn new(
        user: String,
        credential: String,
        connect_string: String,
        wallet_dir: Option<String>,
        wallet_password: Option<String>,
    ) -> Self {
        Self::with_credential_provider(
            user,
            Arc::new(DatabaseCredentialProvider::new(credential)),
            connect_string,
            wallet_dir,
            wallet_password,
        )
    }

    pub(crate) fn with_credential_provider(
        user: String,
        credential_provider: Arc<DatabaseCredentialProvider>,
        connect_string: String,
        wallet_dir: Option<String>,
        wallet_password: Option<String>,
    ) -> Self {
        Self {
            user,
            credential_provider,
            connect_string,
            wallet_dir,
            wallet_password,
        }
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn connect_string(&self) -> &str {
        &self.connect_string
    }

    pub(crate) fn credential_provider(&self) -> &Arc<DatabaseCredentialProvider> {
        &self.credential_provider
    }
}

pub fn connect(user: &str, credential: &str, connect_string: &str) -> Result<Connection, String> {
    let settings = OracleConnectionSettings::new(
        user.to_owned(),
        credential.to_owned(),
        connect_string.to_owned(),
        optional_env("ORACLE_DB_WALLET_DIR"),
        optional_env("ORACLE_DB_WALLET_PASSWORD"),
    );
    connect_with_settings(&settings)
}

pub fn connect_with_settings(settings: &OracleConnectionSettings) -> Result<Connection, String> {
    let credential = settings.credential_provider().current();
    let mut config = Config::default().set_credentials(&settings.user, credential.expose_secret());

    if let Some(wallet_dir) = settings.wallet_dir.as_deref() {
        let wallet_password = settings.wallet_password.as_deref().ok_or_else(|| {
            "ORACLE_DB_WALLET_PASSWORD is required when ORACLE_DB_WALLET_DIR is set".to_owned()
        })?;
        config = config
            .set_config_dir(wallet_dir)
            .set_wallet_location(wallet_dir)
            .set_wallet_password(wallet_password);
    }

    let config = config
        .set_connect_string(&settings.connect_string)
        .map_err(|error| format!("configure Oracle connect string: {error}"))?;
    let connection = oracledb::connect(config).map_err(|error| error.to_string())?;
    for sql in SESSION_INITIALIZATION_SQL {
        connection
            .execute(sql, &[])
            .map_err(|error| format!("initialize Oracle session: {error}"))?;
    }
    Ok(connection)
}

fn optional_env(name: &str) -> Option<String> {
    // ast-grep-ignore: no-distributed-env-read
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_sessions_require_serial_dml() {
        assert_eq!(
            SESSION_INITIALIZATION_SQL,
            ["alter session disable parallel dml"]
        );
    }

    #[test]
    fn connection_settings_retain_the_injected_credential_provider() {
        let credential_provider = Arc::new(DatabaseCredentialProvider::new(
            "database-password".to_owned(),
        ));
        let settings = OracleConnectionSettings::with_credential_provider(
            "ADMIN".to_owned(),
            Arc::clone(&credential_provider),
            "autographsdb_medium".to_owned(),
            None,
            None,
        );

        assert!(Arc::ptr_eq(
            settings.credential_provider(),
            &credential_provider
        ));
    }
}
