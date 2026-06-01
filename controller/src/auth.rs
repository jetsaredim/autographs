use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{Error as PasswordHashError, SaltString},
};
use uuid::Uuid;

const MAX_FAILED_LOGINS: u32 = 5;
const LOCKOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct AuthState {
    inner: Arc<Mutex<AuthInner>>,
    admin_password: Option<String>,
    admin_password_hash: Option<String>,
    operator_token: Option<String>,
}

#[derive(Default)]
struct AuthInner {
    sessions: HashSet<String>,
    failed_logins: u32,
    locked_until: Option<Instant>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum LoginError {
    InvalidCredential,
    Locked,
}

impl AuthState {
    pub fn new(
        admin_password: Option<String>,
        admin_password_hash: Option<String>,
        operator_token: Option<String>,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AuthInner::default())),
            admin_password,
            admin_password_hash,
            operator_token,
        }
    }

    pub fn login(&self, password: &str) -> Result<String, LoginError> {
        let mut inner = self.inner.lock().expect("auth state lock");
        if inner
            .locked_until
            .is_some_and(|locked_until| locked_until > Instant::now())
        {
            return Err(LoginError::Locked);
        }

        if !self.verify_password(password) {
            inner.failed_logins += 1;
            if inner.failed_logins >= MAX_FAILED_LOGINS {
                inner.locked_until = Some(Instant::now() + LOCKOUT);
            }
            return Err(LoginError::InvalidCredential);
        }

        inner.failed_logins = 0;
        inner.locked_until = None;
        let session = Uuid::new_v4().to_string();
        inner.sessions.insert(session.clone());
        Ok(session)
    }

    pub fn logout(&self, session: &str) {
        self.inner
            .lock()
            .expect("auth state lock")
            .sessions
            .remove(session);
    }

    pub fn has_session(&self, session: &str) -> bool {
        self.inner
            .lock()
            .expect("auth state lock")
            .sessions
            .contains(session)
    }

    pub fn has_operator_token(&self, token: &str) -> bool {
        self.operator_token.as_deref() == Some(token)
    }

    fn verify_password(&self, password: &str) -> bool {
        if self.admin_password.as_deref() == Some(password) {
            return true;
        }

        self.admin_password_hash
            .as_deref()
            .and_then(|hash| PasswordHash::new(hash).ok())
            .is_some_and(|hash| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &hash)
                    .is_ok()
            })
    }
}

pub fn hash_password(password: &str) -> Result<String, PasswordHashError> {
    use argon2::PasswordHasher;

    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())?;
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_logins_lock_the_single_admin_path() {
        let auth = AuthState::new(Some("correct".to_owned()), None, None);

        for _ in 0..MAX_FAILED_LOGINS {
            assert_eq!(auth.login("wrong"), Err(LoginError::InvalidCredential));
        }

        assert_eq!(auth.login("correct"), Err(LoginError::Locked));
    }
}
