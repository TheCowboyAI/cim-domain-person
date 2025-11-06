//! Persistence layer for Person domain

use crate::aggregate::{Person, PersonId};
use crate::events::PersonEvent;
use cim_domain::DomainResult;
use std::sync::Arc;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

use super::event_store::EventStore;

/// Snapshot of an aggregate state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSnapshot {
    pub aggregate_id: PersonId,
    pub version: u64,
    pub state: Person,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Snapshot store trait
#[async_trait]
pub trait SnapshotStore: Send + Sync {
    /// Save a snapshot
    async fn save_snapshot(&self, snapshot: PersonSnapshot) -> DomainResult<()>;
    
    /// Load latest snapshot for an aggregate
    async fn get_latest_snapshot(&self, aggregate_id: PersonId) -> DomainResult<Option<PersonSnapshot>>;
    
    /// Delete snapshots older than a certain version
    async fn delete_snapshots_before(&self, aggregate_id: PersonId, version: u64) -> DomainResult<()>;
}

/// In-memory snapshot store
pub struct InMemorySnapshotStore {
    snapshots: Arc<RwLock<HashMap<PersonId, Vec<PersonSnapshot>>>>,
}

impl Default for InMemorySnapshotStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SnapshotStore for InMemorySnapshotStore {
    async fn save_snapshot(&self, snapshot: PersonSnapshot) -> DomainResult<()> {
        let mut store = self.snapshots.write().await;
        let snapshots = store.entry(snapshot.aggregate_id).or_insert_with(Vec::new);
        snapshots.push(snapshot);
        Ok(())
    }
    
    async fn get_latest_snapshot(&self, aggregate_id: PersonId) -> DomainResult<Option<PersonSnapshot>> {
        let store = self.snapshots.read().await;
        Ok(store.get(&aggregate_id)
            .and_then(|snapshots| snapshots.last().cloned()))
    }
    
    async fn delete_snapshots_before(&self, aggregate_id: PersonId, version: u64) -> DomainResult<()> {
        let mut store = self.snapshots.write().await;
        if let Some(snapshots) = store.get_mut(&aggregate_id) {
            snapshots.retain(|s| s.version >= version);
        }
        Ok(())
    }
}

/// Repository for Person aggregates combining event and snapshot stores
pub struct PersonRepository {
    event_store: Arc<dyn EventStore>,
    snapshot_store: Arc<dyn SnapshotStore>,
    snapshot_frequency: u64, // Take snapshot every N events
}

impl PersonRepository {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        snapshot_store: Arc<dyn SnapshotStore>,
        snapshot_frequency: u64,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
            snapshot_frequency,
        }
    }
    
    /// Load a person aggregate
    pub async fn load(&self, aggregate_id: PersonId) -> DomainResult<Option<Person>> {
        // Try to load from snapshot first
        let snapshot = self.snapshot_store.get_latest_snapshot(aggregate_id).await?;
        
        let (person, from_version) = if let Some(snapshot) = snapshot {
            (snapshot.state, snapshot.version + 1)
        } else {
            // Check if aggregate exists
            let events = self.event_store.get_events(aggregate_id).await?;
            if events.is_empty() {
                return Ok(None);
            }
            (Person::empty(), 0)
        };

        // Apply events since snapshot (pure functional)
        let events = self.event_store.get_events_from_version(aggregate_id, from_version).await?;

        use crate::aggregate::EventSourced;
        let person = events.into_iter().try_fold(person, |p, envelope| {
            p.apply_event(&envelope.event)
        })?;

        Ok(Some(person))
    }
    
    /// Save a person aggregate
    pub async fn save(
        &self,
        person: &Person,
        events: Vec<PersonEvent>,
        expected_version: Option<u64>,
    ) -> DomainResult<()> {
        // Save events
        self.event_store.append_events(person.id, events, expected_version).await?;
        
        // Check if we should take a snapshot
        let current_version = self.event_store.get_current_version(person.id).await?;
        if current_version % self.snapshot_frequency == 0 {
            let snapshot = PersonSnapshot {
                aggregate_id: person.id,
                version: current_version,
                state: person.clone(),
                timestamp: chrono::Utc::now(),
            };
            self.snapshot_store.save_snapshot(snapshot).await?;
            
            // Clean up old snapshots
            if current_version > self.snapshot_frequency * 2 {
                self.snapshot_store.delete_snapshots_before(
                    person.id,
                    current_version - self.snapshot_frequency * 2,
                ).await?;
            }
        }
        
        Ok(())
    }
    
    /// Check if a person exists
    pub async fn exists(&self, aggregate_id: PersonId) -> DomainResult<bool> {
        let version = self.event_store.get_current_version(aggregate_id).await?;
        Ok(version > 0)
    }
} 