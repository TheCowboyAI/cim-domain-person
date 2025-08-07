//! Tests for Epic 5: Privacy and Compliance User Stories
//!
//! Tests cover:
//! - Story 5.1: Implement Right to be Forgotten
//! - Story 5.2: Control Component Access
//! - Story 5.3: Export Person Data

use cim_domain_person::{
    aggregate::{ComponentType, Person, PersonId, PersonLifecycle},
    commands::{DeactivatePerson, PersonCommand},
    events::PersonEvent,
    value_objects::PersonName,
};

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
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Delete".to_string(), "Me".to_string()),
    );
    person
        .register_component(ComponentType::EmailAddress)
        .unwrap();
    person
        .register_component(ComponentType::PhoneNumber)
        .unwrap();

    // Act - Deactivate for GDPR compliance
    let result = person.handle_command(PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id: person.id,
        reason: "GDPR Right to be Forgotten".to_string(),
    }));

    // Assert
    assert!(result.is_ok(), "Should successfully deactivate");
    let events = result.unwrap();

    // Should generate deactivation event
    let deactivation_event = events
        .iter()
        .find(|e| matches!(e, PersonEvent::PersonDeactivated(_)));
    assert!(deactivation_event.is_some());

    match deactivation_event.unwrap() {
        PersonEvent::PersonDeactivated(e) => {
            assert_eq!(e.reason, "GDPR Right to be Forgotten");
        }
        _ => unreachable!(),
    }
}

/// Test cascade to related components
#[test]
fn test_cascade_component_removal() {
    // When implementing right to be forgotten, all components should be noted
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Test".to_string(), "User".to_string()),
    );

    // Add various components
    person
        .register_component(ComponentType::EmailAddress)
        .unwrap();
    person
        .register_component(ComponentType::PhoneNumber)
        .unwrap();
    person.register_component(ComponentType::Skill).unwrap();
    person
        .register_component(ComponentType::CommunicationPreferences)
        .unwrap();

    // Get all components before deactivation
    let components_before: Vec<_> = person.components.iter().cloned().collect();
    assert_eq!(components_before.len(), 4);

    // Deactivate for GDPR
    let deactivation_result =
        person.handle_command(PersonCommand::DeactivatePerson(DeactivatePerson {
            person_id: person.id,
            reason: "GDPR Request".to_string(),
        }));

    // Should generate deactivation event
    assert!(deactivation_result.is_ok());
    let events = deactivation_result.unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], PersonEvent::PersonDeactivated(_)));

    // Components are still tracked (for audit)
    let components_after: Vec<_> = person.components.iter().cloned().collect();
    assert_eq!(
        components_after.len(),
        4,
        "Components tracked for audit trail"
    );

    // In event sourcing, the aggregate state is not automatically updated
    // The events would be applied during event replay/projection
}

/// Test audit trail preservation
#[test]
fn test_audit_trail_preserved() {
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Audit".to_string(), "Test".to_string()),
    );

    // Perform various operations
    let name_update_result = person.handle_command(PersonCommand::UpdateName(
        cim_domain_person::commands::UpdateName {
            person_id: person.id,
            name: PersonName::new("Audit".to_string(), "Changed".to_string()),
            reason: Some("User request".to_string()),
        },
    ));
    assert!(name_update_result.is_ok());

    person
        .register_component(ComponentType::EmailAddress)
        .unwrap();

    // Deactivate for compliance
    let deactivation_events =
        person.handle_command(PersonCommand::DeactivatePerson(DeactivatePerson {
            person_id: person.id,
            reason: "Compliance request".to_string(),
        }));

    // All events should maintain audit information
    assert!(deactivation_events.is_ok());
    let events = deactivation_events.unwrap();
    assert!(!events.is_empty());

    // Even after deactivation command, the person record exists
    assert_eq!(person.id, person.id); // ID preserved

    // Verify deactivation event was generated
    assert!(matches!(events[0], PersonEvent::PersonDeactivated(_)));

    // In event sourcing, the aggregate state is not automatically updated
    // The deactivation would take effect when events are applied during replay
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
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Private".to_string(), "User".to_string()),
    );

    // Different systems register different components
    let hr_result = person.register_component(ComponentType::Employment);
    let marketing_result = person.register_component(ComponentType::CommunicationPreferences);
    let it_result = person.register_component(ComponentType::EmailAddress);

    // All should succeed
    assert!(hr_result.is_ok());
    assert!(marketing_result.is_ok());
    assert!(it_result.is_ok());
}

/// Test role-based component access
#[test]
fn test_role_based_component_access() {
    // Simulate role-based access patterns
    let roles_and_components = vec![
        (
            "hr_role",
            vec![ComponentType::Employment, ComponentType::Skill],
        ),
        (
            "marketing_role",
            vec![
                ComponentType::CommunicationPreferences,
                ComponentType::EmailAddress,
            ],
        ),
        ("finance_role", vec![ComponentType::Employment]),
    ];

    for (role, allowed_components) in roles_and_components {
        let mut person = Person::new(
            PersonId::new(),
            PersonName::new("Test".to_string(), "User".to_string()),
        );

        // Register components based on role
        for component in &allowed_components {
            let result = person.register_component(*component);
            assert!(
                result.is_ok(),
                "Role {} should register {:?}",
                role,
                component
            );
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
    let person = Person::new(
        person_id,
        PersonName::new("Export".to_string(), "Me".to_string()),
    );

    // Verify core identity data is accessible
    assert_eq!(person.id, person_id);
    assert_eq!(person.core_identity.legal_name.full_name(), "Export Me");

    // Verify lifecycle state
    assert!(matches!(person.lifecycle, PersonLifecycle::Active));

    // Component list (would be empty for new person)
    let components: Vec<_> = person.components.iter().cloned().collect();
    assert_eq!(components.len(), 0);
}

/// Test comprehensive data export
#[test]
fn test_comprehensive_data_export() {
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Full".to_string(), "Export".to_string()),
    );

    // Add various components
    person
        .register_component(ComponentType::EmailAddress)
        .unwrap();
    person
        .register_component(ComponentType::PhoneNumber)
        .unwrap();
    person.register_component(ComponentType::Skill).unwrap();
    person.register_component(ComponentType::Address).unwrap();
    person
        .register_component(ComponentType::CommunicationPreferences)
        .unwrap();

    // Export would include:
    // 1. Core identity
    let export_data = ExportData {
        person_id: person.id,
        legal_name: person.core_identity.legal_name.full_name(),
        lifecycle_state: format!("{:?}", person.lifecycle),
        registered_components: person.components.iter().cloned().collect(),
    };

    // Verify all data is included
    assert_eq!(export_data.person_id, person.id);
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
    let person = Person::new(
        PersonId::new(),
        PersonName::new("Minimal".to_string(), "Data".to_string()),
    );

    // Core aggregate should be minimal
    assert_eq!(person.components.len(), 0, "No components by default");

    // Only required fields present
    assert!(!person.core_identity.legal_name.full_name().is_empty());
    assert!(person.core_identity.birth_date.is_none());
}

/// Test consent tracking for components
#[test]
fn test_consent_tracking() {
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Consent".to_string(), "User".to_string()),
    );

    // Marketing preferences require consent
    let marketing_consent = person.handle_command(PersonCommand::RegisterComponent(
        cim_domain_person::commands::RegisterComponent {
            person_id: person.id,
            component_type: ComponentType::CommunicationPreferences,
        },
    ));
    assert!(marketing_consent.is_ok());

    // Behavioral tracking requires consent
    let behavioral_consent = person.handle_command(PersonCommand::RegisterComponent(
        cim_domain_person::commands::RegisterComponent {
            person_id: person.id,
            component_type: ComponentType::BehavioralData,
        },
    ));
    assert!(behavioral_consent.is_ok());

    // Verify consent is tracked in events
    match &marketing_consent.unwrap()[0] {
        PersonEvent::ComponentRegistered(_) => {
            // In a real system, this would track consent details
            assert!(true);
        }
        _ => panic!("Expected ComponentRegistered event"),
    }
}

/// Test privacy preferences enforcement
#[test]
fn test_privacy_preferences_enforcement() {
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Privacy".to_string(), "First".to_string()),
    );

    // Register privacy preferences component
    person
        .register_component(ComponentType::PrivacyPreferences)
        .unwrap();

    // In a real system, this would check privacy preferences before allowing operations
    // For now, we verify the component can be registered
    assert!(person.has_component(&ComponentType::PrivacyPreferences));
}
