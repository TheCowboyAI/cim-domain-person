//! Tests for Person component management in ECS architecture

use cim_domain_person::{
    aggregate::{ComponentType, Person, PersonId, PersonLifecycle},
    commands::*,
    events::*,
    value_objects::PersonName,
};
use std::collections::HashSet;

#[test]
fn test_component_registration() {
    let person_id = PersonId::new();
    let name = PersonName::new("Component".to_string(), "User".to_string());
    let mut person = Person::new(person_id, name);

    // Initially no components
    assert_eq!(person.components.len(), 0);
    assert!(!person.has_component(&ComponentType::EmailAddress));

    // Register email component
    let register_email = PersonCommand::RegisterComponent(RegisterComponent {
        person_id,
        component_type: ComponentType::EmailAddress,
    });

    let events = person.handle_command(register_email).unwrap();
    assert_eq!(events.len(), 1);

    // Apply event (in real system this would be done by event handler)
    person
        .register_component(ComponentType::EmailAddress)
        .unwrap();

    assert!(person.has_component(&ComponentType::EmailAddress));
    assert_eq!(person.components.len(), 1);

    // Cannot register same component twice
    let register_again = PersonCommand::RegisterComponent(RegisterComponent {
        person_id,
        component_type: ComponentType::EmailAddress,
    });

    let result = person.handle_command(register_again);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Component already registered");
}

#[test]
fn test_component_unregistration() {
    let person_id = PersonId::new();
    let name = PersonName::new("Component".to_string(), "User".to_string());
    let mut person = Person::new(person_id, name);

    // Register multiple components
    person
        .register_component(ComponentType::EmailAddress)
        .unwrap();
    person
        .register_component(ComponentType::PhoneNumber)
        .unwrap();
    person.register_component(ComponentType::Skill).unwrap();

    assert_eq!(person.components.len(), 3);

    // Unregister phone component
    let unregister_phone = PersonCommand::UnregisterComponent(UnregisterComponent {
        person_id,
        component_type: ComponentType::PhoneNumber,
    });

    let events = person.handle_command(unregister_phone).unwrap();
    assert_eq!(events.len(), 1);

    // Apply event
    person
        .unregister_component(&ComponentType::PhoneNumber)
        .unwrap();

    assert!(!person.has_component(&ComponentType::PhoneNumber));
    assert_eq!(person.components.len(), 2);

    // Cannot unregister non-existent component
    let unregister_missing = PersonCommand::UnregisterComponent(UnregisterComponent {
        person_id,
        component_type: ComponentType::Address,
    });

    let result = person.handle_command(unregister_missing);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Component not registered");
}

#[test]
fn test_multiple_component_types() {
    let person_id = PersonId::new();
    let name = PersonName::new("Multi".to_string(), "Component".to_string());
    let mut person = Person::new(person_id, name);

    // Register various component types
    let component_types = vec![
        ComponentType::EmailAddress,
        ComponentType::PhoneNumber,
        ComponentType::Address,
        ComponentType::Employment,
        ComponentType::Skill,
        ComponentType::Certification,
        ComponentType::SocialProfile,
        ComponentType::CustomerSegment,
        ComponentType::CommunicationPreferences,
        ComponentType::Tag,
        ComponentType::CustomAttribute,
    ];

    for comp_type in &component_types {
        person.register_component(comp_type.clone()).unwrap();
    }

    assert_eq!(person.components.len(), component_types.len());

    // Verify all are registered
    for comp_type in &component_types {
        assert!(person.has_component(comp_type));
    }
}

#[test]
fn test_component_registration_when_inactive() {
    let person_id = PersonId::new();
    let name = PersonName::new("Inactive".to_string(), "User".to_string());
    let mut person = Person::new(person_id, name);

    // Deactivate person
    person.lifecycle = PersonLifecycle::Deactivated {
        reason: "Suspended".to_string(),
        since: chrono::Utc::now(),
    };

    // Cannot register components when inactive
    let result = person.register_component(ComponentType::EmailAddress);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Validation error: Cannot add components to inactive person"
    );

    // But can unregister
    person.components.insert(ComponentType::PhoneNumber);
    let result = person.unregister_component(&ComponentType::PhoneNumber);
    assert!(result.is_ok());
}

#[test]
fn test_external_component_types() {
    let person_id = PersonId::new();
    let name = PersonName::new("External".to_string(), "Components".to_string());
    let mut person = Person::new(person_id, name);

    // Register various component types
    let external_types = vec![
        ComponentType::CustomerSegment,
        ComponentType::Employment,
        ComponentType::PrivacyPreferences,
        ComponentType::GeneralPreferences,
    ];

    for ext_type in &external_types {
        let cmd = PersonCommand::RegisterComponent(RegisterComponent {
            person_id,
            component_type: ext_type.clone(),
        });

        let events = person.handle_command(cmd).unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            PersonEvent::ComponentRegistered(e) => {
                assert_eq!(e.component_type, *ext_type);
            }
            _ => panic!("Expected ComponentRegistered event"),
        }

        person.register_component(ext_type.clone()).unwrap();
    }

    assert_eq!(person.components.len(), external_types.len());
}

#[test]
fn test_component_events() {
    let person_id = PersonId::new();
    let name = PersonName::new("Event".to_string(), "Test".to_string());
    let mut person = Person::new(person_id, name);

    // Test registration event
    let register_cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id,
        component_type: ComponentType::BehavioralData,
    });

    let events = person.handle_command(register_cmd).unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        PersonEvent::ComponentRegistered(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.component_type, ComponentType::BehavioralData);
        }
        _ => panic!("Expected ComponentRegistered event"),
    }

    // Apply and test unregistration event
    person
        .register_component(ComponentType::BehavioralData)
        .unwrap();

    let unregister_cmd = PersonCommand::UnregisterComponent(UnregisterComponent {
        person_id,
        component_type: ComponentType::BehavioralData,
    });

    let events = person.handle_command(unregister_cmd).unwrap();
    assert_eq!(events.len(), 1);

    match &events[0] {
        PersonEvent::ComponentUnregistered(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.component_type, ComponentType::BehavioralData);
        }
        _ => panic!("Expected ComponentUnregistered event"),
    }
}
