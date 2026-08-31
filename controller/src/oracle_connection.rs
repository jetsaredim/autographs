use std::{future::Future, sync::Arc};

use oracledb::{Config, Connection, ErrorKind};

use crate::oracle_credentials::{
    DatabaseCredential, DatabaseCredentialProvider, DatabaseCredentialRefreshOutcome,
};

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
    connect_with_credential(settings, &credential).map_err(|error| error.message)
}

pub(crate) async fn connect_with_recovery(
    settings: Arc<OracleConnectionSettings>,
) -> Result<Connection, String> {
    let credential_provider = Arc::clone(settings.credential_provider());
    connect_with_recovery_using(credential_provider, move |credential| {
        let settings = Arc::clone(&settings);
        async move {
            tokio::task::spawn_blocking(move || connect_with_credential(&settings, &credential))
                .await
                .map_err(|error| {
                    OracleConnectionAttemptError::other(format!(
                        "join Oracle connection task: {error}"
                    ))
                })?
        }
    })
    .await
}

fn connect_with_credential(
    settings: &OracleConnectionSettings,
    credential: &DatabaseCredential,
) -> Result<Connection, OracleConnectionAttemptError> {
    let mut config = Config::default().set_credentials(&settings.user, credential.expose_secret());

    if let Some(wallet_dir) = settings.wallet_dir.as_deref() {
        let wallet_password = settings.wallet_password.as_deref().ok_or_else(|| {
            OracleConnectionAttemptError::other(
                "ORACLE_DB_WALLET_PASSWORD is required when ORACLE_DB_WALLET_DIR is set",
            )
        })?;
        config = config
            .set_config_dir(wallet_dir)
            .set_wallet_location(wallet_dir)
            .set_wallet_password(wallet_password);
    }

    let config = config
        .set_connect_string(&settings.connect_string)
        .map_err(|error| {
            OracleConnectionAttemptError::other(format!("configure Oracle connect string: {error}"))
        })?;
    let connection = oracledb::connect(config).map_err(OracleConnectionAttemptError::connect)?;
    for sql in SESSION_INITIALIZATION_SQL {
        connection.execute(sql, &[]).map_err(|error| {
            OracleConnectionAttemptError::other(format!("initialize Oracle session: {error}"))
        })?;
    }
    Ok(connection)
}

struct OracleConnectionAttemptError {
    message: String,
    invalid_credential: bool,
}

impl OracleConnectionAttemptError {
    fn connect(error: oracledb::Error) -> Self {
        Self {
            invalid_credential: is_invalid_credential_error_kind(error.kind()),
            message: error.to_string(),
        }
    }

    #[cfg(test)]
    fn invalid_credential(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            invalid_credential: true,
        }
    }

    fn other(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            invalid_credential: false,
        }
    }
}

fn is_invalid_credential_error_kind(kind: &ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::DbError(message)
            if message
                .trim_start()
                .split_once(':')
                .is_some_and(|(code, _)| code == "ORA-01017")
    )
}

async fn connect_with_recovery_using<T, Connect, ConnectFuture>(
    credential_provider: Arc<DatabaseCredentialProvider>,
    connect: Connect,
) -> Result<T, String>
where
    Connect: Fn(Arc<DatabaseCredential>) -> ConnectFuture,
    ConnectFuture: Future<Output = Result<T, OracleConnectionAttemptError>>,
{
    let failed_credential = credential_provider.current();
    match connect(Arc::clone(&failed_credential)).await {
        Ok(connection) => Ok(connection),
        Err(error) if error.invalid_credential => {
            let failed_generation = failed_credential.generation();
            drop(failed_credential);
            let outcome = credential_provider
                .refresh_if_stale(failed_generation)
                .await
                .map_err(|refresh_error| {
                    format!(
                        "refresh Oracle database credential after ORA-01017 at generation {failed_generation}: {refresh_error}"
                    )
                })?;

            match outcome {
                DatabaseCredentialRefreshOutcome::NotConfigured => Err(error.message),
                DatabaseCredentialRefreshOutcome::Refreshed { generation } => {
                    tracing::info!(
                        failed_generation,
                        generation,
                        "refreshed Oracle database credential after ORA-01017"
                    );
                    connect(credential_provider.current())
                        .await
                        .map_err(|retry_error| {
                            format!(
                                "connect to Oracle after database credential refresh: {}",
                                retry_error.message
                            )
                        })
                }
                DatabaseCredentialRefreshOutcome::AlreadyRefreshed { generation } => {
                    tracing::info!(
                        failed_generation,
                        generation,
                        "reusing concurrently refreshed Oracle database credential after ORA-01017"
                    );
                    connect(credential_provider.current())
                        .await
                        .map_err(|retry_error| {
                            format!(
                                "connect to Oracle after concurrent database credential refresh: {}",
                                retry_error.message
                            )
                        })
                }
            }
        }
        Err(error) => Err(error.message),
    }
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
    use async_trait::async_trait;
    use std::sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use crate::oracle_credentials::DatabaseCredentialRefreshSource;

    struct TestRefreshSource {
        reads: AtomicUsize,
        result: Result<String, String>,
    }

    #[async_trait]
    impl DatabaseCredentialRefreshSource for TestRefreshSource {
        async fn read_current(&self) -> Result<String, String> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            self.result.clone()
        }
    }

    fn refreshable_provider(
        result: Result<&str, &str>,
    ) -> (Arc<DatabaseCredentialProvider>, Arc<TestRefreshSource>) {
        let source = Arc::new(TestRefreshSource {
            reads: AtomicUsize::new(0),
            result: result.map(str::to_owned).map_err(str::to_owned),
        });
        let provider = Arc::new(DatabaseCredentialProvider::with_refresh_source(
            "original-password".to_owned(),
            source.clone(),
        ));
        (provider, source)
    }

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

    #[test]
    fn only_exact_ora_01017_database_errors_are_invalid_credentials() {
        assert!(is_invalid_credential_error_kind(&ErrorKind::DbError(
            "ORA-01017: invalid credential or not authorized; logon denied".to_owned()
        )));
        assert!(is_invalid_credential_error_kind(&ErrorKind::DbError(
            "  ORA-01017: invalid credential\nHelp: example".to_owned()
        )));
        assert!(!is_invalid_credential_error_kind(&ErrorKind::DbError(
            "ORA-01017x: invalid credential".to_owned()
        )));
        assert!(!is_invalid_credential_error_kind(&ErrorKind::DbError(
            "wrapper: ORA-01017: invalid credential".to_owned()
        )));
        assert!(!is_invalid_credential_error_kind(&ErrorKind::NoCredentials));
    }

    #[tokio::test]
    async fn invalid_credential_refreshes_and_retries_once() {
        let (provider, source) = refreshable_provider(Ok("rotated-password"));
        let attempts = Arc::new(Mutex::new(Vec::new()));

        let result = connect_with_recovery_using(provider, {
            let attempts = Arc::clone(&attempts);
            move |credential| {
                let attempts = Arc::clone(&attempts);
                async move {
                    attempts.lock().unwrap().push((
                        credential.generation(),
                        credential.expose_secret().to_owned(),
                    ));
                    if credential.generation() == 0 {
                        Err(OracleConnectionAttemptError::invalid_credential(
                            "ORA-01017",
                        ))
                    } else {
                        Ok("connected")
                    }
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(result, "connected");
        assert_eq!(
            *attempts.lock().unwrap(),
            vec![
                (0, "original-password".to_owned()),
                (1, "rotated-password".to_owned())
            ]
        );
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn concurrent_invalid_credentials_share_one_refresh() {
        let (provider, source) = refreshable_provider(Ok("rotated-password"));
        let first_attempts_ready = Arc::new(tokio::sync::Barrier::new(2));
        let run = || {
            let provider = Arc::clone(&provider);
            let first_attempts_ready = Arc::clone(&first_attempts_ready);
            async move {
                connect_with_recovery_using(provider, move |credential| {
                    let first_attempts_ready = Arc::clone(&first_attempts_ready);
                    async move {
                        if credential.generation() == 0 {
                            first_attempts_ready.wait().await;
                            Err(OracleConnectionAttemptError::invalid_credential(
                                "ORA-01017",
                            ))
                        } else {
                            Ok(credential.generation())
                        }
                    }
                })
                .await
            }
        };

        let (first, second) = tokio::join!(run(), run());

        assert_eq!(first.unwrap(), 1);
        assert_eq!(second.unwrap(), 1);
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn unrelated_connection_error_is_not_refreshed_or_retried() {
        let (provider, source) = refreshable_provider(Ok("rotated-password"));
        let attempts = Arc::new(AtomicUsize::new(0));

        let error = connect_with_recovery_using(provider, {
            let attempts = Arc::clone(&attempts);
            move |_credential| {
                attempts.fetch_add(1, Ordering::SeqCst);
                async { Err::<(), _>(OracleConnectionAttemptError::other("ORA-12154")) }
            }
        })
        .await
        .unwrap_err();

        assert_eq!(error, "ORA-12154");
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(source.reads.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn direct_password_invalid_credential_preserves_single_attempt_behavior() {
        let provider = Arc::new(DatabaseCredentialProvider::new(
            "direct-password".to_owned(),
        ));
        let attempts = Arc::new(AtomicUsize::new(0));

        let error = connect_with_recovery_using(provider, {
            let attempts = Arc::clone(&attempts);
            move |_credential| {
                attempts.fetch_add(1, Ordering::SeqCst);
                async {
                    Err::<(), _>(OracleConnectionAttemptError::invalid_credential(
                        "ORA-01017",
                    ))
                }
            }
        })
        .await
        .unwrap_err();

        assert_eq!(error, "ORA-01017");
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn refresh_failure_does_not_retry() {
        let (provider, source) = refreshable_provider(Err("Vault unavailable"));
        let attempts = Arc::new(AtomicUsize::new(0));

        let error = connect_with_recovery_using(provider, {
            let attempts = Arc::clone(&attempts);
            move |_credential| {
                attempts.fetch_add(1, Ordering::SeqCst);
                async {
                    Err::<(), _>(OracleConnectionAttemptError::invalid_credential(
                        "ORA-01017",
                    ))
                }
            }
        })
        .await
        .unwrap_err();

        assert!(error.contains("refresh Oracle database credential"));
        assert!(error.contains("Vault unavailable"));
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn failed_replacement_credential_is_not_refreshed_or_retried_again() {
        let (provider, source) = refreshable_provider(Ok("still-invalid-password"));
        let attempts = Arc::new(AtomicUsize::new(0));

        let error = connect_with_recovery_using(provider, {
            let attempts = Arc::clone(&attempts);
            move |_credential| {
                attempts.fetch_add(1, Ordering::SeqCst);
                async {
                    Err::<(), _>(OracleConnectionAttemptError::invalid_credential(
                        "ORA-01017",
                    ))
                }
            }
        })
        .await
        .unwrap_err();

        assert!(error.contains("after database credential refresh"));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    }
}
