//! Integration tests for event versioning

use cim_domain_person::events::{
    create_event_registry, PersonCreatedV2, PersonEventV2, 
    EventMetadata, VersionedEventEnvelope,
};
use cim_domain_person::{aggregate::PersonId, value_objects::PersonName};
use serde_json::json;

#[test]
fn test_event_migration_v1_to_v2() -> Result<(), Box<dyn std::error::Error>> {
    let registry = create_event_registry();
    
    // Create V1 event
    let v1_event = json!({
        "version": "1.0",
        "person_id": "550e8400-e29b-41d4-a716-446655440000",
        "name": {
            "first_name": "John",
            "middle_name": "Q",
            "last_name": "Doe"
        },
        "source": "legacy_import",
        "created_at": "2023-01-01T00:00:00Z"
    });
    
    // Migrate to current version
    let migrated = registry.migrate_to_current("PersonCreated", v1_event)?;
    
    // Verify structure
    assert_eq!(migrated["version"], "2.0");
    assert!(migrated["metadata"].is_object());
    assert_eq!(migrated["metadata"]["version"], "1.0");
    assert_eq!(migrated["metadata"]["timestamp"], "2023-01-01T00:00:00Z");
    
    // Deserialize to typed event
    let event: PersonCreatedV2 = serde_json::from_value(migrated)?;
    assert_eq!(event.source, "legacy_import");
    
    Ok(())
}

#[test]
fn test_event_migration_v2_to_v3() -> Result<(), Box<dyn std::error::Error>> {
    let registry = create_event_registry();
    
    // Create V2 event
    let v2_event = json!({
        "version": "2.0",
        "person_id": "550e8400-e29b-41d4-a716-446655440000",
        "name": {
            "first_name": "Jane",
            "middle_name": null,
            "last_name": "Smith"
        },
        "source": "api",
        "metadata": {
            "version": "1.0",
            "correlation_id": "test-123",
            "timestamp": "2023-06-01T00:00:00Z"
        }
    });
    
    // Migrate to V3
    let migrated = registry.migrate("PersonCreated", v2_event.clone(), "3.0")?;
    
    // Verify V3 structure
    assert_eq!(migrated["version"], "3.0");
    assert!(migrated["ipld_cid"].is_string());
    assert_eq!(migrated["content_hash"], "placeholder_hash");
    
    Ok(())
}

#[test]
fn test_versioned_event_envelope() -> Result<(), Box<dyn std::error::Error>> {
    let person_id = PersonId::new();
    let event = PersonCreatedV2 {
        person_id,
        name: PersonName::new("Test", None, "User")?,
        source: "test".to_string(),
        metadata: EventMetadata::new(),
    };
    
    let envelope = VersionedEventEnvelope::new(event.clone(), EventMetadata::new())?;
    
    assert_eq!(envelope.event_type, "PersonCreatedV2");
    assert_eq!(envelope.version, "2.0");
    assert!(!envelope.event_id.is_empty());
    
    // Serialize and deserialize
    let json = serde_json::to_string(&envelope)?;
    let decoded: VersionedEventEnvelope = serde_json::from_str(&json)?;
    
    assert_eq!(envelope.event_id, decoded.event_id);
    assert_eq!(envelope.version, decoded.version);
    
    Ok(())
}

#[test]
fn test_migration_chain() -> Result<(), Box<dyn std::error::Error>> {
    let registry = create_event_registry();
    
    // Start with V1
    let v1_event = json!({
        "version": "1.0",
        "person_id": "550e8400-e29b-41d4-a716-446655440000",
        "name": {
            "first_name": "Chain",
            "last_name": "Test"
        },
        "source": "test",
        "created_at": "2023-01-01T00:00:00Z"
    });
    
    // Migrate through chain: V1 -> V2 -> V3
    let v2 = registry.migrate("PersonCreated", v1_event, "2.0")?;
    assert_eq!(v2["version"], "2.0");
    
    let v3 = registry.migrate("PersonCreated", v2, "3.0")?;
    assert_eq!(v3["version"], "3.0");
    
    // Direct migration V1 -> V3 should also work
    let v1_event = json!({
        "version": "1.0",
        "person_id": "550e8400-e29b-41d4-a716-446655440000",
        "name": {
            "first_name": "Direct",
            "last_name": "Test"
        },
        "source": "test",
        "created_at": "2023-01-01T00:00:00Z"
    });
    
    let direct_v3 = registry.migrate("PersonCreated", v1_event, "3.0")?;
    assert_eq!(direct_v3["version"], "3.0");
    
    Ok(())
}

#[test]
fn test_unknown_version_handling() -> Result<(), Box<dyn std::error::Error>> {
    let registry = create_event_registry();
    
    // Try to migrate from unknown version
    let unknown_event = json!({
        "version": "99.0",
        "person_id": "550e8400-e29b-41d4-a716-446655440000",
        "data": "unknown"
    });
    
    let result = registry.migrate("PersonCreated", unknown_event, "2.0");
    assert!(result.is_err());
    
    Ok(())
}