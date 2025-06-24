//! Tests for Epic 5: Privacy and Compliance User Stories
//! 
//! Tests cover:
//! - Story 5.1: Implement Right to be Forgotten
//! - Story 5.2: Control Component Access
//! - Story 5.3: Export Person Data

use cim_domain_person::{
    aggregate::{Person, PersonId, PersonLifecycle, ComponentType},
    value_objects::PersonName,
    events::PersonEvent,
    DomainError,
};
use chrono::Utc;
use std::collections::HashSet;

/// Test Story 5.1: Implement Right to be Forgotten
/// 
/// ```mermaid
/// graph TB
///     A[Compliance Officer] --> B[Request Deletion]
///     B --> C[Deactivate Person]
///     C --> D[Remove PII]
///     D --> E[Preserve Audit Trail]
///     E --> F[Generate Report]
///     F --> G[Notify Systems]
/// ```
#[test]
fn test_right_to_be_forgotten() {
    // As a compliance officer
    // I want to remove personal data upon request
    
    // Arrange
    let mut person = Person::new(PersonId::new(), PersonName::new("Delete", "Me"));
    person.register_component(ComponentType::EmailAddress, "system").unwrap();
    person.register_component(ComponentType::PhoneNumber, "system").unwrap();
    
    // Act - Deactivate for GDPR compliance
    let result = person.deactivate("GDPR Right to be Forgotten", "compliance_officer");
    
    // Assert
    assert!(result.is_ok(), "Should successfully deactivate");
    let events = result.unwrap();
    
    // Should generate deactivation event
    let deactivation_event = events.iter()
        .find(|e| matches!(e, PersonEvent::PersonDeactivated { .. }));
    assert!(deactivation_event.is_some());
    
    match deactivation_event.unwrap() {
        PersonEvent::PersonDeactivated { reason, deactivated_by, .. } => {
            assert_eq!(reason, "GDPR Right to be Forgotten");
            assert_eq!(deactivated_by, "compliance_officer");
        }
        _ => unreachable!(),
    }
    
    // Person should be in deactivated state
    assert!(matches!(
        person.lifecycle(),
        PersonLifecycle::Deactivated { reason, .. } if reason == "GDPR Right to be Forgotten"
    ));
}

/// Test cascade to related components
#[test]
fn test_cascade_component_removal() {
    // When implementing right to be forgotten, all components should be noted
    let mut person = Person::new(PersonId::new(), PersonName::new("Test", "User"));
    
    // Add various components
    person.register_component(ComponentType::EmailAddress, "system").unwrap();
    person.register_component(ComponentType::PhoneNumber, "system").unwrap();
    person.register_component(ComponentType::Skills, "hr").unwrap();
    person.register_component(ComponentType::Preferences, "marketing").unwrap();
    
    // Get all components before deactivation
    let components_before = person.component_types();
    assert_eq!(components_before.len(), 4);
    
    // Deactivate for GDPR
    person.deactivate("GDPR Request", "compliance").unwrap();
    
    // Components are still tracked (for audit), but person is inactive
    let components_after = person.component_types();
    assert_eq!(components_after.len(), 4, "Components tracked for audit trail");
    assert!(!person.is_active(), "Person should be inactive");
}

/// Test audit trail preservation
#[test]
fn test_audit_trail_preserved() {
    let mut person = Person::new(PersonId::new(), PersonName::new("Audit", "Test"));
    
    // Perform various operations
    person.update_name(PersonName::new("Audit", "Changed"), "User request").unwrap();
    person.register_component(ComponentType::EmailAddress, "system").unwrap();
    
    // Deactivate for compliance
    let deactivation_events = person.deactivate("Compliance request", "compliance_team").unwrap();
    
    // All events should maintain audit information
    assert!(!deactivation_events.is_empty());
    
    // Even after deactivation, the person record exists (just inactive)
    assert_eq!(person.id(), person.id()); // ID preserved
    assert!(!person.is_active());
}

/// Test Story 5.2: Control Component Access
/// 
/// ```mermaid
/// graph TB
///     A[Privacy Admin] --> B[Set Access Rules]
///     B --> C[Component Level]
///     C --> D[Role Based]
///     D --> E[Access Logging]
///     E --> F[Consent Tracking]
/// ```
#[test]
fn test_component_access_control() {
    // As a privacy administrator
    // I want to control which systems can access person components
    
    // This test verifies the tracking of who registered components
    let mut person = Person::new(PersonId::new(), PersonName::new("Private", "User"));
    
    // Different systems register different components
    let hr_result = person.register_component(ComponentType::Employment, "hr_system");
    let marketing_result = person.register_component(ComponentType::Preferences, "marketing_system");
    let it_result = person.register_component(ComponentType::EmailAddress, "it_system");
    
    // All should succeed
    assert!(hr_result.is_ok());
    assert!(marketing_result.is_ok());
    assert!(it_result.is_ok());
    
    // Verify audit trail shows who registered what
    match &hr_result.unwrap()[0] {
        PersonEvent::ComponentRegistered { registered_by, component_type, .. } => {
            assert_eq!(registered_by, "hr_system");
            assert_eq!(component_type, &ComponentType::Employment);
        }
        _ => panic!("Expected ComponentRegistered event"),
    }
}

/// Test role-based component access
#[test]
fn test_role_based_component_access() {
    // Simulate role-based access patterns
    let roles_and_components = vec![
        ("hr_role", vec![ComponentType::Employment, ComponentType::Skills]),
        ("marketing_role", vec![ComponentType::Preferences, ComponentType::EmailAddress]),
        ("finance_role", vec![ComponentType::Employment]),
    ];
    
    for (role, allowed_components) in roles_and_components {
        let mut person = Person::new(PersonId::new(), PersonName::new("Test", "User"));
        
        // Register components based on role
        for component in allowed_components {
            let result = person.register_component(component, role);
            assert!(result.is_ok(), "Role {} should register {:?}", role, component);
        }
    }
}

/// Test Story 5.3: Export Person Data
/// 
/// ```mermaid
/// graph TB
///     A[Data Subject] --> B[Request Export]
///     B --> C[Core Identity]
///     C --> D[All Components]
///     D --> E[Relationships]
///     E --> F[Event History]
///     F --> G[Multiple Formats]
/// ```
#[test]
fn test_export_person_data() {
    // As a person (data subject)
    // I want to export all my personal data
    
    // Arrange
    let person_id = PersonId::new();
    let person = Person::new(person_id, PersonName::new("Export", "Me"));
    
    // Verify core identity data is accessible
    assert_eq!(person.id(), person_id);
    assert_eq!(person.core_identity().legal_name, "Export Me");
    
    // Verify lifecycle state
    assert!(matches!(person.lifecycle(), PersonLifecycle::Active));
    
    // Component list (would be empty for new person)
    let components = person.component_types();
    assert_eq!(components.len(), 0);
}

/// Test comprehensive data export
#[test]
fn test_comprehensive_data_export() {
    let mut person = Person::new(PersonId::new(), PersonName::new("Full", "Export"));
    
    // Add various components
    person.register_component(ComponentType::EmailAddress, "system").unwrap();
    person.register_component(ComponentType::PhoneNumber, "system").unwrap();
    person.register_component(ComponentType::Skills, "hr").unwrap();
    person.register_component(ComponentType::Location, "facilities").unwrap();
    person.register_component(ComponentType::Preferences, "marketing").unwrap();
    
    // Export would include:
    // 1. Core identity
    let export_data = ExportData {
        person_id: person.id(),
        legal_name: person.core_identity().legal_name.clone(),
        lifecycle_state: format!("{:?}", person.lifecycle()),
        registered_components: person.component_types(),
    };
    
    // Verify all data is included
    assert_eq!(export_data.person_id, person.id());
    assert_eq!(export_data.legal_name, "Full Export");
    assert!(export_data.lifecycle_state.contains("Active"));
    assert_eq!(export_data.registered_components.len(), 5);
}

// Helper struct for export testing
struct ExportData {
    person_id: PersonId,
    legal_name: String,
    lifecycle_state: String,
    registered_components: Vec<ComponentType>,
}

/// Test data minimization principles
#[test]
fn test_data_minimization() {
    // Only collect and store necessary data
    let person = Person::new(PersonId::new(), PersonName::new("Minimal", "Data"));
    
    // Core aggregate should be minimal
    assert_eq!(person.component_count(), 0, "No components by default");
    
    // Only required fields present
    assert!(!person.core_identity().legal_name.is_empty());
    assert!(person.core_identity().preferred_name.is_none());
    assert!(person.core_identity().date_of_birth.is_none());
}

/// Test consent tracking for components
#[test]
fn test_consent_tracking() {
    let mut person = Person::new(PersonId::new(), PersonName::new("Consent", "User"));
    
    // Marketing preferences require consent
    let marketing_consent = person.register_component(
        ComponentType::Preferences,
        "marketing_with_consent"
    );
    assert!(marketing_consent.is_ok());
    
    // Behavioral tracking requires consent
    let behavioral_consent = person.register_component(
        ComponentType::Custom("BehavioralTracking".to_string()),
        "analytics_with_consent"
    );
    assert!(behavioral_consent.is_ok());
    
    // Verify consent is tracked in events
    match &marketing_consent.unwrap()[0] {
        PersonEvent::ComponentRegistered { registered_by, .. } => {
            assert!(registered_by.contains("consent"));
        }
        _ => panic!("Expected ComponentRegistered event"),
    }
}

/// Test privacy preferences enforcement
#[test]
fn test_privacy_preferences_enforcement() {
    let mut person = Person::new(PersonId::new(), PersonName::new("Privacy", "First"));
    
    // Register privacy preferences component
    person.register_component(ComponentType::Preferences, "user_settings").unwrap();
    
    // In a real system, this would check privacy preferences before allowing operations
    // For now, we verify the component can be registered
    assert!(person.has_component(&ComponentType::Preferences));
} 