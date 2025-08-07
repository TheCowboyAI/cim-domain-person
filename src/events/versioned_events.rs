//! Versioned events for Person domain

use super::versioning::{VersionedEvent, EventVersionRegistry, FunctionMigration};
use crate::aggregate::{PersonId, ComponentType};
use crate::value_objects::PersonName;
use crate::infrastructure::EventMetadata;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};

// Macro for defining versioned events
macro_rules! versioned_event {
    ($name:ident, version = $version:expr, event_type = $event_type:expr) => {
        impl VersionedEvent for $name {
            fn version() -> &'static str {
                $version
            }
            
            fn event_type() -> &'static str {
                $event_type
            }
        }
    };
}

// Version 1.0 Events (Original)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreatedV1 {
    pub person_id: PersonId,
    pub name: PersonName,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

versioned_event!(PersonCreatedV1, version = "1.0", event_type = "PersonCreated");

// Version 2.0 Events (With metadata)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreatedV2 {
    pub person_id: PersonId,
    pub name: PersonName,
    pub source: String,
    pub metadata: EventMetadata,
}

versioned_event!(PersonCreatedV2, version = "2.0", event_type = "PersonCreated");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonNameUpdatedV2 {
    pub person_id: PersonId,
    pub old_name: PersonName,
    pub new_name: PersonName,
    pub change_reason: Option<String>,
    pub metadata: EventMetadata,
}

versioned_event!(PersonNameUpdatedV2, version = "2.0", event_type = "PersonNameUpdated");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonActivatedV2 {
    pub person_id: PersonId,
    pub reason: String,
    pub metadata: EventMetadata,
}

versioned_event!(PersonActivatedV2, version = "2.0", event_type = "PersonActivated");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSuspendedV2 {
    pub person_id: PersonId,
    pub reason: String,
    pub metadata: EventMetadata,
}

versioned_event!(PersonSuspendedV2, version = "2.0", event_type = "PersonSuspended");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonArchivedV2 {
    pub person_id: PersonId,
    pub reason: String,
    pub metadata: EventMetadata,
}

versioned_event!(PersonArchivedV2, version = "2.0", event_type = "PersonArchived");

// Component events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAddedV2 {
    pub person_id: PersonId,
    pub component_type: ComponentType,
    pub component_data: serde_json::Value,
    pub metadata: EventMetadata,
}

versioned_event!(ComponentAddedV2, version = "2.0", event_type = "ComponentAdded");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUpdatedV2 {
    pub person_id: PersonId,
    pub component_type: ComponentType,
    pub component_id: uuid::Uuid,
    pub changes: serde_json::Value,
    pub metadata: EventMetadata,
}

versioned_event!(ComponentUpdatedV2, version = "2.0", event_type = "ComponentUpdated");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRemovedV2 {
    pub person_id: PersonId,
    pub component_type: ComponentType,
    pub component_id: uuid::Uuid,
    pub metadata: EventMetadata,
}

versioned_event!(ComponentRemovedV2, version = "2.0", event_type = "ComponentRemoved");

// Version 3.0 Events (Future - with additional fields)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreatedV3 {
    pub person_id: PersonId,
    pub name: PersonName,
    pub source: String,
    pub source_system: Option<String>, // New field
    pub import_id: Option<String>,     // New field
    pub metadata: EventMetadata,
}

versioned_event!(PersonCreatedV3, version = "3.0", event_type = "PersonCreated");

// Migration implementations

/// Create and configure the event version registry
pub fn create_event_registry() -> EventVersionRegistry {
    let mut registry = EventVersionRegistry::new();
    
    // Register current versions
    registry.register_event::<PersonCreatedV2>();
    registry.register_event::<PersonNameUpdatedV2>();
    registry.register_event::<PersonActivatedV2>();
    registry.register_event::<PersonSuspendedV2>();
    registry.register_event::<PersonArchivedV2>();
    registry.register_event::<ComponentAddedV2>();
    registry.register_event::<ComponentUpdatedV2>();
    registry.register_event::<ComponentRemovedV2>();
    
    // Register migrations
    
    // PersonCreated v1 -> v2
    registry.register_migration(
        "PersonCreated",
        "1.0",
        "2.0",
        FunctionMigration::new(|mut data| {
            if let Some(obj) = data.as_object_mut() {
                // Remove created_at field
                let created_at = obj.remove("created_at")
                    .unwrap_or(json!(chrono::Utc::now()));
                
                // Add metadata
                obj.insert("metadata".to_string(), json!({
                    "version": "1.0",
                    "correlation_id": uuid::Uuid::new_v4().to_string(),
                    "causation_id": null,
                    "timestamp": created_at,
                    "actor": null,
                    "context": {}
                }));
                
                obj.insert("version".to_string(), json!("2.0"));
            }
            Ok(data)
        }),
    );
    
    // PersonCreated v2 -> v3 (future migration)
    registry.register_migration(
        "PersonCreated",
        "2.0",
        "3.0",
        FunctionMigration::new(|mut data| {
            if let Some(obj) = data.as_object_mut() {
                // Add new fields with defaults
                obj.insert("source_system".to_string(), json!(null));
                obj.insert("import_id".to_string(), json!(null));
                obj.insert("version".to_string(), json!("3.0"));
            }
            Ok(data)
        }),
    );
    
    // NameUpdated v1 -> v2
    registry.register_migration(
        "PersonNameUpdated",
        "1.0",
        "2.0",
        FunctionMigration::new(|mut data| {
            if let Some(obj) = data.as_object_mut() {
                // Remove updated_at field
                let updated_at = obj.remove("updated_at")
                    .unwrap_or(json!(chrono::Utc::now()));
                
                // Rename reason to change_reason
                if let Some(reason) = obj.remove("reason") {
                    obj.insert("change_reason".to_string(), reason);
                }
                
                // Add metadata
                obj.insert("metadata".to_string(), json!({
                    "version": "1.0",
                    "correlation_id": uuid::Uuid::new_v4().to_string(),
                    "causation_id": null,
                    "timestamp": updated_at,
                    "actor": null,
                    "context": {}
                }));
                
                obj.insert("version".to_string(), json!("2.0"));
            }
            Ok(data)
        }),
    );
    
    registry
}

/// Migrate a batch of legacy events to the latest format
#[allow(dead_code)] // Used by migration scripts
pub fn migrate_legacy_events_batch(events: &[crate::events::PersonEvent]) -> Result<Vec<Value>, cim_domain::DomainError> {
    events.iter()
        .map(migrate_legacy_event)
        .collect()
}

/// Helper to migrate old event format to new versioned format
#[allow(dead_code)] // Used by migration scripts
pub fn migrate_legacy_event(event: &crate::events::PersonEvent) -> Result<Value, cim_domain::DomainError> {
    let data = match event {
        crate::events::PersonEvent::PersonCreated(e) => {
            json!({
                "version": "1.0",
                "person_id": e.person_id,
                "name": e.name,
                "source": e.source,
                "created_at": e.created_at,
            })
        }
        crate::events::PersonEvent::NameUpdated(e) => {
            json!({
                "version": "1.0",
                "person_id": e.person_id,
                "old_name": e.old_name,
                "new_name": e.new_name,
                "reason": e.reason,
                "updated_at": e.updated_at,
            })
        }
        crate::events::PersonEvent::PersonDeactivated(e) => {
            json!({
                "version": "1.0",
                "person_id": e.person_id,
                "reason": e.reason,
                "deactivated_at": e.deactivated_at,
            })
        }
        crate::events::PersonEvent::PersonReactivated(e) => {
            json!({
                "version": "1.0",
                "person_id": e.person_id,
                "reason": e.reason,
                "reactivated_at": e.reactivated_at,
            })
        }
        _ => {
            return Err(cim_domain::DomainError::SerializationError(
                "Unsupported legacy event type".to_string()
            ));
        }
    };
    
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::PersonName;
    
    #[test]
    fn test_legacy_event_migration() {
        // Test migrating a single legacy event
        let legacy_event = crate::events::PersonEvent::PersonCreated(crate::events::PersonCreated {
            person_id: crate::aggregate::PersonId::new(),
            name: PersonName::new("John".to_string(), "Doe".to_string()),
            source: "test".to_string(),
            created_at: chrono::Utc::now(),
        });
        
        let migrated = migrate_legacy_event(&legacy_event).unwrap();
        assert!(migrated.get("version").is_some());
        assert_eq!(migrated["version"], "1.0");
        
        // Test batch migration
        let events = vec![legacy_event];
        let batch_result = migrate_legacy_events_batch(&events).unwrap();
        assert_eq!(batch_result.len(), 1);
    }
    
    #[test]
    fn test_person_created_migration() {
        let registry = create_event_registry();
        
        // Create v1 event
        let v1_data = json!({
            "version": "1.0",
            "person_id": "12345",
            "name": {
                "first_name": "John",
                "middle_name": null,
                "last_name": "Doe"
            },
            "source": "test",
            "created_at": "2024-01-01T00:00:00Z"
        });
        
        // Migrate to current version (2.0)
        let migrated = registry.migrate_to_current("PersonCreated", v1_data).unwrap();
        
        // Check migration
        assert_eq!(migrated["version"], "2.0");
        assert!(migrated["metadata"].is_object());
        assert!(migrated["created_at"].is_null());
    }
}