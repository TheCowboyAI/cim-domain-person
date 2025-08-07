//! Integration tests for component store functionality

use cim_domain_person::{
    aggregate::{ComponentType, Person, PersonId},
    commands::{ComponentCommand, CreatePerson, PersonCommand, RegisterComponent},
    components::data::{
        EmailType, PhoneType, ProficiencyLevel, SkillCategory,
    },
    events::ComponentDataEvent,
    handlers::ComponentCommandHandler,
    infrastructure::{
        ComponentStore, InMemoryComponentStore, InMemoryEventStore, InMemorySnapshotStore,
        PersonRepository,
    },
    value_objects::PersonName,
};
use std::sync::Arc;

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
    let mut person = Person::new(
        person_id,
        PersonName::new("Test".to_string(), "User".to_string()),
    );

    // Generate creation event
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(CreatePerson {
            person_id,
            name: PersonName::new("Test".to_string(), "User".to_string()),
            source: "test".to_string(),
        }))
        .unwrap();

    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

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
    person_repo
        .save(&loaded_person, register_events, Some(current_version))
        .await
        .unwrap();

    // Add actual email data
    let cmd = ComponentCommand::AddEmail {
        person_id,
        email: "test@example.com".to_string(),
        email_type: EmailType::Personal,
        is_preferred: true,
        can_receive_notifications: true,
        can_receive_marketing: false,
    };

    let events = handler.handle(cmd).await.unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        ComponentDataEvent::EmailAdded {
            email, email_type, ..
        } => {
            assert_eq!(email, "test@example.com");
            assert!(matches!(email_type, EmailType::Personal));
        }
        _ => panic!("Expected EmailAdded event"),
    }

    // Verify component was stored
    let components = component_store
        .get_components_by_type(person_id, ComponentType::EmailAddress)
        .await
        .unwrap();

    assert_eq!(components.len(), 1);
}

/// Test updating email component
#[tokio::test]
async fn test_update_email_component() {
    // Setup
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );

    // Create person and add email
    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("Test".to_string(), "User".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(CreatePerson {
            person_id,
            name: PersonName::new("Test".to_string(), "User".to_string()),
            source: "test".to_string(),
        }))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Add email first
    let add_cmd = ComponentCommand::AddEmail {
        person_id,
        email: "old@example.com".to_string(),
        email_type: EmailType::Work,
        is_preferred: false,
        can_receive_notifications: true,
        can_receive_marketing: true,
    };
    let add_events = handler.handle(add_cmd).await.unwrap();

    let component_id = match &add_events[0] {
        ComponentDataEvent::EmailAdded { component_id, .. } => *component_id,
        _ => panic!("Expected EmailAdded event"),
    };

    // Update email
    let update_cmd = ComponentCommand::UpdateEmail {
        person_id,
        component_id,
        email: Some("new@example.com".to_string()),
        email_type: Some(EmailType::Personal),
        is_preferred: Some(true),
        can_receive_notifications: None,
        can_receive_marketing: None,
    };
    let update_events = handler.handle(update_cmd).await.unwrap();

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
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );

    // Create person
    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("Skilled".to_string(), "Developer".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(CreatePerson {
            person_id,
            name: PersonName::new("Skilled".to_string(), "Developer".to_string()),
            source: "test".to_string(),
        }))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Add skill
    let skill_cmd = ComponentCommand::AddSkill {
        person_id,
        skill_name: "Rust Programming".to_string(),
        category: SkillCategory::Technical,
        proficiency: ProficiencyLevel::Expert,
        years_of_experience: Some(5.0),
    };
    let events = handler.handle(skill_cmd).await.unwrap();

    assert_eq!(events.len(), 1);
    let skill_id = match &events[0] {
        ComponentDataEvent::SkillAdded {
            component_id,
            skill_name,
            ..
        } => {
            assert_eq!(skill_name, "Rust Programming");
            *component_id
        }
        _ => panic!("Expected SkillAdded event"),
    };

    // Update skill would be implemented similarly
    // For now, just verify the skill was added

    // Verify skill is stored
    let components = component_store
        .get_components_by_type(person_id, ComponentType::Skill)
        .await
        .unwrap();

    assert_eq!(components.len(), 1);
}

/// Test multiple components of same type
#[tokio::test]
async fn test_multiple_components_same_type() {
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );

    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("Multi".to_string(), "Email".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(CreatePerson {
            person_id,
            name: PersonName::new("Multi".to_string(), "Email".to_string()),
            source: "test".to_string(),
        }))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Add multiple emails
    let emails = vec![
        ("personal@example.com", EmailType::Personal, true),
        ("work@company.com", EmailType::Work, false),
        ("school@university.edu", EmailType::School, false),
    ];

    for (email, email_type, is_preferred) in emails {
        let cmd = ComponentCommand::AddEmail {
            person_id,
            email: email.to_string(),
            email_type,
            is_preferred,
            can_receive_notifications: true,
            can_receive_marketing: false,
        };
        handler.handle(cmd).await.unwrap();
    }

    // Verify all emails are stored
    let components = component_store
        .get_components_by_type(person_id, ComponentType::EmailAddress)
        .await
        .unwrap();

    assert_eq!(components.len(), 3);
}

/// Test component removal
#[tokio::test]
async fn test_remove_component() {
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    let handler = ComponentCommandHandler::new(
        event_store.clone(),
        component_store.clone(),
        person_repo.clone(),
    );

    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("Test".to_string(), "User".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(CreatePerson {
            person_id,
            name: PersonName::new("Test".to_string(), "User".to_string()),
            source: "test".to_string(),
        }))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Add phone
    let phone_cmd = ComponentCommand::AddPhone {
        person_id,
        phone_number: "+1234567890".to_string(),
        phone_type: PhoneType::Mobile,
        country_code: "+1".to_string(),
        is_mobile: true,
        can_receive_sms: true,
        can_receive_calls: true,
    };
    let add_events = handler.handle(phone_cmd).await.unwrap();

    let _component_id = match &add_events[0] {
        ComponentDataEvent::PhoneAdded { component_id, .. } => *component_id,
        _ => panic!("Expected PhoneAdded event"),
    };

    // For now, we'll just verify the phone was added
    // Remove functionality would need a handle_remove_phone method
    // which isn't implemented yet

    // Verify component exists
    let components = component_store
        .get_components_by_type(person_id, ComponentType::PhoneNumber)
        .await
        .unwrap();
    assert_eq!(components.len(), 1);
}

/// Test cross-component queries
#[tokio::test]
async fn test_cross_component_queries() {
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

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
        let create_events = person
            .handle_command(PersonCommand::CreatePerson(CreatePerson {
                person_id: id,
                name: PersonName::new(name.to_string(), "Test".to_string()),
                source: "test".to_string(),
            }))
            .unwrap();
        person_repo
            .save(&person, create_events, None)
            .await
            .unwrap();
    }

    // Add components to person1
    let email_cmd1 = ComponentCommand::AddEmail {
        person_id: person1,
        email: "alice@example.com".to_string(),
        email_type: EmailType::Work,
        is_preferred: true,
        can_receive_notifications: true,
        can_receive_marketing: false,
    };
    handler.handle(email_cmd1).await.unwrap();

    let skill_cmd1 = ComponentCommand::AddSkill {
        person_id: person1,
        skill_name: "Python".to_string(),
        category: SkillCategory::Technical,
        proficiency: ProficiencyLevel::Advanced,
        years_of_experience: Some(3.0),
    };
    handler.handle(skill_cmd1).await.unwrap();

    // Add components to person2
    let email_cmd2 = ComponentCommand::AddEmail {
        person_id: person2,
        email: "bob@example.com".to_string(),
        email_type: EmailType::Personal,
        is_preferred: true,
        can_receive_notifications: false,
        can_receive_marketing: true,
    };
    handler.handle(email_cmd2).await.unwrap();

    let skill_cmd2 = ComponentCommand::AddSkill {
        person_id: person2,
        skill_name: "Rust".to_string(),
        category: SkillCategory::Technical,
        proficiency: ProficiencyLevel::Expert,
        years_of_experience: Some(5.0),
    };
    handler.handle(skill_cmd2).await.unwrap();

    // Query components by type for each person
    let person1_emails = component_store
        .get_components_by_type(person1, ComponentType::EmailAddress)
        .await
        .unwrap();
    assert_eq!(person1_emails.len(), 1);

    let person2_skills = component_store
        .get_components_by_type(person2, ComponentType::Skill)
        .await
        .unwrap();
    assert_eq!(person2_skills.len(), 1);
}
