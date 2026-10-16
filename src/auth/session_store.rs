use std::collections::HashMap;
use std::sync::{Arc, Mutex as IndexMutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::sync::{Mutex, Semaphore};
use tower_sessions::session::{Id, Record};
use tower_sessions::{MemoryStore, Session, SessionStore, session_store};

use super::Clock;

#[cfg(test)]
#[path = "session_store_tests.rs"]
mod tests;

pub(super) const OWNER_KEY: &str = "auth_owner";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct Owner {
    pub lineage: String,
    pub revision: String,
}

impl Owner {
    pub fn fresh(lineage: String) -> Self {
        Self {
            lineage,
            revision: uuid::Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Authority {
    pub committed: Option<Record>,
    pub permit: Option<Owner>,
    pub creating: bool,
}

#[derive(Debug)]
pub(super) struct Lineage {
    // Protocol coordination and store serialization are distinct: handlers can save
    // while holding the lifecycle gate; the store never acquires it in reverse.
    pub gate: Mutex<()>,
    pub authority: Mutex<Authority>,
}

#[derive(Default)]
struct Index {
    lineages: HashMap<String, (Arc<Lineage>, OffsetDateTime)>,
    aliases: HashMap<Id, (String, OffsetDateTime)>,
}

/// Per-instance deterministic storage fault seam. No process environment is used.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreOperation {
    Create,
    Save,
    Load,
    Delete,
}

/// A one-shot failure, optionally after the backing write has happened.
#[doc(hidden)]
#[derive(Clone, Debug, Default)]
pub struct SessionFaults {
    failure: Arc<IndexMutex<Option<(StoreOperation, bool)>>>,
    pause: Arc<IndexMutex<Option<(StoreOperation, SessionBarrier)>>>,
}

/// A deterministic, one-shot store barrier, scoped to one backend instance.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct SessionBarrier {
    entered: Arc<Semaphore>,
    released: Arc<Semaphore>,
}

impl SessionBarrier {
    pub async fn wait_entered(&self) {
        if let Ok(permit) = self.entered.acquire().await {
            permit.forget();
        }
    }

    pub fn release(&self) {
        self.released.add_permits(1);
    }
}

impl SessionFaults {
    /// # Panics
    /// Panics if an earlier test poisoned the fault mutex.
    pub fn fail_next(&self, operation: StoreOperation, after_write: bool) {
        *self.failure.lock().expect("fault mutex poisoned") = Some((operation, after_write));
    }

    /// # Panics
    /// Panics if an earlier test poisoned the fault mutex.
    pub fn pause_next(&self, operation: StoreOperation) -> SessionBarrier {
        let barrier = SessionBarrier {
            entered: Arc::new(Semaphore::new(0)),
            released: Arc::new(Semaphore::new(0)),
        };
        *self.pause.lock().expect("fault mutex poisoned") = Some((operation, barrier.clone()));
        barrier
    }

    async fn checkpoint(&self, operation: StoreOperation) -> Option<bool> {
        let fault = {
            let mut fault = self.failure.lock().expect("fault mutex poisoned");
            if fault.as_ref().is_some_and(|(op, _)| *op == operation) {
                fault.take().map(|(_, after)| after)
            } else {
                None
            }
        };
        let pause = {
            let mut pause = self.pause.lock().expect("fault mutex poisoned");
            if pause.as_ref().is_some_and(|(op, _)| *op == operation) {
                pause.take().map(|(_, barrier)| barrier)
            } else {
                None
            }
        };
        if let Some(barrier) = pause {
            barrier.entered.add_permits(1);
            barrier
                .released
                .acquire()
                .await
                .expect("barrier remains open")
                .forget();
        }
        fault
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub struct AuthStore {
    backing: MemoryStore,
    index: Arc<IndexMutex<Index>>,
    clock: Arc<dyn Clock>,
    pub faults: SessionFaults,
}

impl std::fmt::Debug for AuthStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthStore").finish_non_exhaustive()
    }
}

fn owner(record: &Record) -> Option<Owner> {
    serde_json::from_value(record.data.get(OWNER_KEY)?.clone()).ok()
}

fn fault_error() -> session_store::Error {
    session_store::Error::Backend("injected session persistence failure".into())
}

impl AuthStore {
    pub(super) fn new(clock: Arc<dyn Clock>, faults: SessionFaults) -> Self {
        Self {
            backing: MemoryStore::default(),
            index: Arc::default(),
            clock,
            faults,
        }
    }

    fn prune(index: &mut Index, now: OffsetDateTime) {
        index.aliases.retain(|_, (_, expiry)| *expiry > now);
        index.lineages.retain(|_, (_, expiry)| *expiry > now);
    }

    pub(super) fn lineage(&self, key: &str) -> Option<Arc<Lineage>> {
        let mut index = self.index.lock().expect("session index poisoned");
        Self::prune(&mut index, self.clock.now());
        index
            .lineages
            .get(key)
            .map(|(lineage, _)| Arc::clone(lineage))
    }

    pub(super) fn for_id(&self, id: Id) -> Option<Arc<Lineage>> {
        let mut index = self.index.lock().expect("session index poisoned");
        Self::prune(&mut index, self.clock.now());
        let (key, _) = index.aliases.get(&id)?;
        index
            .lineages
            .get(key)
            .map(|(lineage, _)| Arc::clone(lineage))
    }

    pub(super) fn allocate(&self, expiry: OffsetDateTime) -> (String, Arc<Lineage>) {
        let key = uuid::Uuid::new_v4().to_string();
        let lineage = Arc::new(Lineage {
            gate: Mutex::new(()),
            authority: Mutex::new(Authority::default()),
        });
        let mut index = self.index.lock().expect("session index poisoned");
        Self::prune(&mut index, self.clock.now());
        index
            .lineages
            .insert(key.clone(), (Arc::clone(&lineage), expiry));
        (key, lineage)
    }

    pub(super) fn retire_empty(&self, key: &str) {
        // Called only for an allocation which never acquired a backing record.
        self.index
            .lock()
            .expect("session index poisoned")
            .lineages
            .remove(key);
    }

    fn remember(&self, record: &Record, owner: &Owner) {
        let mut index = self.index.lock().expect("session index poisoned");
        // Aliases have the same absolute retirement deadline as the current
        // generation. Rotation never creates permanent tombstones.
        for (key, expiry) in index.aliases.values_mut() {
            if *key == owner.lineage {
                *expiry = record.expiry_date;
            }
        }
        index
            .aliases
            .insert(record.id, (owner.lineage.clone(), record.expiry_date));
        if let Some((_, expiry)) = index.lineages.get_mut(&owner.lineage) {
            *expiry = record.expiry_date;
        }
    }

    pub(super) async fn current(&self, session: &Session) -> bool {
        let Some(id) = session.id() else {
            return false;
        };
        let Ok(Some(marker)) = session.get::<Owner>(OWNER_KEY).await else {
            return false;
        };
        self.owns(id, &marker).await
    }

    pub(super) async fn owns(&self, id: Id, marker: &Owner) -> bool {
        let Some(lineage) = self.lineage(&marker.lineage) else {
            return false;
        };
        let authority = lineage.authority.lock().await;
        authority.committed.as_ref().is_some_and(|record| {
            record.id == id
                && owner(record).as_ref() == Some(marker)
                && record.expiry_date > self.clock.now()
        }) && self.backing.load(&id).await.ok().flatten().is_some()
    }

    pub(super) async fn rollback_binding(&self, session: &Session, lineage: &Lineage, key: &str) {
        {
            let mut authority = lineage.authority.lock().await;
            authority.permit = authority.committed.as_ref().and_then(owner);
            authority.creating = false;
        }
        // Caller holds the lineage gate. Reload the old committed binding and
        // its absolute expiry; never roll back using the request's old snapshot.
        session.clear().await;
        if session.id().is_some() {
            let _ = self.refresh(session).await;
        }
        if lineage.authority.lock().await.committed.is_none() {
            self.retire_empty(key);
        }
    }

    pub(super) async fn revoke(&self, lineage: &Lineage) -> session_store::Result<()> {
        let mut authority = lineage.authority.lock().await;
        authority.permit = None;
        authority.creating = false;
        let record = authority.committed.take();
        if let Some(record) = record {
            // Keep terminal authority and rotation aliases until their existing
            // absolute deadline. Delayed requests must not become fresh logins.
            self.delete_backing(&record.id).await?;
        }
        Ok(())
    }

    async fn delete_backing(&self, id: &Id) -> session_store::Result<()> {
        let fault = self.faults.checkpoint(StoreOperation::Delete).await;
        if fault == Some(false) {
            return Err(fault_error());
        }
        self.backing.delete(id).await?;
        if fault == Some(true) {
            return Err(fault_error());
        }
        Ok(())
    }

    pub(super) async fn refresh(
        &self,
        session: &Session,
    ) -> Result<(), tower_sessions::session::Error> {
        session.load().await?;
        if let Some(lineage) = session.id().and_then(|id| self.for_id(id)) {
            let authority = lineage.authority.lock().await;
            if let Some(record) = authority
                .committed
                .as_ref()
                .filter(|record| Some(record.id) == session.id())
                && session.is_modified()
            {
                session.set_expiry(Some(tower_sessions::Expiry::AtDateTime(record.expiry_date)));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl SessionStore for AuthStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        self.write(record, true).await
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        self.write(&mut record.clone(), false).await
    }

    async fn load(&self, id: &Id) -> session_store::Result<Option<Record>> {
        if self.faults.checkpoint(StoreOperation::Load).await.is_some() {
            return Err(fault_error());
        }
        let Some(lineage) = self.for_id(*id) else {
            return Ok(None);
        };
        let authority = lineage.authority.lock().await;
        if !authority
            .committed
            .as_ref()
            .is_some_and(|record| record.id == *id)
        {
            return Ok(None);
        }
        self.backing.load(id).await
    }

    async fn delete(&self, id: &Id) -> session_store::Result<()> {
        if let Some(lineage) = self.for_id(*id) {
            let _authority = lineage.authority.lock().await;
            self.delete_backing(id).await
        } else {
            self.delete_backing(id).await
        }
    }
}

impl AuthStore {
    async fn write(&self, record: &mut Record, create: bool) -> session_store::Result<()> {
        let operation = if create {
            StoreOperation::Create
        } else {
            StoreOperation::Save
        };
        // Pause before validation so a delayed middleware save can lose ownership
        // while another request progresses. Validation and backing I/O stay atomic.
        let fault = self.faults.checkpoint(operation).await;
        let Some(marker) = owner(record) else {
            return Ok(());
        };
        let Some(lineage) = self.lineage(&marker.lineage) else {
            return Ok(());
        };
        let mut authority = lineage.authority.lock().await;
        if authority.permit.as_ref() != Some(&marker) {
            return Ok(());
        }
        if create {
            if !authority.creating {
                return Ok(());
            }
        } else if !authority
            .committed
            .as_ref()
            .is_some_and(|old| old.id == record.id)
            || self.backing.load(&record.id).await?.is_none()
        {
            // MemoryStore::save upserts; never let a deleted alias reach it.
            return Ok(());
        }
        if !create
            && authority.committed.as_ref().is_some_and(|old| {
                old.id == record.id
                    && old.data == record.data
                    && old.expiry_date == record.expiry_date
            })
        {
            // The manager auto-saves explicitly persisted sessions. Existence was
            // checked above; an exact duplicate is not another backing write.
            return Ok(());
        }
        if fault == Some(false) {
            return Err(fault_error());
        }
        if create {
            self.backing.create(record).await?;
        } else {
            self.backing.save(record).await?;
        }
        if fault == Some(true) {
            if create {
                self.backing.delete(&record.id).await?;
            } else if let Some(old) = &authority.committed {
                self.backing.save(old).await?;
            }
            return Err(fault_error());
        }
        authority.committed = Some(record.clone());
        authority.creating = false;
        self.remember(record, &marker);
        Ok(())
    }
}
