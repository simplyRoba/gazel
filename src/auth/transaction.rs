use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

use openidconnect::{Nonce, PkceCodeVerifier};
use time::{Duration, OffsetDateTime};
use tokio::sync::Mutex;

use super::return_to::SafeReturnTo;

pub(super) const TRANSACTION_TTL: Duration = Duration::minutes(5);
const MAX_PENDING_LOGIN_TRANSACTIONS: usize = 1_024;

/// Time source used for transaction and authenticated-session expiry.
pub trait Clock: Send + Sync {
    /// Return the current UTC time.
    fn now(&self) -> OffsetDateTime;

    /// Return the current process-local monotonic time.
    #[doc(hidden)]
    fn monotonic_now(&self) -> Instant {
        Instant::now()
    }
}

#[derive(Debug)]
pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

pub(crate) struct LoginTransaction {
    pub(crate) nonce: Nonce,
    pub(crate) pkce_verifier: PkceCodeVerifier,
    pub(crate) return_to: SafeReturnTo,
    pub(crate) expires_at: OffsetDateTime,
}

impl fmt::Debug for LoginTransaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginTransaction")
            .field("nonce", &"[REDACTED]")
            .field("pkce_verifier", &"[REDACTED]")
            .field("return_to", &self.return_to)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[derive(Default)]
pub(crate) struct TransactionRegistry {
    pub(super) entries: Mutex<HashMap<String, LoginTransaction>>,
}

impl fmt::Debug for TransactionRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransactionRegistry")
            .finish_non_exhaustive()
    }
}

impl TransactionRegistry {
    pub(super) async fn replace(
        &self,
        old_state: Option<&str>,
        state: String,
        transaction: LoginTransaction,
        now: OffsetDateTime,
    ) -> Result<(), ()> {
        let mut entries = self.entries.lock().await;
        entries.retain(|_, transaction| transaction.expires_at > now);
        if entries.len() >= MAX_PENDING_LOGIN_TRANSACTIONS {
            return Err(());
        }
        if let Some(old_state) = old_state {
            entries.remove(old_state);
        }
        entries.insert(state, transaction);
        Ok(())
    }

    pub(super) async fn consume(
        &self,
        state: &str,
        now: OffsetDateTime,
    ) -> Option<LoginTransaction> {
        let mut entries = self.entries.lock().await;
        entries.retain(|_, transaction| transaction.expires_at > now);
        entries.remove(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transaction(expires_at: OffsetDateTime) -> LoginTransaction {
        LoginTransaction {
            nonce: Nonce::new(String::from("nonce")),
            pkce_verifier: PkceCodeVerifier::new("x".repeat(43)),
            return_to: SafeReturnTo::new(Some("/settings")),
            expires_at,
        }
    }

    #[tokio::test]
    async fn transaction_registry_expires_at_exactly_five_minutes() {
        let registry = TransactionRegistry::default();
        let start = OffsetDateTime::UNIX_EPOCH;
        let expires_at = start + TRANSACTION_TTL;

        registry
            .replace(
                None,
                String::from("before-boundary"),
                transaction(expires_at),
                start,
            )
            .await
            .unwrap();
        assert!(
            registry
                .consume("before-boundary", expires_at - Duration::nanoseconds(1))
                .await
                .is_some()
        );

        registry
            .replace(
                None,
                String::from("at-boundary"),
                transaction(expires_at),
                start,
            )
            .await
            .unwrap();
        assert!(registry.consume("at-boundary", expires_at).await.is_none());
    }

    async fn full_registry(start: OffsetDateTime) -> TransactionRegistry {
        let registry = TransactionRegistry::default();
        for index in 0..MAX_PENDING_LOGIN_TRANSACTIONS {
            registry
                .replace(
                    None,
                    index.to_string(),
                    transaction(start + TRANSACTION_TTL),
                    start,
                )
                .await
                .unwrap();
        }
        registry
    }

    #[tokio::test]
    async fn transaction_capacity_accepts_last_slot_and_preserves_all_entries_on_rejection() {
        let start = OffsetDateTime::UNIX_EPOCH;
        let registry = full_registry(start).await;
        for old_state in [None, Some("0")] {
            assert_eq!(
                registry
                    .replace(
                        old_state,
                        String::from("rejected"),
                        transaction(start + TRANSACTION_TTL + Duration::seconds(1)),
                        start + Duration::seconds(1)
                    )
                    .await,
                Err(())
            );
            let entries = registry.entries.lock().await;
            assert_eq!(entries.len(), MAX_PENDING_LOGIN_TRANSACTIONS);
            for index in 0..MAX_PENDING_LOGIN_TRANSACTIONS {
                let old = &entries[&index.to_string()];
                assert_eq!(old.expires_at, start + TRANSACTION_TTL);
                assert_eq!(old.return_to.as_str(), "/settings");
                assert_eq!(old.nonce.secret(), "nonce");
                assert_eq!(old.pkce_verifier.secret(), &"x".repeat(43));
            }
        }
        assert!(registry.consume("0", start).await.is_some());
        registry
            .replace(
                None,
                String::from("recovered"),
                transaction(start + TRANSACTION_TTL),
                start,
            )
            .await
            .unwrap();
        assert_eq!(
            registry.entries.lock().await.len(),
            MAX_PENDING_LOGIN_TRANSACTIONS
        );
    }

    #[tokio::test]
    async fn transaction_capacity_prunes_before_replacement_at_exact_five_minutes() {
        let start = OffsetDateTime::UNIX_EPOCH;
        let registry = full_registry(start).await;
        let expires_at = start + TRANSACTION_TTL;
        assert!(
            registry
                .replace(
                    Some("0"),
                    String::from("too-early"),
                    transaction(expires_at + TRANSACTION_TTL),
                    expires_at - Duration::nanoseconds(1)
                )
                .await
                .is_err()
        );
        registry
            .replace(
                Some("0"),
                String::from("fresh"),
                transaction(expires_at + TRANSACTION_TTL),
                expires_at,
            )
            .await
            .unwrap();
        let entries = registry.entries.lock().await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries["fresh"].expires_at, expires_at + TRANSACTION_TTL);
    }
}
