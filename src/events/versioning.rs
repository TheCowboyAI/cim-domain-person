//! Event versioning and schema evolution support

use cim_domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

/// Trait for versioned events
pub trait VersionedEvent: Serialize + for<'de> Deserialize<'de> + Debug {
    /// Get the event type name
    fn event_type() -> &'static str;
    
    /// Get the current version
    fn version() -> &'static str;
    
    /// Get the schema for this version
    fn schema() -> &'static str {
        ""
    }
}

/// Event version registry
pub struct EventVersionRegistry {
    /// Map from (event_type, version) to migration function
    migrations: HashMap<(String, String), Box<dyn EventMigration>>,
    /// Map from event_type to current version
    current_versions: HashMap<String, String>,
}

impl EventVersionRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
            current_versions: HashMap::new(),
        }
    }
    
    /// Register an event type with its current version
    pub fn register_event<E: VersionedEvent + 'static>(&mut self) {
        let event_type = E::event_type();
        let version = E::version();
        self.current_versions.insert(event_type.to_string(), version.to_string());
    }
    
    /// Register a migration between versions
    pub fn register_migration<M: EventMigration + 'static>(
        &mut self,
        event_type: &str,
        from_version: &str,
        to_version: &str,
        migration: M,
    ) {
        let key = (event_type.to_string(), format!("{}->{}", from_version, to_version));
        self.migrations.insert(key, Box::new(migration));
    }
    
    /// Migrate an event to the current version
    pub fn migrate_to_current(&self, event_type: &str, event_data: Value) -> DomainResult<Value> {
        let version = event_data.get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DomainError::SerializationError("Event missing version".to_string()))?
            .to_string();
        
        let current_version = self.current_versions.get(event_type)
            .ok_or_else(|| DomainError::ValidationError(format!("Unknown event type: {}", event_type)))?;
        
        if version == *current_version {
            return Ok(event_data);
        }
        
        // Find migration path
        self.migrate_event(event_type, &version, current_version, event_data)
    }
    
    /// Migrate an event through version chain
    fn migrate_event(
        &self,
        event_type: &str,
        from_version: &str,
        to_version: &str,
        mut event_data: Value,
    ) -> DomainResult<Value> {
        let mut current_version = from_version.to_string();
        
        // Simple linear migration for now
        // In production, you'd want a graph-based migration path finder
        while current_version != to_version {
            let next_version = self.find_next_version(event_type, &current_version, to_version)?;
            let migration_key = (event_type.to_string(), format!("{}->{}", current_version, next_version));
            
            let migration = self.migrations.get(&migration_key)
                .ok_or_else(|| DomainError::ValidationError(
                    format!("No migration from {} to {} for {}", current_version, next_version, event_type)
                ))?;
            
            event_data = migration.migrate(event_data)?;
            current_version = next_version;
        }
        
        Ok(event_data)
    }
    
    /// Find next version in migration path (simplified)
    fn find_next_version(&self, event_type: &str, from: &str, _to: &str) -> DomainResult<String> {
        // This is simplified - just looks for any migration from current version
        for (key, _) in &self.migrations {
            if key.0 == event_type && key.1.starts_with(&format!("{}->", from)) {
                let parts: Vec<&str> = key.1.split("->").collect();
                if parts.len() == 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }
        
        Err(DomainError::ValidationError(format!("No migration path from {}", from)))
    }
}

/// Trait for event migrations
pub trait EventMigration: Send + Sync {
    /// Migrate event data from one version to another
    fn migrate(&self, event_data: Value) -> DomainResult<Value>;
}

/// Function-based migration
pub struct FunctionMigration<F>
where
    F: Fn(Value) -> DomainResult<Value> + Send + Sync,
{
    migration_fn: F,
}

impl<F> FunctionMigration<F>
where
    F: Fn(Value) -> DomainResult<Value> + Send + Sync,
{
    pub fn new(migration_fn: F) -> Self {
        Self { migration_fn }
    }
}

impl<F> EventMigration for FunctionMigration<F>
where
    F: Fn(Value) -> DomainResult<Value> + Send + Sync,
{
    fn migrate(&self, event_data: Value) -> DomainResult<Value> {
        (self.migration_fn)(event_data)
    }
}

/// Macro to define versioned events
#[macro_export]
macro_rules! versioned_event {
    (
        $name:ident,
        version = $version:expr,
        event_type = $event_type:expr
    ) => {
        impl $crate::events::versioning::VersionedEvent for $name {
            fn event_type() -> &'static str {
                $event_type
            }
            
            fn version() -> &'static str {
                $version
            }
        }
    };
}

/// Event envelope with version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedEventEnvelope {
    /// Event type name
    pub event_type: String,
    /// Event version
    pub version: String,
    /// Event data
    pub data: Value,
    /// Metadata
    pub metadata: crate::infrastructure::EventMetadata,
}

impl VersionedEventEnvelope {
    /// Create a new versioned envelope
    pub fn new<E: VersionedEvent>(event: E, metadata: crate::infrastructure::EventMetadata) -> DomainResult<Self> {
        Ok(Self {
            event_type: E::event_type().to_string(),
            version: E::version().to_string(),
            data: serde_json::to_value(event)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?,
            metadata,
        })
    }
    
    /// Deserialize to a specific event type
    pub fn deserialize_as<E: VersionedEvent>(&self) -> DomainResult<E> {
        serde_json::from_value(self.data.clone())
            .map_err(|e| DomainError::SerializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_event_migration() {
        let mut registry = EventVersionRegistry::new();
        
        // Register a migration from v1 to v2
        registry.register_migration(
            "PersonCreated",
            "1.0",
            "2.0",
            FunctionMigration::new(|mut data| {
                // Add middle_name field in v2
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("middle_name".to_string(), json!(null));
                    obj.insert("version".to_string(), json!("2.0"));
                }
                Ok(data)
            }),
        );
        
        registry.current_versions.insert("PersonCreated".to_string(), "2.0".to_string());
        
        // Test migration
        let v1_event = json!({
            "version": "1.0",
            "person_id": "123",
            "first_name": "John",
            "last_name": "Doe"
        });
        
        let migrated = registry.migrate_to_current("PersonCreated", v1_event).unwrap();
        
        assert_eq!(migrated.get("version").unwrap().as_str().unwrap(), "2.0");
        assert!(migrated.get("middle_name").is_some());
    }
}