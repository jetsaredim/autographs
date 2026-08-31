use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tokio::sync::{Mutex, OnceCell};

use crate::oci_secrets::{OciVaultSecretClient, VaultSecretValue};

pub(crate) struct DatabaseCredential {
    value: String,
    generation: u64,
    vault_version: Option<u64>,
}

impl DatabaseCredential {
    fn new(value: String, generation: u64, vault_version: Option<u64>) -> Self {
        Self {
            value,
            generation,
            vault_version,
        }
    }

    pub(crate) fn expose_secret(&self) -> &str {
        &self.value
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn vault_version(&self) -> Option<u64> {
        self.vault_version
    }
}

#[async_trait]
pub(crate) trait DatabaseCredentialRefreshSource: Send + Sync {
    async fn read_current(&self) -> Result<VaultSecretValue, String>;
}

struct OciVaultDatabaseCredentialRefreshSource {
    client: OciVaultSecretClient,
    secret_id: String,
}

impl OciVaultDatabaseCredentialRefreshSource {
    fn new(secret_id: String) -> Result<Self, String> {
        Ok(Self {
            client: OciVaultSecretClient::new()?,
            secret_id,
        })
    }
}

#[async_trait]
impl DatabaseCredentialRefreshSource for OciVaultDatabaseCredentialRefreshSource {
    async fn read_current(&self) -> Result<VaultSecretValue, String> {
        self.client.read_current_secret(&self.secret_id).await
    }
}

pub(crate) struct DatabaseCredentialProvider {
    current: RwLock<Arc<DatabaseCredential>>,
    refresh_source: Option<Arc<dyn DatabaseCredentialRefreshSource>>,
    active_refresh: Mutex<Option<Arc<DatabaseCredentialRefreshFlight>>>,
}

struct DatabaseCredentialRefreshFlight {
    failed_generation: u64,
    result: OnceCell<Result<DatabaseCredentialRefreshOutcome, String>>,
}

impl DatabaseCredentialRefreshFlight {
    fn new(failed_generation: u64) -> Self {
        Self {
            failed_generation,
            result: OnceCell::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DatabaseCredentialRefreshOutcome {
    NotConfigured,
    Refreshed { generation: u64, vault_version: u64 },
    AlreadyRefreshed { generation: u64, vault_version: u64 },
}

impl DatabaseCredentialProvider {
    pub(crate) fn new(credential: String) -> Self {
        Self::with_optional_refresh_source(credential, None, None)
    }

    pub(crate) fn with_oci_vault_refresh(
        credential: String,
        vault_version: u64,
        secret_id: String,
    ) -> Result<Self, String> {
        let source = OciVaultDatabaseCredentialRefreshSource::new(secret_id)?;
        Ok(Self::with_optional_refresh_source(
            credential,
            Some(vault_version),
            Some(Arc::new(source)),
        ))
    }

    fn with_optional_refresh_source(
        credential: String,
        vault_version: Option<u64>,
        refresh_source: Option<Arc<dyn DatabaseCredentialRefreshSource>>,
    ) -> Self {
        Self {
            current: RwLock::new(Arc::new(DatabaseCredential::new(
                credential,
                0,
                vault_version,
            ))),
            refresh_source,
            active_refresh: Mutex::new(None),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_refresh_source(
        credential: String,
        vault_version: u64,
        refresh_source: Arc<dyn DatabaseCredentialRefreshSource>,
    ) -> Self {
        Self::with_optional_refresh_source(credential, Some(vault_version), Some(refresh_source))
    }

    pub(crate) fn current(&self) -> Arc<DatabaseCredential> {
        Arc::clone(
            &self
                .current
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }

    pub(crate) async fn refresh_if_stale(
        &self,
        failed_generation: u64,
    ) -> Result<DatabaseCredentialRefreshOutcome, String> {
        let Some(source) = self.refresh_source.as_ref() else {
            return Ok(DatabaseCredentialRefreshOutcome::NotConfigured);
        };

        let (flight, joined_existing) = {
            let mut active_refresh = self.active_refresh.lock().await;
            let current = self.current();
            if current.generation() != failed_generation {
                let vault_version = current.vault_version().ok_or_else(|| {
                    "refreshable database credential is missing its Vault version".to_owned()
                })?;
                return Ok(DatabaseCredentialRefreshOutcome::AlreadyRefreshed {
                    generation: current.generation(),
                    vault_version,
                });
            }

            match active_refresh.as_ref() {
                Some(flight) if flight.failed_generation == failed_generation => {
                    (Arc::clone(flight), true)
                }
                _ => {
                    let flight = Arc::new(DatabaseCredentialRefreshFlight::new(failed_generation));
                    *active_refresh = Some(Arc::clone(&flight));
                    (flight, false)
                }
            }
        };

        let result = flight
            .result
            .get_or_init(|| self.refresh_from_source(source, failed_generation))
            .await
            .clone();

        let mut active_refresh = self.active_refresh.lock().await;
        if active_refresh
            .as_ref()
            .is_some_and(|active| Arc::ptr_eq(active, &flight))
        {
            *active_refresh = None;
        }
        drop(active_refresh);

        match (joined_existing, result) {
            (
                true,
                Ok(DatabaseCredentialRefreshOutcome::Refreshed {
                    generation,
                    vault_version,
                }),
            ) => Ok(DatabaseCredentialRefreshOutcome::AlreadyRefreshed {
                generation,
                vault_version,
            }),
            (_, result) => result,
        }
    }

    async fn refresh_from_source(
        &self,
        source: &Arc<dyn DatabaseCredentialRefreshSource>,
        failed_generation: u64,
    ) -> Result<DatabaseCredentialRefreshOutcome, String> {
        let refreshed = source.read_current().await?;
        if refreshed.expose_secret().trim().is_empty() {
            return Err("OCI Vault database credential resolved to a blank value".to_owned());
        }
        let generation = failed_generation
            .checked_add(1)
            .ok_or_else(|| "database credential generation overflowed".to_owned())?;
        let vault_version = refreshed.version_number();
        let refreshed = Arc::new(DatabaseCredential::new(
            refreshed.into_secret(),
            generation,
            Some(vault_version),
        ));
        *self
            .current
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = refreshed;

        Ok(DatabaseCredentialRefreshOutcome::Refreshed {
            generation,
            vault_version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use tokio::sync::Semaphore;

    struct TestRefreshSource {
        reads: AtomicUsize,
        result: Result<VaultSecretValue, String>,
    }

    impl TestRefreshSource {
        fn new(result: Result<&str, &str>) -> Self {
            Self {
                reads: AtomicUsize::new(0),
                result: result
                    .map(|value| VaultSecretValue::new(value.to_owned(), 2))
                    .map_err(str::to_owned),
            }
        }
    }

    #[async_trait]
    impl DatabaseCredentialRefreshSource for TestRefreshSource {
        async fn read_current(&self) -> Result<VaultSecretValue, String> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            self.result
                .as_ref()
                .map(|secret| {
                    VaultSecretValue::new(
                        secret.expose_secret().to_owned(),
                        secret.version_number(),
                    )
                })
                .map_err(Clone::clone)
        }
    }

    struct BlockingFailureRefreshSource {
        reads: AtomicUsize,
        entered: Semaphore,
        release: Semaphore,
    }

    impl BlockingFailureRefreshSource {
        fn new() -> Self {
            Self {
                reads: AtomicUsize::new(0),
                entered: Semaphore::new(0),
                release: Semaphore::new(0),
            }
        }
    }

    #[async_trait]
    impl DatabaseCredentialRefreshSource for BlockingFailureRefreshSource {
        async fn read_current(&self) -> Result<VaultSecretValue, String> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            self.entered.add_permits(1);
            self.release
                .acquire()
                .await
                .expect("test semaphore remains open")
                .forget();
            Err("Vault unavailable".to_owned())
        }
    }

    #[test]
    fn static_provider_reuses_one_credential_allocation() {
        let provider = DatabaseCredentialProvider::new("database-password".to_owned());

        let first = provider.current();
        let second = provider.current();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.expose_secret(), "database-password");
        assert_eq!(first.generation(), 0);
        assert_eq!(first.vault_version(), None);
    }

    #[tokio::test]
    async fn static_provider_does_not_refresh() {
        let provider = DatabaseCredentialProvider::new("database-password".to_owned());

        assert_eq!(
            provider.refresh_if_stale(0).await.unwrap(),
            DatabaseCredentialRefreshOutcome::NotConfigured
        );
        assert_eq!(provider.current().expose_secret(), "database-password");
    }

    #[tokio::test]
    async fn refresh_replaces_the_snapshot_and_advances_its_generation() {
        let source = Arc::new(TestRefreshSource::new(Ok("rotated-password")));
        let provider = DatabaseCredentialProvider::with_refresh_source(
            "database-password".to_owned(),
            1,
            source.clone(),
        );
        let previous = provider.current();

        assert_eq!(
            provider
                .refresh_if_stale(previous.generation())
                .await
                .unwrap(),
            DatabaseCredentialRefreshOutcome::Refreshed {
                generation: 1,
                vault_version: 2,
            }
        );
        let current = provider.current();
        assert_eq!(current.expose_secret(), "rotated-password");
        assert_eq!(current.generation(), 1);
        assert_eq!(current.vault_version(), Some(2));
        assert_eq!(previous.vault_version(), Some(1));
        assert_eq!(previous.expose_secret(), "database-password");
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn failed_and_blank_refreshes_preserve_the_current_snapshot() {
        for result in [Err("Vault unavailable"), Ok("   ")] {
            let source = Arc::new(TestRefreshSource::new(result));
            let provider = DatabaseCredentialProvider::with_refresh_source(
                "database-password".to_owned(),
                1,
                source,
            );
            let previous = provider.current();

            assert!(provider.refresh_if_stale(0).await.is_err());
            let current = provider.current();
            assert!(Arc::ptr_eq(&previous, &current));
            assert_eq!(current.generation(), 0);
        }
    }

    #[tokio::test]
    async fn concurrent_stale_refreshes_read_the_source_once() {
        let source = Arc::new(TestRefreshSource::new(Ok("rotated-password")));
        let provider = Arc::new(DatabaseCredentialProvider::with_refresh_source(
            "database-password".to_owned(),
            1,
            source.clone(),
        ));
        let first = tokio::spawn({
            let provider = Arc::clone(&provider);
            async move { provider.refresh_if_stale(0).await.unwrap() }
        });
        let second = tokio::spawn({
            let provider = Arc::clone(&provider);
            async move { provider.refresh_if_stale(0).await.unwrap() }
        });

        let outcomes = [first.await.unwrap(), second.await.unwrap()];
        assert!(
            outcomes.contains(&DatabaseCredentialRefreshOutcome::Refreshed {
                generation: 1,
                vault_version: 2,
            })
        );
        assert!(
            outcomes.contains(&DatabaseCredentialRefreshOutcome::AlreadyRefreshed {
                generation: 1,
                vault_version: 2,
            })
        );
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn concurrent_failed_refreshes_share_one_error_but_a_later_call_retries() {
        let source = Arc::new(BlockingFailureRefreshSource::new());
        let provider = Arc::new(DatabaseCredentialProvider::with_refresh_source(
            "database-password".to_owned(),
            1,
            source.clone(),
        ));

        let first = tokio::spawn({
            let provider = Arc::clone(&provider);
            async move { provider.refresh_if_stale(0).await }
        });
        source
            .entered
            .acquire()
            .await
            .expect("test semaphore remains open")
            .forget();

        let second = tokio::spawn({
            let provider = Arc::clone(&provider);
            async move { provider.refresh_if_stale(0).await }
        });
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let joined_the_same_flight = provider
                    .active_refresh
                    .lock()
                    .await
                    .as_ref()
                    .is_some_and(|flight| Arc::strong_count(flight) >= 3);
                if joined_the_same_flight {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("both callers should join the active refresh flight");

        source.release.add_permits(1);
        let first_error = first.await.unwrap().unwrap_err();
        let second_error = second.await.unwrap().unwrap_err();

        assert_eq!(first_error, "Vault unavailable");
        assert_eq!(second_error, first_error);
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
        assert_eq!(provider.current().generation(), 0);

        source.release.add_permits(1);
        assert_eq!(
            provider.refresh_if_stale(0).await.unwrap_err(),
            "Vault unavailable"
        );
        assert_eq!(source.reads.load(Ordering::SeqCst), 2);
    }
}
