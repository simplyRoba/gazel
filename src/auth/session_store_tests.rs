use std::sync::Arc;

use time::{Duration, OffsetDateTime};
use tower_sessions::{Expiry, Session, SessionStore};

use super::{AuthStore, Lineage, OWNER_KEY, Owner, SessionFaults, StoreOperation};
use crate::auth::Clock;
use crate::auth::flow::SystemClock;

async fn seed(store: &AuthStore) -> (Session, Arc<Lineage>, Owner) {
    let expiry = OffsetDateTime::now_utc() + Duration::minutes(5);
    let (key, lineage) = store.allocate(expiry);
    let marker = Owner::fresh(key);
    let session = Session::new(
        None,
        Arc::new(store.clone()),
        Some(Expiry::AtDateTime(expiry)),
    );
    {
        let mut authority = lineage.authority.lock().await;
        authority.permit = Some(marker.clone());
        authority.creating = true;
    }
    session.insert(OWNER_KEY, &marker).await.unwrap();
    session.insert("oidc_transaction", "prior").await.unwrap();
    session.save().await.unwrap();
    (session, lineage, marker)
}

fn independent(store: &AuthStore, original: &Session) -> Session {
    Session::new(original.id(), Arc::new(store.clone()), None)
}

async fn rebind(session: &Session, lineage: &Lineage, marker: Owner, state: &str) {
    lineage.authority.lock().await.permit = Some(marker.clone());
    session.insert(OWNER_KEY, marker).await.unwrap();
    session.insert("oidc_transaction", state).await.unwrap();
    session.save().await.unwrap();
}

#[tokio::test]
async fn independent_preloaded_snapshots_cannot_overwrite_a_newer_binding() {
    let faults = SessionFaults::default();
    let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
    let (original, lineage, marker) = seed(&store).await;
    let older = independent(&store, &original);
    let newer = independent(&store, &original);
    for session in [&older, &newer] {
        assert_eq!(
            session
                .get::<String>("oidc_transaction")
                .await
                .unwrap()
                .as_deref(),
            Some("prior")
        );
    }
    let pause = faults.pause_next(StoreOperation::Save);
    faults.fail_next(StoreOperation::Save, true);
    let task = tokio::spawn(async move { older.save().await });
    pause.wait_entered().await;
    let next = Owner::fresh(marker.lineage);
    rebind(&newer, &lineage, next, "winner").await;
    pause.release();
    task.await.unwrap().unwrap();
    assert!(store.current(&newer).await);
    let loaded = independent(&store, &original);
    assert_eq!(
        loaded
            .get::<String>("oidc_transaction")
            .await
            .unwrap()
            .as_deref(),
        Some("winner")
    );
}

#[tokio::test]
async fn authoritative_reload_restores_data_and_absolute_expiry_before_mutation() {
    let store = AuthStore::new(Arc::new(SystemClock), SessionFaults::default());
    let (original, lineage, marker) = seed(&store).await;
    let stale = independent(&store, &original);
    stale.load().await.unwrap();
    stale.insert("outdated", true).await.unwrap();
    let next = Owner::fresh(marker.lineage);
    rebind(&original, &lineage, next.clone(), "winner").await;
    let expiry = lineage
        .authority
        .lock()
        .await
        .committed
        .as_ref()
        .unwrap()
        .expiry_date;
    store.refresh(&stale).await.unwrap();
    assert_eq!(stale.get::<Owner>(OWNER_KEY).await.unwrap(), Some(next));
    assert!(stale.get::<bool>("outdated").await.unwrap().is_none());
    assert_eq!(stale.expiry(), Some(Expiry::AtDateTime(expiry)));
    stale.save().await.unwrap();
    assert_eq!(
        lineage
            .authority
            .lock()
            .await
            .committed
            .as_ref()
            .unwrap()
            .expiry_date,
        expiry
    );
}

#[tokio::test]
async fn deletion_and_revocation_never_upsert_aliases_or_allow_stale_creates() {
    let store = AuthStore::new(Arc::new(SystemClock), SessionFaults::default());
    let (original, lineage, marker) = seed(&store).await;
    let stale = independent(&store, &original);
    stale.load().await.unwrap();
    let old_id = original.id().unwrap();
    lineage.authority.lock().await.creating = true;
    original.cycle_id().await.unwrap();
    original.save().await.unwrap();
    assert_ne!(original.id(), Some(old_id));
    assert!(
        store.for_id(old_id).is_some(),
        "rotation alias permits logout"
    );
    stale.save().await.unwrap();
    assert!(store.backing.load(&old_id).await.unwrap().is_none());
    store.revoke(&lineage).await.unwrap();
    assert!(!store.current(&original).await);
    // Both a cached save and a completely independent create retain the revoked
    // ownership marker but cannot persist any usable record.
    original.save().await.unwrap();
    let late = Session::new(None, Arc::new(store.clone()), None);
    late.insert(OWNER_KEY, marker).await.unwrap();
    late.insert("authenticated", true).await.unwrap();
    late.save().await.unwrap();
    assert!(
        store
            .backing
            .load(&late.id().unwrap())
            .await
            .unwrap()
            .is_none()
    );
    assert!(store.load(&original.id().unwrap()).await.unwrap().is_none());
}

#[tokio::test]
async fn create_retains_backing_memory_store_collision_semantics() {
    let store = AuthStore::new(Arc::new(SystemClock), SessionFaults::default());
    let (first, lineage, _) = seed(&store).await;
    let mut collision = lineage.authority.lock().await.committed.clone().unwrap();
    let (key, next) = store.allocate(collision.expiry_date);
    let marker = Owner::fresh(key);
    collision
        .data
        .insert(OWNER_KEY.into(), serde_json::to_value(&marker).unwrap());
    {
        let mut authority = next.authority.lock().await;
        authority.permit = Some(marker);
        authority.creating = true;
    }
    store.create(&mut collision).await.unwrap();
    assert_ne!(Some(collision.id), first.id());
    assert!(store.current(&first).await);
}

#[tokio::test]
async fn exact_generation_duplicates_skip_backend_failures_but_real_changes_persist() {
    for after_write in [false, true] {
        let faults = SessionFaults::default();
        let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
        let (session, lineage, _) = seed(&store).await;
        let record = lineage.authority.lock().await.committed.clone().unwrap();
        faults.fail_next(StoreOperation::Save, after_write);
        session.save().await.unwrap();
        let saved = store.backing.load(&record.id).await.unwrap().unwrap();
        assert_eq!(saved.data, record.data);
        assert_eq!(saved.expiry_date, record.expiry_date);
        session.insert("real_change", true).await.unwrap();
        session.save().await.unwrap();
        assert_eq!(
            store.backing.load(&record.id).await.unwrap().unwrap().data["real_change"],
            true
        );
        // Even a duplicate must not recreate an absent backing record.
        store.backing.delete(&record.id).await.unwrap();
        session.save().await.unwrap();
        assert!(store.backing.load(&record.id).await.unwrap().is_none());
    }
}

#[tokio::test]
async fn delete_serializes_against_an_independently_cached_save() {
    let faults = SessionFaults::default();
    let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
    let (session, lineage, _) = seed(&store).await;
    let stale = independent(&store, &session);
    stale.load().await.unwrap();
    stale.insert("real_change", true).await.unwrap();
    let id = session.id().unwrap();
    let pause = faults.pause_next(StoreOperation::Delete);
    let delete = {
        let store = store.clone();
        tokio::spawn(async move { store.delete(&id).await })
    };
    pause.wait_entered().await;
    assert!(
        lineage.authority.try_lock().is_err(),
        "deletion holds the same authority lock as writes across backing I/O"
    );
    let saving = faults.pause_next(StoreOperation::Save);
    let save = tokio::spawn(async move { stale.save().await });
    saving.wait_entered().await;
    saving.release();
    assert!(
        !save.is_finished(),
        "delete holds backing serialization across I/O"
    );
    pause.release();
    delete.await.unwrap().unwrap();
    save.await.unwrap().unwrap();
    assert!(store.backing.load(&id).await.unwrap().is_none());
}

#[tokio::test]
async fn failed_create_removes_the_direct_provisional_identifier_and_allocation() {
    for after_write in [false, true] {
        let faults = SessionFaults::default();
        let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
        let expiry = OffsetDateTime::now_utc() + Duration::minutes(5);
        let (key, lineage) = store.allocate(expiry);
        let marker = Owner::fresh(key.clone());
        {
            let mut authority = lineage.authority.lock().await;
            authority.permit = Some(marker.clone());
            authority.creating = true;
        }
        let mut proposed = tower_sessions::session::Record {
            id: tower_sessions::session::Id::default(),
            data: std::collections::HashMap::default(),
            expiry_date: expiry,
        };
        proposed
            .data
            .insert(OWNER_KEY.into(), serde_json::to_value(marker).unwrap());
        faults.fail_next(StoreOperation::Create, after_write);
        assert!(store.create(&mut proposed).await.is_err());
        assert!(store.backing.load(&proposed.id).await.unwrap().is_none());
        assert!(lineage.authority.lock().await.committed.is_none());
        let session = Session::new(None, Arc::new(store.clone()), None);
        store.rollback_binding(&session, &lineage, &key).await;
        assert!(store.lineage(&key).is_none());
        let index = store.index.lock().unwrap();
        assert!(index.lineages.is_empty());
        assert!(index.aliases.is_empty());
    }
}

#[tokio::test]
async fn failed_rotated_auth_create_cleans_the_exact_provisional_id() {
    for after_write in [false, true] {
        let faults = SessionFaults::default();
        let store = AuthStore::new(Arc::new(SystemClock), faults.clone());
        let (session, lineage, previous) = seed(&store).await;
        let old_id = session.id().unwrap();
        let marker = Owner::fresh(previous.lineage);
        {
            let mut authority = lineage.authority.lock().await;
            authority.permit = Some(marker.clone());
            authority.creating = true;
        }
        session.cycle_id().await.unwrap();
        let mut proposed = tower_sessions::session::Record {
            id: tower_sessions::session::Id::default(),
            data: std::collections::HashMap::from([
                (OWNER_KEY.into(), serde_json::to_value(marker).unwrap()),
                (
                    "authenticated".into(),
                    serde_json::json!({"authenticated_at": 1, "expires_at": i64::MAX}),
                ),
            ]),
            expiry_date: OffsetDateTime::now_utc() + Duration::hours(12),
        };
        faults.fail_next(StoreOperation::Create, after_write);
        assert!(store.create(&mut proposed).await.is_err());
        store.revoke(&lineage).await.unwrap();
        for id in [old_id, proposed.id] {
            assert!(store.backing.load(&id).await.unwrap().is_none());
            assert!(store.load(&id).await.unwrap().is_none());
        }
        session.save().await.unwrap();
        assert!(
            store
                .backing
                .load(&session.id().unwrap())
                .await
                .unwrap()
                .is_none()
        );
    }
}

#[derive(Debug)]
struct MutableClock(std::sync::Mutex<OffsetDateTime>);
impl super::Clock for MutableClock {
    fn now(&self) -> OffsetDateTime {
        *self.0.lock().unwrap()
    }
}

#[tokio::test]
async fn coordination_records_and_rotation_aliases_retire_at_absolute_deadline() {
    for ttl in [Duration::minutes(5), Duration::hours(12)] {
        assert_terminal_retirement(ttl).await;
    }
}

async fn assert_terminal_retirement(ttl: Duration) {
    let clock = Arc::new(MutableClock(std::sync::Mutex::new(
        OffsetDateTime::now_utc(),
    )));
    let store = AuthStore::new(clock.clone(), SessionFaults::default());
    let (session, lineage, marker) = seed(&store).await;
    let old_id = session.id().unwrap();
    let expiry = clock.now() + ttl;
    lineage.authority.lock().await.creating = true;
    session.cycle_id().await.unwrap();
    session.set_expiry(Some(Expiry::AtDateTime(expiry)));
    session.save().await.unwrap();
    let rotated_id = session.id().unwrap();
    let expiry = lineage
        .authority
        .lock()
        .await
        .committed
        .as_ref()
        .unwrap()
        .expiry_date;
    {
        let index = store.index.lock().unwrap();
        assert_eq!(index.lineages.len(), 1);
        assert_eq!(index.aliases.len(), 2);
    }
    store.revoke(&lineage).await.unwrap();
    assert!(
        store.for_id(old_id).is_some(),
        "revocation keeps the alias deadline"
    );
    assert!(store.for_id(rotated_id).is_some());
    assert!(store.backing.load(&old_id).await.unwrap().is_none());
    assert!(store.backing.load(&rotated_id).await.unwrap().is_none());
    *clock.0.lock().unwrap() = expiry - Duration::nanoseconds(1);
    assert!(store.lineage(&marker.lineage).is_some());
    assert!(store.for_id(old_id).is_some());
    *clock.0.lock().unwrap() = expiry;
    assert!(store.lineage(&marker.lineage).is_none());
    assert!(store.for_id(session.id().unwrap()).is_none());
    {
        let index = store.index.lock().unwrap();
        assert!(index.aliases.is_empty());
        assert!(index.lineages.is_empty());
    }
    // Pruning terminal coordination must never re-authorize an old marker.
    session.save().await.unwrap();
    assert!(store.backing.load(&rotated_id).await.unwrap().is_none());
    let stale_create = Session::new(None, Arc::new(store.clone()), None);
    stale_create.insert(OWNER_KEY, marker).await.unwrap();
    stale_create.save().await.unwrap();
    assert!(
        store
            .backing
            .load(&stale_create.id().unwrap())
            .await
            .unwrap()
            .is_none()
    );
    let index = store.index.lock().unwrap();
    assert!(index.lineages.is_empty());
    assert!(index.aliases.is_empty());
}
