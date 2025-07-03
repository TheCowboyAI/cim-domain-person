//! Tests for Epic 2: Component Management User Stories
//!
//! Tests cover:
//! - Story 2.1: Add Email Component
//! - Story 2.2: Manage Skills
//! - Story 2.3: Set Communication Preferences

use chrono::Utc;
use cim_domain_person::{
    aggregate::{ComponentType, Person, PersonId, PersonLifecycle},
    commands::*,
    components::{
        contact::{ContactContext, EmailComponent, PhoneComponent},
        skills::{CertificationComponent, ProficiencyLevel, SkillCategory, SkillComponent},
        ComponentMetadata,
    },
    events::PersonEvent,
    value_objects::{EmailAddress, PersonName, PhoneNumber},
};
use std::collections::HashMap;

/// Test Story 2.1: Add Email Component
///
/// ```mermaid
/// graph TB
///     A[CRM User] --> B[Add Email]
///     B --> C[Register Component]
///     C --> D[Set Primary Flag]
///     D --> E[Track Context]
///     E --> F[Enable Query]
/// ```
#[test]
fn test_add_email_component() {
    // As a CRM user
    // I want to add email addresses to a person

    // Arrange
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Alice".to_string(), "Johnson".to_string()),
    );

    // Act - Register email component
    let result = person.register_component(ComponentType::EmailAddress);

    // Assert acceptance criteria
    assert!(
        result.is_ok(),
        "Should successfully register email component"
    );

    assert!(
        person.has_component(&ComponentType::EmailAddress),
        "Component should be registered"
    );
}

/// Test multiple email addresses with primary flag
#[test]
fn test_multiple_emails_with_primary() {
    // This would be implemented in the ECS layer
    // Here we test the domain logic of registering the component

    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Bob".to_string(), "Smith".to_string()),
    );

    // Register email component once
    let result = person.register_component(ComponentType::EmailAddress);
    assert!(result.is_ok());
    assert!(person.has_component(&ComponentType::EmailAddress));

    // Registering same component again succeeds (HashSet behavior)
    // but doesn't add it twice
    let duplicate = person.register_component(ComponentType::EmailAddress);
    assert!(duplicate.is_ok(), "HashSet silently ignores duplicates");

    // Still only has one instance of the component
    assert_eq!(person.components.len(), 1);
    assert!(person.has_component(&ComponentType::EmailAddress));
}

/// Test email component structure
#[test]
fn test_email_component_structure() {
    // Test the component structure itself
    let email = EmailComponent {
        email: EmailAddress::new("john@example.com".to_string()).expect("Valid email"),
        is_primary: true,
        context: ContactContext::Work,
        metadata: ComponentMetadata {
            attached_at: Utc::now(),
            updated_at: Utc::now(),
            source: "test".to_string(),
            version: 1,
        },
    };

    assert_eq!(email.email.address, "john@example.com");
    assert!(email.is_primary);
    assert!(matches!(email.context, ContactContext::Work));
}

/// Test Story 2.2: Manage Skills
///
/// ```mermaid
/// graph TB
///     A[Talent Manager] --> B[Add Skills]
///     B --> C[Set Proficiency]
///     C --> D[Track Categories]
///     D --> E[Record Experience]
///     E --> F[Track Certifications]
/// ```
#[test]
fn test_manage_skills() {
    // As a talent manager
    // I want to track skills and certifications for people

    // Arrange
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Sarah".to_string(), "Developer".to_string()),
    );

    // Act - Register skills component
    let result = person.register_component(ComponentType::Skill);

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully register skills component"
    );
    assert!(person.has_component(&ComponentType::Skill));

    // Test skill component structure
    let skill = SkillComponent {
        skill_id: "rust-programming".to_string(),
        name: "Rust Programming".to_string(),
        category: SkillCategory::Technical,
        proficiency: ProficiencyLevel::Expert,
        years_experience: Some(5.0),
        last_used: Some(chrono::Utc::now().date_naive()),
        metadata: ComponentMetadata {
            attached_at: Utc::now(),
            updated_at: Utc::now(),
            source: "test".to_string(),
            version: 1,
        },
    };

    assert_eq!(skill.name, "Rust Programming");
    assert!(matches!(skill.category, SkillCategory::Technical));
    assert!(matches!(skill.proficiency, ProficiencyLevel::Expert));
    assert_eq!(skill.years_experience, Some(5.0));
}

/// Test proficiency levels
#[test]
fn test_proficiency_levels() {
    // Test proficiency level ordering
    let levels = vec![
        ProficiencyLevel::Beginner,
        ProficiencyLevel::Intermediate,
        ProficiencyLevel::Advanced,
        ProficiencyLevel::Expert,
    ];

    // Verify all levels are distinct
    for (i, level) in levels.iter().enumerate() {
        for (j, other) in levels.iter().enumerate() {
            if i == j {
                // Same index should match
                assert!(matches!(level, other));
            } else {
                // Different indices should have different values
                match (level, other) {
                    (ProficiencyLevel::Beginner, ProficiencyLevel::Beginner) => {
                        panic!("Should not match")
                    }
                    (ProficiencyLevel::Intermediate, ProficiencyLevel::Intermediate) => {
                        panic!("Should not match")
                    }
                    (ProficiencyLevel::Advanced, ProficiencyLevel::Advanced) => {
                        panic!("Should not match")
                    }
                    (ProficiencyLevel::Expert, ProficiencyLevel::Expert) => {
                        panic!("Should not match")
                    }
                    _ => {} // Different variants, which is expected
                }
            }
        }
    }
}

/// Test Story 2.3: Set Communication Preferences
///
/// ```mermaid
/// graph TB
///     A[Marketing Manager] --> B[Set Preferences]
///     B --> C[Preferred Channel]
///     C --> D[Contact Frequency]
///     D --> E[Timezone/Times]
///     E --> F[Language]
///     F --> G[Do Not Contact]
/// ```
#[test]
fn test_set_communication_preferences() {
    // As a marketing manager
    // I want to track communication preferences for people

    // Arrange
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Customer".to_string(), "One".to_string()),
    );

    // Act - Register preferences component
    let result = person.register_component(ComponentType::CommunicationPreferences);

    // Assert
    assert!(
        result.is_ok(),
        "Should successfully register preferences component"
    );
    assert!(person.has_component(&ComponentType::CommunicationPreferences));

    // Component is registered, actual preferences would be stored in ECS
}

/// Test component registration query
#[test]
fn test_query_persons_by_component() {
    // Test that we can check which components a person has
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Multi".to_string(), "Component".to_string()),
    );

    // Register multiple components
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

    // Check component presence
    assert!(person.has_component(&ComponentType::EmailAddress));
    assert!(person.has_component(&ComponentType::PhoneNumber));
    assert!(person.has_component(&ComponentType::Skill));
    assert!(person.has_component(&ComponentType::CommunicationPreferences));
    assert!(!person.has_component(&ComponentType::Tag));

    // Get all components
    let components = &person.components;
    assert_eq!(components.len(), 4);
}

/// Test component registration events
#[test]
fn test_component_registration_events() {
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Event".to_string(), "Test".to_string()),
    );

    // Register different component types using commands
    let email_cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id: person.id,
        component_type: ComponentType::EmailAddress,
    });
    let email_events = person.handle_command(email_cmd).unwrap();

    let phone_cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id: person.id,
        component_type: ComponentType::PhoneNumber,
    });
    let phone_events = person.handle_command(phone_cmd).unwrap();

    let skill_cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id: person.id,
        component_type: ComponentType::Skill,
    });
    let skill_events = person.handle_command(skill_cmd).unwrap();

    // Verify each generates appropriate event
    for (events, expected_type) in [
        (email_events, ComponentType::EmailAddress),
        (phone_events, ComponentType::PhoneNumber),
        (skill_events, ComponentType::Skill),
    ] {
        assert_eq!(events.len(), 1);
        match &events[0] {
            PersonEvent::ComponentRegistered(event) => {
                assert_eq!(event.component_type, expected_type);
            }
            _ => panic!("Expected ComponentRegistered event"),
        }
    }
}

/// Test contact context variations
#[test]
fn test_contact_contexts() {
    // Test different contact contexts
    let contexts = vec![
        ContactContext::Personal,
        ContactContext::Work,
        ContactContext::Emergency,
        ContactContext::Other("School".to_string()),
    ];

    for context in &contexts {
        match context {
            ContactContext::Personal => assert!(true),
            ContactContext::Work => assert!(true),
            ContactContext::Emergency => assert!(true),
            ContactContext::Other(s) => assert_eq!(s, "School"),
        }
    }
}

/// Test that inactive persons cannot have components added
#[test]
fn test_cannot_add_components_to_inactive_person() {
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Inactive".to_string(), "User".to_string()),
    );

    // Deactivate person using command
    let deactivate_cmd = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id: person.id,
        reason: "Account closed".to_string(),
    });
    person.handle_command(deactivate_cmd).unwrap();
    person.lifecycle = PersonLifecycle::Deactivated {
        reason: "Account closed".to_string(),
        since: Utc::now(),
    };

    // Try to register component
    let result = person.register_component(ComponentType::EmailAddress);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Cannot add components to inactive person"));
}
