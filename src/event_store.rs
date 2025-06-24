//! Event store integration for Person domain
//!
//! This module demonstrates how to integrate with an event store
//! to load and save Person aggregates using event sourcing.

use crate::aggregate::{Person, PersonId};
use crate::events::PersonEvent;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Trait for loading Person aggregates from an event store
#[async_trait]
pub trait PersonEventStore: Send + Sync {
    /// Load all events for a person
    async fn load_events(&self, person_id: PersonId) -> Result<Vec<PersonEvent>, EventStoreError>;
    
    /// Load events after a specific version (for snapshots)
    async fn load_events_after_version(
        &self,
        person_id: PersonId,
        version: u64,
    ) -> Result<Vec<PersonEvent>, EventStoreError>;
    
    /// Save new events for a person
    async fn save_events(
        &self,
        person_id: PersonId,
        events: Vec<PersonEvent>,
        expected_version: Option<u64>,
    ) -> Result<(), EventStoreError>;
    
    /// Load a snapshot if available
    async fn load_snapshot(
        &self,
        person_id: PersonId,
    ) -> Result<Option<(Person, u64)>, EventStoreError>;
    
    /// Save a snapshot
    async fn save_snapshot(
        &self,
        person: &Person,
    ) -> Result<(), EventStoreError>;
}

/// Repository for loading Person aggregates from event store
pub struct PersonRepository<ES: PersonEventStore> {
    event_store: ES,
    snapshot_frequency: u64,
}

impl<ES: PersonEventStore> PersonRepository<ES> {
    pub fn new(event_store: ES) -> Self {
        Self {
            event_store,
            snapshot_frequency: 100, // Create snapshot every 100 events
        }
    }
    
    /// Load a person by replaying their event stream
    pub async fn load(&self, person_id: PersonId) -> Result<Option<Person>, EventStoreError> {
        // Try to load from snapshot first
        if let Some((snapshot, version)) = self.event_store.load_snapshot(person_id).await? {
            // Load events after snapshot
            let events = self.event_store
                .load_events_after_version(person_id, version)
                .await?;
            
            if events.is_empty() {
                // Snapshot is up to date
                Ok(Some(snapshot))
            } else {
                // Apply events after snapshot
                let person = Person::replay_from_snapshot(snapshot, events, version)
                    .map_err(|e| EventStoreError::ReplayError(e))?;
                Ok(Some(person))
            }
        } else {
            // No snapshot, load all events
            let events = self.event_store.load_events(person_id).await?;
            
            if events.is_empty() {
                Ok(None)
            } else {
                let person = Person::replay_events(events)
                    .map_err(|e| EventStoreError::ReplayError(e))?;
                Ok(Some(person))
            }
        }
    }
    
    /// Save a person by storing new events
    pub async fn save(
        &self,
        person: &Person,
        events: Vec<PersonEvent>,
        expected_version: Option<u64>,
    ) -> Result<(), EventStoreError> {
        if events.is_empty() {
            return Ok(());
        }
        
        // Save events
        self.event_store
            .save_events(person.id, events, expected_version)
            .await?;
        
        // Check if we need to create a snapshot
        if person.version() % self.snapshot_frequency == 0 {
            self.event_store.save_snapshot(person).await?;
        }
        
        Ok(())
    }
}

/// Errors that can occur in event store operations
#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Event replay failed: {0}")]
    ReplayError(String),
    
    #[error("Concurrent modification detected: expected version {expected}, but was {actual}")]
    ConcurrentModification { expected: u64, actual: u64 },
    
    #[error("Event store error: {0}")]
    StoreError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// In-memory event store for testing
pub struct InMemoryEventStore {
    events: RwLock<HashMap<PersonId, Vec<PersonEvent>>>,
    snapshots: RwLock<HashMap<PersonId, (Person, u64)>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
            snapshots: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl PersonEventStore for InMemoryEventStore {
    async fn load_events(&self, person_id: PersonId) -> Result<Vec<PersonEvent>, EventStoreError> {
        let events = self.events.read().await;
        Ok(events.get(&person_id).cloned().unwrap_or_default())
    }
    
    async fn load_events_after_version(
        &self,
        person_id: PersonId,
        version: u64,
    ) -> Result<Vec<PersonEvent>, EventStoreError> {
        let events = self.events.read().await;
        if let Some(all_events) = events.get(&person_id) {
            Ok(all_events.iter()
                .skip(version as usize)
                .cloned()
                .collect())
        } else {
            Ok(vec![])
        }
    }
    
    async fn save_events(
        &self,
        person_id: PersonId,
        new_events: Vec<PersonEvent>,
        expected_version: Option<u64>,
    ) -> Result<(), EventStoreError> {
        let mut events = self.events.write().await;
        let person_events = events.entry(person_id).or_insert_with(Vec::new);
        
        // Check expected version
        if let Some(expected) = expected_version {
            let actual = person_events.len() as u64;
            if actual != expected {
                return Err(EventStoreError::ConcurrentModification { expected, actual });
            }
        }
        
        person_events.extend(new_events);
        Ok(())
    }
    
    async fn load_snapshot(
        &self,
        person_id: PersonId,
    ) -> Result<Option<(Person, u64)>, EventStoreError> {
        let snapshots = self.snapshots.read().await;
        Ok(snapshots.get(&person_id).cloned())
    }
    
    async fn save_snapshot(
        &self,
        person: &Person,
    ) -> Result<(), EventStoreError> {
        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(person.id, (person.clone(), person.version()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::*;
    use crate::events::PersonCreated;
    use crate::value_objects::*;
    use crate::aggregate::PersonId;
    
    #[tokio::test]
    async fn test_repository_load_save() {
        let store = InMemoryEventStore::new();
        let repo = PersonRepository::new(store);
        
        let person_id = PersonId::new();
        let name = PersonName::new("Test".to_string(), "User".to_string());
        
        // Create person with PersonCreated event
        let create_event = PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: name.clone(),
            source: "Test".to_string(),
            created_at: chrono::Utc::now(),
        });
        
        // Start with empty person and apply creation event
        let mut person = Person::empty();
        person = person.apply_event(&create_event);
        
        // Save the creation event
        repo.save(&person, vec![create_event], None).await.unwrap();
        
        // Now add an email
        let events = person.handle_command(PersonCommand::AddEmail(AddEmail {
            person_id,
            email: EmailAddress {
                address: "test@example.com".to_string(),
                verified: false,
            },
            primary: true,
        })).unwrap();
        
        // Apply events to person
        for event in &events {
            person = person.apply_event(event);
        }
        
        // Save the new events
        repo.save(&person, events, Some(1)).await.unwrap();
        
        // Load
        let loaded = repo.load(person_id).await.unwrap().unwrap();
        
        assert_eq!(loaded.id, person_id);
        assert_eq!(loaded.emails.len(), 1);
        assert_eq!(loaded.version(), 2); // 1 create + 1 email
    }
    
    #[tokio::test]
    async fn test_snapshot_creation() {
        let store = InMemoryEventStore::new();
        let mut repo = PersonRepository::new(store);
        repo.snapshot_frequency = 2; // Snapshot every 2 events
        
        let person_id = PersonId::new();
        let name = PersonName::new("Test".to_string(), "User".to_string());
        
        // Create person with PersonCreated event
        let create_event = PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: name.clone(),
            source: "Test".to_string(),
            created_at: chrono::Utc::now(),
        });
        
        // Start with empty person and apply creation event
        let mut person = Person::empty();
        person = person.apply_event(&create_event);
        
        // Save the creation event
        repo.save(&person, vec![create_event], None).await.unwrap();
        
        // Generate 3 more events (total 4 events, should create 2 snapshots)
        for i in 0..3 {
            let events = person.handle_command(PersonCommand::AddTag(AddTag {
                person_id,
                tag: Tag {
                    name: format!("Tag{}", i),
                    category: "Test".to_string(),
                    added_by: uuid::Uuid::new_v4(),
                    added_at: chrono::Utc::now(),
                },
            })).unwrap();
            
            for event in &events {
                person = person.apply_event(event);
            }
            
            repo.save(&person, events, Some(person.version() - 1)).await.unwrap();
        }
        
        // Load should use snapshot
        let loaded = repo.load(person_id).await.unwrap().unwrap();
        assert_eq!(loaded.version(), 4); // 1 create + 3 tags
        assert_eq!(loaded.tags.len(), 3);
    }
} 