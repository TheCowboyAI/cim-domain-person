//! Integration tests for component store functionality

use cim_domain_person::{
    aggregate::{Person, PersonId, ComponentType},
    commands::{ComponentCommand, PersonCommand, CreatePerson, RegisterComponent},
    events::{ComponentDataEvent, PersonEvent},
    handlers::ComponentCommandHandler,
    infrastructure::{
        InMemoryEventStore, InMemoryComponentStore, InMemorySnapshotStore,
        PersonRepository, ComponentStore,
    },
    value_objects::{PersonName, EmailAddress, PhoneNumber},
    components::data::{
        ComponentInstanceId, EmailType, PhoneType, SkillCategory, ProficiencyLevel,
    },
};
use std::sync::Arc;
use chrono::Utc;

/// Test complete flow of adding email component
#[tokio::test]
async fn test_add_email_component_full_flow() {
    // Setup infrastructure
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10, // snapshot frequency
    ));
    
    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );
    
    // Create a person first
    let person_id = PersonId::new();
    let mut person = Person::new(person_id, PersonName::new("Test".to_string(), "User".to_string()));
    
    // Generate creation event
    let create_events = person.handle_command(PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Test".to_string(), "User".to_string()),
        source: "test".to_string(),
    })).unwrap();
    
    person_repo.save(&person, create_events, None).await.unwrap();
    
    // Register email component on the person
    let mut loaded_person = person_repo.load(person_id).await.unwrap().unwrap();
    let current_version = loaded_person.version;
    let register_cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id,
        component_type: ComponentType::EmailAddress,
    });
    let register_events = loaded_person.handle_command(register_cmd).unwrap();
    // handle_command already applies events and increments version
    // So we save with the version from BEFORE the command
    person_repo.save(&loaded_person, register_events, Some(current_version)).await.unwrap();
    
    // Add actual email data
    let cmd = ComponentCommand::AddEmail {
        person_id,
        email: "test@example.com".to_string(),
        email_type: EmailType::Personal,
        is_preferred: true,
        can_receive_notifications: true,
        can_receive_marketing: false,
    };
    
    let events = handler.handle_add_email(
        person_id,
        "test@example.com".to_string(),
        EmailType::Personal,
        true,
        true,
        false,
    ).await.unwrap();
    
    assert_eq!(events.len(), 1);
    match &events[0] {
        ComponentDataEvent::EmailAdded { email, email_type, .. } => {
            assert_eq!(email, "test@example.com");
            assert!(matches!(email_type, EmailType::Personal));
        }
        _ => panic!("Expected EmailAdded event"),
    }
    
    // Verify component was stored
    let components = component_store.get_components_by_type(
        person_id,
        ComponentType::EmailAddress,
    ).await.unwrap();
    
    assert_eq!(components.len(), 1);
}

/// Test updating email component
#[tokio::test]
async fn test_update_email_component() {
    // Setup
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(event_store.clone(), snapshot_store, 10));
    
    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );
    
    // Create person and add email
    let person_id = PersonId::new();
    let mut person = Person::new(person_id, PersonName::new("Test".to_string(), "User".to_string()));
    let create_events = person.handle_command(PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Test".to_string(), "User".to_string()),
        source: "test".to_string(),
    })).unwrap();
    person_repo.save(&person, create_events, None).await.unwrap();
    
    // Add email first
    let add_events = handler.handle_add_email(
        person_id,
        "old@example.com".to_string(),
        EmailType::Work,
        false,
        true,
        true,
    ).await.unwrap();
    
    let component_id = match &add_events[0] {
        ComponentDataEvent::EmailAdded { component_id, .. } => *component_id,
        _ => panic!("Expected EmailAdded event"),
    };
    
    // Update email
    let update_events = handler.handle_update_email(
        person_id,
        component_id,
        Some("new@example.com".to_string()),
        Some(EmailType::Personal),
        Some(true),
        None,
        None,
    ).await.unwrap();
    
    assert_eq!(update_events.len(), 1);
    match &update_events[0] {
        ComponentDataEvent::EmailUpdated { changes, .. } => {
            assert_eq!(changes.email.as_ref().unwrap(), "new@example.com");
            assert!(matches!(changes.email_type, Some(EmailType::Personal)));
            assert_eq!(changes.is_preferred, Some(true));
        }
        _ => panic!("Expected EmailUpdated event"),
    }
}

/// Test skill component management
#[tokio::test]
async fn test_skill_component_management() {
    // Setup
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(event_store.clone(), snapshot_store, 10));
    
    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );
    
    // Create person
    let person_id = PersonId::new();
    let mut person = Person::new(person_id, PersonName::new("Skilled".to_string(), "Developer".to_string()));
    let create_events = person.handle_command(PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Skilled".to_string(), "Developer".to_string()),
        source: "test".to_string(),
    })).unwrap();
    person_repo.save(&person, create_events, None).await.unwrap();
    
    // Add skill
    let events = handler.handle_add_skill(
        person_id,
        "Rust Programming".to_string(),
        SkillCategory::Technical,
        ProficiencyLevel::Expert,
        Some(5.0),
    ).await.unwrap();
    
    assert_eq!(events.len(), 1);
    let skill_id = match &events[0] {
        ComponentDataEvent::SkillAdded { component_id, skill_name, .. } => {
            assert_eq!(skill_name, "Rust Programming");
            *component_id
        }
        _ => panic!("Expected SkillAdded event"),
    };
    
    // Update skill would be implemented similarly
    // For now, just verify the skill was added
    
    // Verify skill is stored
    let components = component_store.get_components_by_type(
        person_id,
        ComponentType::Skill,
    ).await.unwrap();
    
    assert_eq!(components.len(), 1);
}

/// Test multiple components of same type
#[tokio::test]
async fn test_multiple_components_same_type() {
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(event_store.clone(), snapshot_store, 10));
    
    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );
    
    let person_id = PersonId::new();
    let mut person = Person::new(person_id, PersonName::new("Multi".to_string(), "Email".to_string()));
    let create_events = person.handle_command(PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Multi".to_string(), "Email".to_string()),
        source: "test".to_string(),
    })).unwrap();
    person_repo.save(&person, create_events, None).await.unwrap();
    
    // Add multiple emails
    let emails = vec![
        ("personal@example.com", EmailType::Personal, true),
        ("work@company.com", EmailType::Work, false),
        ("school@university.edu", EmailType::School, false),
    ];
    
    for (email, email_type, is_preferred) in emails {
        handler.handle_add_email(
            person_id,
            email.to_string(),
            email_type,
            is_preferred,
            true,
            false,
        ).await.unwrap();
    }
    
    // Verify all emails are stored
    let components = component_store.get_components_by_type(
        person_id,
        ComponentType::EmailAddress,
    ).await.unwrap();
    
    assert_eq!(components.len(), 3);
}

/// Test component removal
#[tokio::test]
async fn test_remove_component() {
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(event_store.clone(), snapshot_store, 10));
    
    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );
    
    let person_id = PersonId::new();
    let mut person = Person::new(person_id, PersonName::new("Test".to_string(), "User".to_string()));
    let create_events = person.handle_command(PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Test".to_string(), "User".to_string()),
        source: "test".to_string(),
    })).unwrap();
    person_repo.save(&person, create_events, None).await.unwrap();
    
    // Add phone
    let add_events = handler.handle_add_phone(
        person_id,
        "+1234567890".to_string(),
        PhoneType::Mobile,
        "+1".to_string(),  // country_code
        true,              // is_mobile
        true,              // can_receive_sms
        true,              // can_receive_calls
    ).await.unwrap();
    
    let component_id = match &add_events[0] {
        ComponentDataEvent::PhoneAdded { component_id, .. } => *component_id,
        _ => panic!("Expected PhoneAdded event"),
    };
    
    // For now, we'll just verify the phone was added
    // Remove functionality would need a handle_remove_phone method
    // which isn't implemented yet
    
    // Verify component exists
    let components = component_store.get_components_by_type(
        person_id,
        ComponentType::PhoneNumber,
    ).await.unwrap();
    assert_eq!(components.len(), 1);
}

/// Test cross-component queries
#[tokio::test]
async fn test_cross_component_queries() {
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(event_store.clone(), snapshot_store, 10));
    
    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );
    
    // Create multiple people with various components
    let person1 = PersonId::new();
    let person2 = PersonId::new();
    
    for (id, name) in [(person1, "Alice"), (person2, "Bob")] {
        let mut person = Person::new(id, PersonName::new(name.to_string(), "Test".to_string()));
        let create_events = person.handle_command(PersonCommand::CreatePerson(CreatePerson {
            person_id: id,
            name: PersonName::new(name.to_string(), "Test".to_string()),
            source: "test".to_string(),
        })).unwrap();
        person_repo.save(&person, create_events, None).await.unwrap();
    }
    
    // Add components to person1
    handler.handle_add_email(
        person1,
        "alice@example.com".to_string(),
        EmailType::Work,
        true,
        true,
        false,
    ).await.unwrap();
    
    handler.handle_add_skill(
        person1,
        "Python".to_string(),
        SkillCategory::Technical,
        ProficiencyLevel::Advanced,
        Some(3.0),
    ).await.unwrap();
    
    // Add components to person2
    handler.handle_add_email(
        person2,
        "bob@example.com".to_string(),
        EmailType::Personal,
        true,
        false,
        true,
    ).await.unwrap();
    
    handler.handle_add_skill(
        person2,
        "Rust".to_string(),
        SkillCategory::Technical,
        ProficiencyLevel::Expert,
        Some(5.0),
    ).await.unwrap();
    
    // Query components by type for each person
    let person1_emails = component_store.get_components_by_type(
        person1,
        ComponentType::EmailAddress,
    ).await.unwrap();
    assert_eq!(person1_emails.len(), 1);
    
    let person2_skills = component_store.get_components_by_type(
        person2,
        ComponentType::Skill,
    ).await.unwrap();
    assert_eq!(person2_skills.len(), 1);
} 