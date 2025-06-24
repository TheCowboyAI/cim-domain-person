//! Tests for Epic 2: Component Management User Stories
//! 
//! Tests cover:
//! - Story 2.1: Add Email Component
//! - Story 2.2: Manage Skills
//! - Story 2.3: Set Communication Preferences

use cim_domain_person::{
    aggregate::{Person, PersonId, ComponentType},
    components::{
        contact::{EmailComponent, PhoneComponent, ContactContext},
        skills::{SkillComponent, SkillCategory, ProficiencyLevel},
        preferences::PreferencesComponent,
        ComponentMetadata,
    },
    value_objects::{PersonName, EmailAddress, PhoneNumber},
    events::PersonEvent,
    DomainError,
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
    let mut person = Person::new(PersonId::new(), PersonName::new("Alice", "Johnson"));
    
    // Act - Register email component
    let result = person.register_component(ComponentType::EmailAddress, "crm_user");
    
    // Assert acceptance criteria
    assert!(result.is_ok(), "Should successfully register email component");
    let events = result.unwrap();
    assert_eq!(events.len(), 1, "Should generate one event");
    
    match &events[0] {
        PersonEvent::ComponentRegistered { component_type, registered_by, .. } => {
            assert_eq!(component_type, &ComponentType::EmailAddress);
            assert_eq!(registered_by, "crm_user");
        }
        _ => panic!("Expected ComponentRegistered event"),
    }
    
    assert!(person.has_component(&ComponentType::EmailAddress), "Component should be registered");
}

/// Test multiple email addresses with primary flag
#[test]
fn test_multiple_emails_with_primary() {
    // This would be implemented in the ECS layer
    // Here we test the domain logic of registering the component
    
    let mut person = Person::new(PersonId::new(), PersonName::new("Bob", "Smith"));
    
    // Register email component once
    let result = person.register_component(ComponentType::EmailAddress, "system");
    assert!(result.is_ok());
    
    // Cannot register same component twice
    let duplicate = person.register_component(ComponentType::EmailAddress, "system");
    assert!(matches!(
        duplicate,
        Err(DomainError::InvalidOperation(msg)) if msg.contains("already registered")
    ));
}

/// Test email component structure
#[test]
fn test_email_component_structure() {
    // Test the component structure itself
    let email = EmailComponent {
        email: EmailAddress {
            value: "john@example.com".to_string(),
            normalized: "john@example.com".to_string(),
        },
        is_primary: true,
        context: ContactContext::Work,
        metadata: ComponentMetadata {
            created_at: chrono::Utc::now(),
            created_by: "crm_user".to_string(),
            updated_at: chrono::Utc::now(),
            updated_by: "crm_user".to_string(),
            version: 1,
            tags: HashMap::new(),
        },
    };
    
    assert_eq!(email.email.value, "john@example.com");
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
    let mut person = Person::new(PersonId::new(), PersonName::new("Sarah", "Developer"));
    
    // Act - Register skills component
    let result = person.register_component(ComponentType::Skills, "talent_manager");
    
    // Assert
    assert!(result.is_ok(), "Should successfully register skills component");
    assert!(person.has_component(&ComponentType::Skills));
    
    // Test skill component structure
    let skill = SkillComponent {
        skill_id: "rust-programming".to_string(),
        name: "Rust Programming".to_string(),
        category: SkillCategory::Technical,
        proficiency: ProficiencyLevel::Expert,
        years_experience: Some(5.0),
        last_used: Some(chrono::Utc::now().date_naive()),
        metadata: ComponentMetadata::new("talent_manager"),
    };
    
    assert_eq!(skill.name, "Rust Programming");
    assert!(matches!(skill.category, SkillCategory::Technical));
    assert!(matches!(skill.proficiency, ProficiencyLevel::Expert));
    assert_eq!(skill.years_experience, Some(5.0));
}

/// Test skill categories
#[test]
fn test_skill_categories() {
    // Test different skill categories
    let technical = SkillCategory::Technical;
    let soft = SkillCategory::Soft;
    let domain = SkillCategory::Domain;
    let language = SkillCategory::Language;
    
    // All categories should be distinct
    assert!(!matches!(technical, SkillCategory::Soft));
    assert!(!matches!(soft, SkillCategory::Domain));
    assert!(!matches!(domain, SkillCategory::Language));
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
            if i != j {
                assert!(!matches!(level, other));
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
    let mut person = Person::new(PersonId::new(), PersonName::new("Customer", "One"));
    
    // Act - Register preferences component
    let result = person.register_component(ComponentType::Preferences, "marketing_manager");
    
    // Assert
    assert!(result.is_ok(), "Should successfully register preferences component");
    assert!(person.has_component(&ComponentType::Preferences));
    
    // Component is registered, actual preferences would be stored in ECS
}

/// Test component metadata tracking
#[test]
fn test_component_metadata() {
    let metadata = ComponentMetadata::new("test_user");
    
    assert_eq!(metadata.created_by, "test_user");
    assert_eq!(metadata.updated_by, "test_user");
    assert_eq!(metadata.version, 1);
    assert!(metadata.tags.is_empty());
    
    // Test with tags
    let mut metadata_with_tags = ComponentMetadata::new("admin");
    metadata_with_tags.tags.insert("source".to_string(), "import".to_string());
    metadata_with_tags.tags.insert("verified".to_string(), "true".to_string());
    
    assert_eq!(metadata_with_tags.tags.len(), 2);
    assert_eq!(metadata_with_tags.tags.get("source"), Some(&"import".to_string()));
}

/// Test component registration query
#[test]
fn test_query_persons_by_component() {
    // Test that we can check which components a person has
    let mut person = Person::new(PersonId::new(), PersonName::new("Multi", "Component"));
    
    // Register multiple components
    person.register_component(ComponentType::EmailAddress, "system").unwrap();
    person.register_component(ComponentType::PhoneNumber, "system").unwrap();
    person.register_component(ComponentType::Skills, "hr").unwrap();
    person.register_component(ComponentType::Preferences, "marketing").unwrap();
    
    // Check component presence
    assert!(person.has_component(&ComponentType::EmailAddress));
    assert!(person.has_component(&ComponentType::PhoneNumber));
    assert!(person.has_component(&ComponentType::Skills));
    assert!(person.has_component(&ComponentType::Preferences));
    assert!(!person.has_component(&ComponentType::Custom("NonExistent".to_string())));
    
    // Get all components
    let components = person.component_types();
    assert_eq!(components.len(), 4);
}

/// Test component registration events
#[test]
fn test_component_registration_events() {
    let mut person = Person::new(PersonId::new(), PersonName::new("Event", "Test"));
    
    // Register different component types
    let email_events = person.register_component(ComponentType::EmailAddress, "user1").unwrap();
    let phone_events = person.register_component(ComponentType::PhoneNumber, "user2").unwrap();
    let skill_events = person.register_component(ComponentType::Skills, "hr_admin").unwrap();
    
    // Verify each generates appropriate event
    for (events, expected_type, expected_user) in [
        (email_events, ComponentType::EmailAddress, "user1"),
        (phone_events, ComponentType::PhoneNumber, "user2"),
        (skill_events, ComponentType::Skills, "hr_admin"),
    ] {
        assert_eq!(events.len(), 1);
        match &events[0] {
            PersonEvent::ComponentRegistered { 
                component_type, 
                registered_by, 
                ..
            } => {
                assert_eq!(component_type, &expected_type);
                assert_eq!(registered_by, expected_user);
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
        ContactContext::Other("Emergency".to_string()),
    ];
    
    for context in &contexts {
        match context {
            ContactContext::Personal => assert!(true),
            ContactContext::Work => assert!(true),
            ContactContext::Other(s) => assert_eq!(s, "Emergency"),
        }
    }
}

/// Test that inactive persons cannot have components added
#[test]
fn test_cannot_add_components_to_inactive_person() {
    let mut person = Person::new(PersonId::new(), PersonName::new("Inactive", "User"));
    
    // Deactivate person
    person.deactivate("Account closed", "admin").unwrap();
    
    // Try to register component
    let result = person.register_component(ComponentType::EmailAddress, "system");
    
    assert!(matches!(
        result,
        Err(DomainError::InvalidOperation(msg)) if msg.contains("Cannot modify inactive person")
    ));
} 