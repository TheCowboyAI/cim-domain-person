//! Tests for Person lifecycle management in ECS architecture

use chrono::{NaiveDate, Utc};
use cim_domain_person::{
    aggregate::{ComponentType, Person, PersonId, PersonLifecycle},
    commands::*,
    events::*,
    value_objects::PersonName,
};

#[test]
fn test_person_lifecycle_transitions() {
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Start as active
    assert!(matches!(person.lifecycle, PersonLifecycle::Active));
    assert!(person.is_active());

    // Deactivate
    person.lifecycle = PersonLifecycle::Deactivated {
        reason: "Account suspended".to_string(),
        since: Utc::now(),
    };
    assert!(!person.is_active());

    // Can't add components when deactivated
    let result = person.register_component(ComponentType::EmailAddress);
    assert!(result.is_err());

    // Record death
    person.lifecycle = PersonLifecycle::Deceased {
        date_of_death: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    };
    assert!(!person.is_active());

    // Merge into another
    person.lifecycle = PersonLifecycle::MergedInto {
        target_id: PersonId::new(),
        merged_at: Utc::now(),
    };
    assert!(!person.is_active());
}

#[test]
fn test_lifecycle_command_validation() {
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name.clone());

    // Can update active person
    let cmd = PersonCommand::UpdateName(UpdateName {
        person_id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: None,
    });
    assert!(person.handle_command(cmd).is_ok());

    // Cannot update deceased person
    person.lifecycle = PersonLifecycle::Deceased {
        date_of_death: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    };

    let cmd = PersonCommand::UpdateName(UpdateName {
        person_id,
        name: PersonName::new("Another".to_string(), "Name".to_string()),
        reason: None,
    });
    let result = person.handle_command(cmd);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot modify a deceased person");
}

#[test]
fn test_deactivation_and_reactivation() {
    let person_id = PersonId::new();
    let name = PersonName::new("Active".to_string(), "User".to_string());
    let mut person = Person::new(person_id, name);

    // Deactivate
    let deactivate_cmd = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id,
        reason: "Inactive for 90 days".to_string(),
    });

    let events = person.handle_command(deactivate_cmd).unwrap();
    assert_eq!(events.len(), 1);

    // Apply the event (in real system this would be done by event handler)
    person.lifecycle = PersonLifecycle::Deactivated {
        reason: "Inactive for 90 days".to_string(),
        since: Utc::now(),
    };

    // Cannot deactivate again
    let deactivate_again = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id,
        reason: "Still inactive".to_string(),
    });
    assert!(person.handle_command(deactivate_again).is_err());

    // Reactivate
    let reactivate_cmd = PersonCommand::ReactivatePerson(ReactivatePerson {
        person_id,
        reason: "User logged in".to_string(),
    });

    let events = person.handle_command(reactivate_cmd).unwrap();
    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::PersonReactivated(e) => {
            assert_eq!(e.reason, "User logged in");
        }
        _ => panic!("Expected PersonReactivated event"),
    }
}

#[test]
fn test_merge_lifecycle() {
    let source_id = PersonId::new();
    let target_id = PersonId::new();
    let name = PersonName::new("Duplicate".to_string(), "Account".to_string());
    let mut source = Person::new(source_id, name);

    // Add some components to source
    source.components.insert(ComponentType::EmailAddress);
    source.components.insert(ComponentType::PhoneNumber);

    // Merge command
    let merge_cmd = PersonCommand::MergePersons(MergePersons {
        source_person_id: source_id,
        target_person_id: target_id,
        merge_reason: MergeReason::DuplicateIdentity,
    });

    let events = source.handle_command(merge_cmd).unwrap();
    assert_eq!(events.len(), 1);

    // Apply merge (in real system this would be done by event handler)
    source.lifecycle = PersonLifecycle::MergedInto {
        target_id,
        merged_at: Utc::now(),
    };

    // Cannot modify merged person
    let update_cmd = PersonCommand::UpdateName(UpdateName {
        person_id: source_id,
        name: PersonName::new("Cannot".to_string(), "Update".to_string()),
        reason: None,
    });

    let result = source.handle_command(update_cmd);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot modify a merged person");
}

#[test]
fn test_birth_and_death_dates() {
    let person_id = PersonId::new();
    let name = PersonName::new("Mortal".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Set birth date
    let birth_date = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();
    let set_birth = PersonCommand::SetBirthDate(SetBirthDate {
        person_id,
        birth_date,
    });

    let events = person.handle_command(set_birth).unwrap();
    assert_eq!(events.len(), 1);

    // Apply event
    person.core_identity.birth_date = Some(birth_date);

    // Cannot change birth date once set
    let new_birth = PersonCommand::SetBirthDate(SetBirthDate {
        person_id,
        birth_date: NaiveDate::from_ymd_opt(1991, 5, 15).unwrap(),
    });

    let result = person.handle_command(new_birth);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Birth date is immutable once set");

    // Record death
    let death_date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap();
    let record_death = PersonCommand::RecordDeath(RecordDeath {
        person_id,
        date_of_death: death_date,
    });

    let events = person.handle_command(record_death).unwrap();
    assert_eq!(events.len(), 1);

    // Apply death
    person.lifecycle = PersonLifecycle::Deceased {
        date_of_death: death_date,
    };
    person.core_identity.death_date = Some(death_date);

    // Cannot record death again
    let record_again = PersonCommand::RecordDeath(RecordDeath {
        person_id,
        date_of_death: NaiveDate::from_ymd_opt(2024, 1, 21).unwrap(),
    });

    assert!(person.handle_command(record_again).is_err());
}
