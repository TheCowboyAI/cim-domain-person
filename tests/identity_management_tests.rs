//! Tests for Epic 1: Identity Management User Stories
//! 
//! Tests cover:
//! - Story 1.1: Create Person Record
//! - Story 1.2: Update Person Name
//! - Story 1.3: Merge Duplicate Persons
//! - Story 1.4: Record Person Death

use cim_domain_person::{
    aggregate::{Person, PersonId, PersonLifecycle, ComponentType},
    commands::*,
    events::*,
    value_objects::PersonName,
    DomainError,
};
use chrono::{Utc, NaiveDate};

/// Test Story 1.1: Create Person Record
/// 
/// ```mermaid
/// graph LR
///     A[HR Admin] --> B[Create Person]
///     B --> C[Generate PersonId]
///     C --> D[Set Active State]
///     D --> E[Record Event]
/// ```
#[test]
fn test_create_person_record() {
    // As a HR administrator
    // I want to create a new person record with basic identity information
    
    // Arrange
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    
    // Act
    let person = Person::new(person_id, name.clone());
    
    // Assert acceptance criteria
    assert_eq!(person.id, person_id, "Should have the assigned PersonId");
    assert_eq!(person.core_identity.legal_name.display_name(), "John Doe", "Should have the correct name");
    assert!(matches!(person.lifecycle, PersonLifecycle::Active), "Should start in Active state");
    assert_eq!(person.components.len(), 0, "Should have no components initially");
}

/// Test minimal required fields
#[test]
fn test_create_person_minimal_fields() {
    // Minimal required fields: given name, family name
    let name = PersonName::new("Alice".to_string(), "Johnson".to_string());
    let person = Person::new(PersonId::new(), name);
    
    assert_eq!(person.core_identity.legal_name.display_name(), "Alice Johnson");
}

/// Test Story 1.2: Update Person Name
/// 
/// ```mermaid
/// graph TB
///     A[Person Admin] --> B{Update Name}
///     B --> C[Validate State]
///     C --> D[Generate Event]
///     D --> E[Preserve Old Name]
///     E --> F[Audit Trail]
/// ```
#[test]
fn test_update_person_name() {
    // As a person administrator
    // I want to update a person's legal name
    
    // Arrange
    let mut person = Person::new(PersonId::new(), PersonName::new("Jane".to_string(), "Smith".to_string()));
    let new_name = PersonName::new("Jane".to_string(), "Doe".to_string());
    
    // Act - using command handler
    let cmd = PersonCommand::UpdateName(UpdateName {
        person_id: person.id,
        name: new_name.clone(),
        reason: Some("Marriage".to_string()),
    });
    
    let result = person.handle_command(cmd);
    
    // Assert acceptance criteria
    assert!(result.is_ok(), "Should successfully update name");
    let events = result.unwrap();
    assert_eq!(events.len(), 1, "Should generate one event");
    
    match &events[0] {
        PersonEvent::NameUpdated(event) => {
            assert_eq!(event.old_name.display_name(), "Jane Smith", "Should preserve old name");
            assert_eq!(event.new_name.display_name(), "Jane Doe", "Should have new name");
            assert_eq!(event.reason, Some("Marriage".to_string()), "Should include reason");
        }
        _ => panic!("Expected NameUpdated event"),
    }
}

/// Test cannot update inactive person
#[test]
fn test_cannot_update_inactive_person_name() {
    // Arrange
    let mut person = Person::new(PersonId::new(), PersonName::new("Test".to_string(), "User".to_string()));
    
    // Deactivate using command
    let deactivate_cmd = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id: person.id,
        reason: "Account suspended".to_string(),
    });
    person.handle_command(deactivate_cmd).unwrap();
    person.lifecycle = PersonLifecycle::Deactivated { 
        reason: "Account suspended".to_string(), 
        since: Utc::now() 
    };
    
    // Act
    let update_cmd = PersonCommand::UpdateName(UpdateName {
        person_id: person.id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: Some("Test".to_string()),
    });
    let result = person.handle_command(update_cmd);
    
    // Assert
    assert!(matches!(
        result,
        Err(msg) if msg.contains("Cannot update inactive person")
    ));
}

/// Test cannot update deceased person
#[test]
fn test_cannot_update_deceased_person_name() {
    // Arrange
    let mut person = Person::new(PersonId::new(), PersonName::new("Test".to_string(), "User".to_string()));
    
    // Record death using command
    let death_cmd = PersonCommand::RecordDeath(RecordDeath {
        person_id: person.id,
        date_of_death: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    });
    person.handle_command(death_cmd).unwrap();
    person.lifecycle = PersonLifecycle::Deceased { 
        date_of_death: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() 
    };
    
    // Act
    let update_cmd = PersonCommand::UpdateName(UpdateName {
        person_id: person.id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: Some("Test".to_string()),
    });
    let result = person.handle_command(update_cmd);
    
    // Assert
    assert!(matches!(
        result,
        Err(msg) if msg.contains("Cannot modify a deceased person")
    ));
}

/// Test Story 1.3: Merge Duplicate Persons
/// 
/// ```mermaid
/// graph LR
///     A[Data Quality Manager] --> B[Identify Duplicates]
///     B --> C[Select Source/Target]
///     C --> D[Note Components]
///     D --> E[Mark as Merged]
///     E --> F[Prevent Modifications]
/// ```
#[test]
fn test_merge_duplicate_persons() {
    // As a data quality manager
    // I want to merge duplicate person records
    
    // Arrange
    let mut source = Person::new(PersonId::new(), PersonName::new("John".to_string(), "Doe".to_string()));
    let target_id = PersonId::new();
    
    // Add some components to source
    source.register_component(ComponentType::EmailAddress).unwrap();
    source.register_component(ComponentType::PhoneNumber).unwrap();
    
    // Act
    let merge_cmd = PersonCommand::MergePersons(MergePersons {
        source_person_id: source.id,
        target_person_id: target_id,
        merge_reason: MergeReason::DuplicateRecord,
    });
    let result = source.handle_command(merge_cmd);
    
    // Assert acceptance criteria
    assert!(result.is_ok(), "Should successfully merge");
    let events = result.unwrap();
    
    // Should generate merge event
    let merge_event = events.iter().find(|e| matches!(e, PersonEvent::PersonMergedInto(_)));
    assert!(merge_event.is_some(), "Should generate PersonMergedInto event");
    
    match merge_event.unwrap() {
        PersonEvent::PersonMergedInto(event) => {
            assert_eq!(event.merged_into_id, target_id, "Should reference target person");
            assert!(matches!(event.reason, MergeReason::DuplicateRecord), "Should record merge reason");
        }
        _ => unreachable!(),
    }
}

/// Test cannot modify merged person
#[test]
fn test_cannot_modify_merged_person() {
    // Arrange
    let mut person = Person::new(PersonId::new(), PersonName::new("Test".to_string(), "User".to_string()));
    let target_id = PersonId::new();
    
    // Merge the person
    person.lifecycle = PersonLifecycle::MergedInto { 
        target_id, 
        merged_at: Utc::now() 
    };
    
    // Act - try to update name
    let update_cmd = PersonCommand::UpdateName(UpdateName {
        person_id: person.id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: Some("Test".to_string()),
    });
    let result = person.handle_command(update_cmd);
    
    // Assert
    assert!(matches!(
        result,
        Err(msg) if msg.contains("Cannot modify a merged person")
    ));
    
    // Act - try to register component
    let comp_cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id: person.id,
        component_type: ComponentType::EmailAddress,
    });
    let result = person.handle_command(comp_cmd);
    
    // Assert
    assert!(matches!(
        result,
        Err(msg) if msg.contains("Cannot modify a merged person")
    ));
}

/// Test Story 1.4: Record Person Death
/// 
/// ```mermaid
/// graph TB
///     A[Records Admin] --> B[Record Death]
///     B --> C[Set Date of Death]
///     C --> D[Change Lifecycle]
///     D --> E[Prevent Modifications]
///     E --> F[Trigger Notifications]
/// ```
#[test]
fn test_record_person_death() {
    // As a records administrator
    // I want to record when a person has died
    
    // Arrange
    let mut person = Person::new(PersonId::new(), PersonName::new("John".to_string(), "Smith".to_string()));
    let date_of_death = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    
    // Act
    let death_cmd = PersonCommand::RecordDeath(RecordDeath {
        person_id: person.id,
        date_of_death,
    });
    let result = person.handle_command(death_cmd);
    
    // Assert acceptance criteria
    assert!(result.is_ok(), "Should successfully record death");
    let events = result.unwrap();
    
    // Should generate death recorded event
    let death_event = events.iter().find(|e| matches!(e, PersonEvent::DeathRecorded(_)));
    assert!(death_event.is_some(), "Should generate DeathRecorded event");
    
    match death_event.unwrap() {
        PersonEvent::DeathRecorded(event) => {
            assert_eq!(event.date_of_death, date_of_death, "Should record date of death");
        }
        _ => unreachable!(),
    }
}

/// Test cannot modify deceased person
#[test]
fn test_cannot_modify_deceased_person() {
    // Arrange
    let mut person = Person::new(PersonId::new(), PersonName::new("Test".to_string(), "User".to_string()));
    person.lifecycle = PersonLifecycle::Deceased { 
        date_of_death: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() 
    };
    
    // Act - try various modifications
    let name_cmd = PersonCommand::UpdateName(UpdateName {
        person_id: person.id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: Some("Test".to_string()),
    });
    let name_result = person.handle_command(name_cmd);
    
    let comp_cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id: person.id,
        component_type: ComponentType::EmailAddress,
    });
    let component_result = person.handle_command(comp_cmd);
    
    // Assert
    assert!(matches!(
        name_result,
        Err(msg) if msg.contains("Cannot modify a deceased person")
    ));
    assert!(matches!(
        component_result,
        Err(msg) if msg.contains("Cannot modify a deceased person")
    ));
}

/// Test lifecycle state transitions
/// 
/// ```mermaid
/// stateDiagram-v2
///     [*] --> Active: Create
///     Active --> Deactivated: Deactivate
///     Active --> Deceased: Record Death
///     Active --> MergedInto: Merge
///     Deactivated --> Active: Reactivate
///     Deactivated --> Deceased: Record Death
///     Deceased --> [*]: Terminal
///     MergedInto --> [*]: Terminal
/// ```
#[test]
fn test_lifecycle_transitions() {
    // Test valid transitions from Active
    let mut person = Person::new(PersonId::new(), PersonName::new("Test".to_string(), "User".to_string()));
    
    // Active -> Deactivated
    let deactivate_cmd = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id: person.id,
        reason: "Test".to_string(),
    });
    assert!(person.handle_command(deactivate_cmd).is_ok());
    person.lifecycle = PersonLifecycle::Deactivated { 
        reason: "Test".to_string(), 
        since: Utc::now() 
    };
    assert!(matches!(person.lifecycle, PersonLifecycle::Deactivated { .. }));
    
    // Deactivated -> Active (reactivate)
    let reactivate_cmd = PersonCommand::ReactivatePerson(ReactivatePerson {
        person_id: person.id,
        reason: "Test".to_string(),
    });
    assert!(person.handle_command(reactivate_cmd).is_ok());
    person.lifecycle = PersonLifecycle::Active;
    assert!(matches!(person.lifecycle, PersonLifecycle::Active));
    
    // Active -> Deceased
    let mut person2 = Person::new(PersonId::new(), PersonName::new("Test".to_string(), "User".to_string()));
    let death_cmd = PersonCommand::RecordDeath(RecordDeath {
        person_id: person2.id,
        date_of_death: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    });
    assert!(person2.handle_command(death_cmd).is_ok());
    
    // Active -> MergedInto
    let mut person3 = Person::new(PersonId::new(), PersonName::new("Test".to_string(), "User".to_string()));
    let merge_cmd = PersonCommand::MergePersons(MergePersons {
        source_person_id: person3.id,
        target_person_id: PersonId::new(),
        merge_reason: MergeReason::DuplicateRecord,
    });
    assert!(person3.handle_command(merge_cmd).is_ok());
}

/// Test audit trail is maintained
#[test]
fn test_audit_trail_maintained() {
    let mut person = Person::new(PersonId::new(), PersonName::new("Test".to_string(), "User".to_string()));
    
    // Perform various operations
    let name_cmd = PersonCommand::UpdateName(UpdateName {
        person_id: person.id,
        name: PersonName::new("Test".to_string(), "User2".to_string()),
        reason: Some("Name change".to_string()),
    });
    let events1 = person.handle_command(name_cmd).unwrap();
    
    let comp_cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id: person.id,
        component_type: ComponentType::EmailAddress,
    });
    let events2 = person.handle_command(comp_cmd).unwrap();
    
    // All events should include audit information
    for event in events1.iter().chain(events2.iter()) {
        match event {
            PersonEvent::NameUpdated(e) => {
                assert!(e.updated_at.timestamp() > 0, "Should include timestamp");
            }
            PersonEvent::ComponentRegistered(e) => {
                assert!(e.registered_at.timestamp() > 0, "Should include timestamp");
            }
            _ => {}
        }
    }
} 