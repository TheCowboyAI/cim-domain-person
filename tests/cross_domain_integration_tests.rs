//! Integration tests for cross-domain functionality

use chrono::Utc;
use cim_domain_person::{
    aggregate::{ComponentType, Person, PersonId},
    commands::PersonCommand,
    cross_domain::{
        AddressType, AddressUsageType, AgentDomainEvent, AgentEventHandler, AgentType,
        CrossDomainCommand, CrossDomainEvent, GitDomainEvent, GitEventHandler, IdentityDomainEvent,
        IdentityEventHandler, LanguageStats, LocationDomainEvent, LocationEventHandler,
    },
    infrastructure::{
        ComponentStore, InMemoryComponentStore, InMemoryEventStore, InMemorySnapshotStore,
        PersonRepository,
    },
    value_objects::PersonName,
};
use std::sync::Arc;

/// Test handling organization events from Identity domain
#[tokio::test]
async fn test_identity_domain_integration() {
    // Setup
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    let handler = IdentityEventHandler::new(person_repo.clone(), component_store.clone());

    // Create a person
    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("John".to_string(), "Doe".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(
            cim_domain_person::commands::CreatePerson {
                person_id,
                name: PersonName::new("John".to_string(), "Doe".to_string()),
                source: "test".to_string(),
            },
        ))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Handle person joining organization
    let event = IdentityDomainEvent::PersonJoinedOrganization {
        person_id,
        org_id: "org123".to_string(),
        role: "Software Engineer".to_string(),
        department: Some("Engineering".to_string()),
        start_date: Utc::now(),
        employment_type: "full-time".to_string(),
    };

    let events = handler.handle_event(event).await.unwrap();
    assert_eq!(events.len(), 1);

    // Verify employment component was created
    let components = component_store
        .get_components_by_type(person_id, ComponentType::Employment)
        .await
        .unwrap();
    assert_eq!(components.len(), 1);
}

/// Test handling address events from Location domain
#[tokio::test]
async fn test_location_domain_integration() {
    // Setup
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    let handler = LocationEventHandler::new(person_repo.clone());

    // Create a person
    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("Jane".to_string(), "Smith".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(
            cim_domain_person::commands::CreatePerson {
                person_id,
                name: PersonName::new("Jane".to_string(), "Smith".to_string()),
                source: "test".to_string(),
            },
        ))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Handle address association
    let event = LocationDomainEvent::AddressAssociatedWithPerson {
        address_id: "addr123".to_string(),
        person_id,
        address_type: AddressUsageType::Residential,
        is_primary: true,
        effective_date: Utc::now(),
    };

    let events = handler.handle_event(event).await.unwrap();

    // Should register Address component
    assert_eq!(events.len(), 1);
    match &events[0] {
        cim_domain_person::events::PersonEvent::ComponentRegistered(e) => {
            assert_eq!(e.component_type, ComponentType::Address);
        }
        _ => panic!("Expected ComponentRegistered event"),
    }
}

/// Test handling Git contribution events
#[tokio::test]
async fn test_git_domain_integration() {
    // Setup
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    let handler = GitEventHandler::new(person_repo.clone(), component_store.clone());

    // Create a person
    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("Dev".to_string(), "Eloper".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(
            cim_domain_person::commands::CreatePerson {
                person_id,
                name: PersonName::new("Dev".to_string(), "Eloper".to_string()),
                source: "test".to_string(),
            },
        ))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Handle contribution metrics
    let event = GitDomainEvent::ContributionMetricsCalculated {
        person_id,
        repository: "awesome-project".to_string(),
        total_commits: 150,
        total_lines_added: 5000,
        total_lines_deleted: 2000,
        first_commit: Utc::now() - chrono::Duration::days(365),
        last_commit: Utc::now(),
        primary_languages: vec![
            LanguageStats {
                language: "Rust".to_string(),
                files: 50,
                lines: 10000,
                percentage: 60.0,
            },
            LanguageStats {
                language: "Python".to_string(),
                files: 20,
                lines: 3000,
                percentage: 20.0,
            },
        ],
    };

    let events = handler.handle_event(event).await.unwrap();

    // Should create skill components for languages
    assert_eq!(events.len(), 2); // Rust and Python skills

    // Verify skills were created
    let components = component_store
        .get_components_by_type(person_id, ComponentType::Skill)
        .await
        .unwrap();
    assert_eq!(components.len(), 2);
}

/// Test handling agent assignment events
#[tokio::test]
async fn test_agent_domain_integration() {
    // Setup
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    let handler = AgentEventHandler::new(person_repo.clone());

    // Create a person
    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("User".to_string(), "Name".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(
            cim_domain_person::commands::CreatePerson {
                person_id,
                name: PersonName::new("User".to_string(), "Name".to_string()),
                source: "test".to_string(),
            },
        ))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Handle agent assignment
    let event = AgentDomainEvent::AgentAssignedToPerson {
        agent_id: "agent123".to_string(),
        person_id,
        assignment_type: cim_domain_person::cross_domain::AssignmentType::Exclusive,
        permissions: vec![
            cim_domain_person::cross_domain::AgentPermission::ReadPersonalData,
            cim_domain_person::cross_domain::AgentPermission::AccessCalendar,
        ],
        assigned_at: Utc::now(),
    };

    let events = handler.handle_event(event).await.unwrap();

    // For now, this just logs - no events generated
    assert_eq!(events.len(), 0);
}

/// Test cross-domain command sending
#[tokio::test]
async fn test_cross_domain_commands() {
    let person_id = PersonId::new();

    // Test various cross-domain commands
    let commands = vec![
        CrossDomainCommand::RequestOrganizationDetails {
            org_id: "org123".to_string(),
            requester_id: person_id,
        },
        CrossDomainCommand::CreateAddressForPerson {
            person_id,
            street: "123 Main St".to_string(),
            city: "Anytown".to_string(),
            state: Some("CA".to_string()),
            country: "USA".to_string(),
            postal_code: "12345".to_string(),
            address_type: AddressType::Home,
        },
        CrossDomainCommand::LinkGitIdentity {
            person_id,
            git_email: "dev@example.com".to_string(),
            git_username: Some("devuser".to_string()),
        },
        CrossDomainCommand::RequestAgentAssignment {
            person_id,
            agent_type: "PersonalAssistant".to_string(),
            required_capabilities: vec!["Scheduling".to_string(), "EmailManagement".to_string()],
        },
    ];

    // Just verify commands can be created
    assert_eq!(commands.len(), 4);
}

/// Test multiple domain events affecting same person
#[tokio::test]
async fn test_multi_domain_integration() {
    // Setup shared infrastructure
    let event_store = Arc::new(InMemoryEventStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let person_repo = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10,
    ));

    // Create handlers
    let identity_handler = IdentityEventHandler::new(person_repo.clone(), component_store.clone());
    let location_handler = LocationEventHandler::new(person_repo.clone());
    let git_handler = GitEventHandler::new(person_repo.clone(), component_store.clone());

    // Create a person
    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("Multi".to_string(), "Domain".to_string()),
    );
    let create_events = person
        .handle_command(PersonCommand::CreatePerson(
            cim_domain_person::commands::CreatePerson {
                person_id,
                name: PersonName::new("Multi".to_string(), "Domain".to_string()),
                source: "test".to_string(),
            },
        ))
        .unwrap();
    person_repo
        .save(&person, create_events, None)
        .await
        .unwrap();

    // Process events from multiple domains

    // 1. Person joins organization
    identity_handler
        .handle_event(IdentityDomainEvent::PersonJoinedOrganization {
            person_id,
            org_id: "tech-corp".to_string(),
            role: "Senior Developer".to_string(),
            department: Some("R&D".to_string()),
            start_date: Utc::now(),
            employment_type: "full-time".to_string(),
        })
        .await
        .unwrap();

    // 2. Person gets an address
    let address_events = location_handler
        .handle_event(LocationDomainEvent::AddressAssociatedWithPerson {
            address_id: "home-addr".to_string(),
            person_id,
            address_type: AddressUsageType::Residential,
            is_primary: true,
            effective_date: Utc::now(),
        })
        .await
        .unwrap();

    // Apply the events to the person
    if !address_events.is_empty() {
        let mut person = person_repo.load(person_id).await.unwrap().unwrap();
        let current_version = person.version;
        // The events are already in the correct format
        person_repo
            .save(&person, address_events, Some(current_version))
            .await
            .unwrap();
    }

    // 3. Git contributions analyzed
    git_handler
        .handle_event(GitDomainEvent::ContributionMetricsCalculated {
            person_id,
            repository: "company-repo".to_string(),
            total_commits: 500,
            total_lines_added: 15000,
            total_lines_deleted: 5000,
            first_commit: Utc::now() - chrono::Duration::days(730),
            last_commit: Utc::now(),
            primary_languages: vec![LanguageStats {
                language: "Rust".to_string(),
                files: 100,
                lines: 25000,
                percentage: 80.0,
            }],
        })
        .await
        .unwrap();

    // Verify person has components from all domains
    let employment_components = component_store
        .get_components_by_type(person_id, ComponentType::Employment)
        .await
        .unwrap();
    assert_eq!(employment_components.len(), 1);

    let skill_components = component_store
        .get_components_by_type(person_id, ComponentType::Skill)
        .await
        .unwrap();
    assert_eq!(skill_components.len(), 1);

    // Load person and verify component registrations
    let loaded_person = person_repo.load(person_id).await.unwrap().unwrap();
    assert!(loaded_person.has_component(&ComponentType::Address));
}
