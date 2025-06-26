//! Event Store implementation for Person domain

use async_trait::async_trait;
use cim_domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::aggregate::{Person, PersonId, EventSourced};
use crate::events::PersonEvent;

/// Event wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub aggregate_id: PersonId,
    pub sequence: u64,
    pub event: PersonEvent,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: String,
    pub causation_id: String,
}

/// Event Store trait for persistence
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events for an aggregate
    async fn append_events(
        &self,
        aggregate_id: PersonId,
        events: Vec<PersonEvent>,
        expected_version: Option<u64>,
    ) -> DomainResult<()>;
    
    /// Load all events for an aggregate
    async fn get_events(&self, aggregate_id: PersonId) -> DomainResult<Vec<EventEnvelope>>;
    
    /// Load events from a specific version
    async fn get_events_from_version(
        &self,
        aggregate_id: PersonId,
        from_version: u64,
    ) -> DomainResult<Vec<EventEnvelope>>;
    
    /// Get current version of an aggregate
    async fn get_current_version(&self, aggregate_id: PersonId) -> DomainResult<u64>;
}

/// In-memory event store for testing
pub struct InMemoryEventStore {
    events: Arc<RwLock<HashMap<PersonId, Vec<EventEnvelope>>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append_events(
        &self,
        aggregate_id: PersonId,
        events: Vec<PersonEvent>,
        expected_version: Option<u64>,
    ) -> DomainResult<()> {
        let mut store = self.events.write().await;
        let aggregate_events = store.entry(aggregate_id).or_insert_with(Vec::new);
        
        // Check expected version
        let current_version = aggregate_events.len() as u64;
        if let Some(expected) = expected_version {
            if expected != current_version {
                return Err(DomainError::ConcurrencyConflict {
                    expected,
                    actual: current_version,
                });
            }
        }
        
        // Append events
        for (i, event) in events.into_iter().enumerate() {
            let envelope = EventEnvelope {
                aggregate_id,
                sequence: current_version + i as u64 + 1,
                event,
                timestamp: chrono::Utc::now(),
                correlation_id: uuid::Uuid::new_v4().to_string(),
                causation_id: uuid::Uuid::new_v4().to_string(),
            };
            aggregate_events.push(envelope);
        }
        
        Ok(())
    }
    
    async fn get_events(&self, aggregate_id: PersonId) -> DomainResult<Vec<EventEnvelope>> {
        let store = self.events.read().await;
        Ok(store.get(&aggregate_id).cloned().unwrap_or_default())
    }
    
    async fn get_events_from_version(
        &self,
        aggregate_id: PersonId,
        from_version: u64,
    ) -> DomainResult<Vec<EventEnvelope>> {
        let store = self.events.read().await;
        let events = store.get(&aggregate_id).cloned().unwrap_or_default();
        Ok(events.into_iter().filter(|e| e.sequence >= from_version).collect())
    }
    
    async fn get_current_version(&self, aggregate_id: PersonId) -> DomainResult<u64> {
        let store = self.events.read().await;
        Ok(store.get(&aggregate_id).map(|e| e.len() as u64).unwrap_or(0))
    }
}

/// Load an aggregate from the event store
pub async fn load_aggregate(
    store: &dyn EventStore,
    aggregate_id: PersonId,
) -> DomainResult<Person> {
    let events = store.get_events(aggregate_id).await?;
    
    let mut aggregate = Person::empty();
    aggregate.id = aggregate_id;
    
    for envelope in events {
        aggregate.apply_event(&envelope.event)?;
    }
    
    Ok(aggregate)
}

/// Save aggregate events
pub async fn save_aggregate_events(
    store: &dyn EventStore,
    aggregate_id: PersonId,
    events: Vec<PersonEvent>,
    expected_version: Option<u64>,
) -> DomainResult<()> {
    store.append_events(aggregate_id, events, expected_version).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{PersonCommand, CreatePerson};
    use crate::value_objects::PersonName;
    use crate::aggregate::PersonId;
    
    #[tokio::test]
    async fn test_event_store_append_and_load() {
        let store = InMemoryEventStore::new();
        let person_id = PersonId::new();
        
        // Create person
        let mut person = Person::new(person_id, PersonName::new("John".to_string(), "Doe".to_string()));
        let events = person.handle_command(PersonCommand::CreatePerson(CreatePerson {
            person_id,
            name: PersonName::new("John".to_string(), "Doe".to_string()),
            source: "test".to_string(),
        })).unwrap();
        
        // Save events
        store.append_events(person_id, events, None).await.unwrap();
        
        // Load aggregate
        let loaded = load_aggregate(&store, person_id).await.unwrap();
        assert_eq!(loaded.id, person_id);
        assert_eq!(loaded.core_identity.legal_name.given_name, "John");
    }
    
    #[tokio::test]
    async fn test_concurrent_modification_detection() {
        let store = InMemoryEventStore::new();
        let person_id = PersonId::new();
        
        // Create initial events
        let events = vec![PersonEvent::PersonCreated(crate::events::PersonCreated {
            person_id,
            name: PersonName::new("John".to_string(), "Doe".to_string()),
            source: "test".to_string(),
            created_at: chrono::Utc::now(),
        })];
        
        store.append_events(person_id, events.clone(), None).await.unwrap();
        
        // Try to append with wrong expected version
        let result = store.append_events(person_id, events, Some(0)).await;
        assert!(result.is_err());
    }
} 