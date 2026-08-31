use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::oci_secrets::OciVaultSecretClient;

pub(crate) struct DatabaseCredential {
    value: String,
    generation: u64,
}

impl DatabaseCredential {
    fn new(value: String, generation: u64) -> Self {
        Self { value, generation }
    }

    pub(crate) fn expose_secret(&self) -> &str {
        &self.value
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

#[async_trait]
pub(crate) trait DatabaseCredentialRefreshSource: Send + Sync {
    async fn read_current(&self) -> Result<String, String>;
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
    async fn read_current(&self) -> Result<String, String> {
        self.client.read_current_secret(&self.secret_id).await
    }
}

pub(crate) struct DatabaseCredentialProvider {
    current: RwLock<Arc<DatabaseCredential>>,
    refresh_source: Option<Arc<dyn DatabaseCredentialRefreshSource>>,
    refresh_lock: Mutex<()>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum DatabaseCredentialRefreshOutcome {
    NotConfigured,
    Refreshed { generation: u64 },
    AlreadyRefreshed { generation: u64 },
}

impl DatabaseCredentialProvider {
    pub(crate) fn new(credential: String) -> Self {
        Self::with_optional_refresh_source(credential, None)
    }

    pub(crate) fn with_oci_vault_refresh(
        credential: String,
        secret_id: String,
    ) -> Result<Self, String> {
        let source = OciVaultDatabaseCredentialRefreshSource::new(secret_id)?;
        Ok(Self::with_optional_refresh_source(
            credential,
            Some(Arc::new(source)),
        ))
    }

    fn with_optional_refresh_source(
        credential: String,
        refresh_source: Option<Arc<dyn DatabaseCredentialRefreshSource>>,
    ) -> Self {
        Self {
            current: RwLock::new(Arc::new(DatabaseCredential::new(credential, 0))),
            refresh_source,
            refresh_lock: Mutex::new(()),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_refresh_source(
        credential: String,
        refresh_source: Arc<dyn DatabaseCredentialRefreshSource>,
    ) -> Self {
        Self::with_optional_refresh_source(credential, Some(refresh_source))
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

        let _guard = self.refresh_lock.lock().await;
        let current_generation = self.current().generation();
        if current_generation != failed_generation {
            return Ok(DatabaseCredentialRefreshOutcome::AlreadyRefreshed {
                generation: current_generation,
            });
        }

        let refreshed = source.read_current().await?;
        if refreshed.trim().is_empty() {
            return Err("OCI Vault database credential resolved to a blank value".to_owned());
        }
        let generation = failed_generation
            .checked_add(1)
            .ok_or_else(|| "database credential generation overflowed".to_owned())?;
        *self
            .current
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Arc::new(DatabaseCredential::new(refreshed, generation));

        Ok(DatabaseCredentialRefreshOutcome::Refreshed { generation })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestRefreshSource {
        reads: AtomicUsize,
        result: Result<String, String>,
    }

    impl TestRefreshSource {
        fn new(result: Result<&str, &str>) -> Self {
            Self {
                reads: AtomicUsize::new(0),
                result: result.map(str::to_owned).map_err(str::to_owned),
            }
        }
    }

    #[async_trait]
    impl DatabaseCredentialRefreshSource for TestRefreshSource {
        async fn read_current(&self) -> Result<String, String> {
            self.reads.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            self.result.clone()
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
            source.clone(),
        );
        let previous = provider.current();

        assert_eq!(
            provider
                .refresh_if_stale(previous.generation())
                .await
                .unwrap(),
            DatabaseCredentialRefreshOutcome::Refreshed { generation: 1 }
        );
        let current = provider.current();
        assert_eq!(current.expose_secret(), "rotated-password");
        assert_eq!(current.generation(), 1);
        assert_eq!(previous.expose_secret(), "database-password");
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn failed_and_blank_refreshes_preserve_the_current_snapshot() {
        for result in [Err("Vault unavailable"), Ok("   ")] {
            let source = Arc::new(TestRefreshSource::new(result));
            let provider = DatabaseCredentialProvider::with_refresh_source(
                "database-password".to_owned(),
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
        assert!(outcomes.contains(&DatabaseCredentialRefreshOutcome::Refreshed { generation: 1 }));
        assert!(
            outcomes
                .contains(&DatabaseCredentialRefreshOutcome::AlreadyRefreshed { generation: 1 })
        );
        assert_eq!(source.reads.load(Ordering::SeqCst), 1);
    }
}
