//! Tests for ECS-oriented Person aggregate

use chrono::Utc;
use cim_domain_person::{
    aggregate::{ComponentType, Person, PersonId, PersonLifecycle},
    commands::*,
    events::*,
    value_objects::PersonName,
};

#[test]
fn test_create_person() {
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());

    let person = Person::new(person_id, name.clone());

    assert_eq!(person.id, person_id);
    assert_eq!(person.core_identity.legal_name, name);
    assert!(person.is_active());
    assert_eq!(person.components.len(), 0);
}

#[test]
fn test_handle_create_person_command() {
    let person_id = PersonId::new();
    let name = PersonName::new("Jane".to_string(), "Smith".to_string());
    let mut person = Person::empty();

    let cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: name.clone(),
        source: "test".to_string(),
    });

    let events = person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::PersonCreated(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.name, name);
            assert_eq!(e.source, "test");
        }
        _ => panic!("Expected PersonCreated event"),
    }
}

#[test]
fn test_update_name() {
    let person_id = PersonId::new();
    let old_name = PersonName::new("John".to_string(), "Doe".to_string());
    let new_name = PersonName::new("John".to_string(), "Smith".to_string());
    let mut person = Person::new(person_id, old_name.clone());

    let cmd = PersonCommand::UpdateName(UpdateName {
        person_id,
        name: new_name.clone(),
        reason: Some("Marriage".to_string()),
    });

    let events = person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::NameUpdated(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.old_name, old_name);
            assert_eq!(e.new_name, new_name);
            assert_eq!(e.reason, Some("Marriage".to_string()));
        }
        _ => panic!("Expected NameUpdated event"),
    }
}

#[test]
fn test_set_birth_date() {
    let person_id = PersonId::new();
    let name = PersonName::new("Baby".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    let birth_date = chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();

    let cmd = PersonCommand::SetBirthDate(SetBirthDate {
        person_id,
        birth_date,
    });

    let events = person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::BirthDateSet(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.birth_date, birth_date);
        }
        _ => panic!("Expected BirthDateSet event"),
    }
}

#[test]
fn test_cannot_change_birth_date() {
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Set birth date first time
    person.core_identity.birth_date = Some(chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap());

    // Try to set it again
    let cmd = PersonCommand::SetBirthDate(SetBirthDate {
        person_id,
        birth_date: chrono::NaiveDate::from_ymd_opt(1991, 1, 1).unwrap(),
    });

    let result = person.handle_command(cmd);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Birth date is immutable once set");
}

#[test]
fn test_register_component() {
    let person_id = PersonId::new();
    let name = PersonName::new("Component".to_string(), "User".to_string());
    let mut person = Person::new(person_id, name);

    let cmd = PersonCommand::RegisterComponent(RegisterComponent {
        person_id,
        component_type: ComponentType::EmailAddress,
    });

    let events = person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::ComponentRegistered(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.component_type, ComponentType::EmailAddress);
        }
        _ => panic!("Expected ComponentRegistered event"),
    }
}

#[test]
fn test_unregister_component() {
    let person_id = PersonId::new();
    let name = PersonName::new("Component".to_string(), "User".to_string());
    let mut person = Person::new(person_id, name);

    // First register a component
    person.components.insert(ComponentType::EmailAddress);

    let cmd = PersonCommand::UnregisterComponent(UnregisterComponent {
        person_id,
        component_type: ComponentType::EmailAddress,
    });

    let events = person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::ComponentUnregistered(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.component_type, ComponentType::EmailAddress);
        }
        _ => panic!("Expected ComponentUnregistered event"),
    }
}

#[test]
fn test_deactivate_person() {
    let person_id = PersonId::new();
    let name = PersonName::new("Active".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    let cmd = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id,
        reason: "Inactive account".to_string(),
    });

    let events = person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::PersonDeactivated(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.reason, "Inactive account");
        }
        _ => panic!("Expected PersonDeactivated event"),
    }
}

#[test]
fn test_reactivate_person() {
    let person_id = PersonId::new();
    let name = PersonName::new("Inactive".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // First deactivate
    person.lifecycle = PersonLifecycle::Deactivated {
        reason: "Test".to_string(),
        since: Utc::now(),
    };

    let cmd = PersonCommand::ReactivatePerson(ReactivatePerson {
        person_id,
        reason: "Account restored".to_string(),
    });

    let events = person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::PersonReactivated(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.reason, "Account restored");
        }
        _ => panic!("Expected PersonReactivated event"),
    }
}

#[test]
fn test_record_death() {
    let person_id = PersonId::new();
    let name = PersonName::new("Living".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    let date_of_death = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let cmd = PersonCommand::RecordDeath(RecordDeath {
        person_id,
        date_of_death,
    });

    let events = person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::DeathRecorded(e) => {
            assert_eq!(e.person_id, person_id);
            assert_eq!(e.date_of_death, date_of_death);
        }
        _ => panic!("Expected DeathRecorded event"),
    }
}

#[test]
fn test_merge_persons() {
    let source_id = PersonId::new();
    let target_id = PersonId::new();
    let name = PersonName::new("Duplicate".to_string(), "Person".to_string());
    let mut source_person = Person::new(source_id, name);

    let cmd = PersonCommand::MergePersons(MergePersons {
        source_person_id: source_id,
        target_person_id: target_id,
        merge_reason: MergeReason::DuplicateIdentity,
    });

    let events = source_person.handle_command(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::PersonMergedInto(e) => {
            assert_eq!(e.source_person_id, source_id);
            assert_eq!(e.merged_into_id, target_id);
            match &e.merge_reason {
                MergeReason::DuplicateIdentity => (),
                _ => panic!("Expected DuplicateIdentity reason"),
            }
        }
        _ => panic!("Expected PersonMergedInto event"),
    }
}

#[test]
fn test_cannot_modify_merged_person() {
    let person_id = PersonId::new();
    let name = PersonName::new("Merged".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Set as merged
    person.lifecycle = PersonLifecycle::MergedInto {
        target_id: PersonId::new(),
        merged_at: Utc::now(),
    };

    let cmd = PersonCommand::UpdateName(UpdateName {
        person_id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: None,
    });

    let result = person.handle_command(cmd);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot modify a merged person");
}

#[test]
fn test_cannot_modify_deceased_person() {
    let person_id = PersonId::new();
    let name = PersonName::new("Deceased".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Set as deceased
    person.lifecycle = PersonLifecycle::Deceased {
        date_of_death: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    };

    let cmd = PersonCommand::UpdateName(UpdateName {
        person_id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: None,
    });

    let result = person.handle_command(cmd);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot modify a deceased person");
}
