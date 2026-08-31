use std::sync::Arc;

pub(crate) struct DatabaseCredential {
    value: String,
}

impl DatabaseCredential {
    fn new(value: String) -> Self {
        Self { value }
    }

    pub(crate) fn expose_secret(&self) -> &str {
        &self.value
    }
}

pub(crate) struct DatabaseCredentialProvider {
    current: Arc<DatabaseCredential>,
}

impl DatabaseCredentialProvider {
    pub(crate) fn new(credential: String) -> Self {
        Self {
            current: Arc::new(DatabaseCredential::new(credential)),
        }
    }

    pub(crate) fn current(&self) -> Arc<DatabaseCredential> {
        Arc::clone(&self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_provider_reuses_one_credential_allocation() {
        let provider = DatabaseCredentialProvider::new("database-password".to_owned());

        let first = provider.current();
        let second = provider.current();

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.expose_secret(), "database-password");
    }
}
