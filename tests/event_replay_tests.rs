//! Tests for event replay functionality in the Person domain
//!
//! ```mermaid
//! graph TD
//!     A[Event Stream] --> B[replay_events]
//!     B --> C[Empty Person]
//!     C --> D[Apply Event 1]
//!     D --> E[Apply Event 2]
//!     E --> F[Apply Event N]
//!     F --> G[Final State]
//!     
//!     H[Snapshot] --> I[replay_from_snapshot]
//!     I --> J[Apply New Events]
//!     J --> K[Updated State]
//! ```

use cim_domain_person::{
    aggregate::{Person, PersonId},
    events::*,
    value_objects::*,
};
use chrono::Utc;

#[test]
fn test_event_replay_from_empty() {
    // Create a sequence of events
    let person_id = PersonId::new();
    let events = vec![
        PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: PersonName::new("John".to_string(), "Doe".to_string()),
            source: "Test".to_string(),
            created_at: Utc::now(),
        }),
        PersonEvent::EmailAdded(EmailAdded {
            person_id,
            email: EmailAddress {
                address: "john@example.com".to_string(),
                verified: false,
            },
            primary: true,
            added_at: Utc::now(),
        }),
        PersonEvent::PhoneAdded(PhoneAdded {
            person_id,
            phone: PhoneNumber {
                number: "+1234567890".to_string(),
                country_code: Some("US".to_string()),
                extension: None,
                sms_capable: false,
            },
            primary: true,
            added_at: Utc::now(),
        }),
    ];

    // Replay events
    let person = Person::replay_events(events).unwrap();

    // Verify state
    assert_eq!(person.id, person_id);
    assert_eq!(person.name.given_name, "John");
    assert_eq!(person.name.family_name, "Doe");
    assert_eq!(person.emails.len(), 1);
    assert_eq!(person.primary_email, Some("john@example.com".to_string()));
    assert_eq!(person.phones.len(), 1);
    assert_eq!(person.primary_phone, Some("+1234567890".to_string()));
    assert_eq!(person.version, 3); // 3 events applied
}

#[test]
fn test_event_replay_complex_sequence() {
    let person_id = PersonId::new();
    let org_id = uuid::Uuid::new_v4();
    
    let events = vec![
        PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: PersonName::new("Jane".to_string(), "Smith".to_string()),
            source: "CRM".to_string(),
            created_at: Utc::now(),
        }),
        PersonEvent::EmploymentAdded(EmploymentAdded {
            person_id,
            employment: Employment {
                organization_id: org_id,
                organization_name: "TechCorp".to_string(),
                position: "Software Engineer".to_string(),
                department: Some("Engineering".to_string()),
                start_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                end_date: None,
                employment_type: EmploymentType::FullTime,
                manager_id: None,
            },
            added_at: Utc::now(),
        }),
        PersonEvent::SkillAdded(SkillAdded {
            person_id,
            skill: Skill {
                name: "Rust".to_string(),
                category: "Programming".to_string(),
                proficiency: ProficiencyLevel::Expert,
                years_experience: Some(5.0),
                last_used: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                certifications: vec![],
            },
            added_at: Utc::now(),
        }),
        PersonEvent::PersonDeactivated(PersonDeactivated {
            person_id,
            reason: "Test deactivation".to_string(),
            deactivated_at: Utc::now(),
        }),
        PersonEvent::PersonReactivated(PersonReactivated {
            person_id,
            reason: "Test reactivation".to_string(),
            reactivated_at: Utc::now(),
        }),
    ];

    let person = Person::replay_events(events).unwrap();

    assert_eq!(person.employments.len(), 1);
    assert_eq!(person.current_employment, Some(org_id));
    assert_eq!(person.skills.len(), 1);
    assert!(person.skills.contains_key("Rust"));
    assert!(person.is_active);
    assert_eq!(person.version, 5);
}

#[test]
fn test_event_replay_with_snapshot() {
    let person_id = PersonId::new();
    
    // Create initial events
    let initial_events = vec![
        PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: PersonName::new("Bob".to_string(), "Johnson".to_string()),
            source: "Import".to_string(),
            created_at: Utc::now(),
        }),
        PersonEvent::EmailAdded(EmailAdded {
            person_id,
            email: EmailAddress {
                address: "bob@example.com".to_string(),
                verified: true,
            },
            primary: true,
            added_at: Utc::now(),
        }),
    ];

    // Create snapshot from initial events
    let snapshot = Person::replay_events(initial_events).unwrap();
    let snapshot_version = snapshot.version();

    // New events after snapshot
    let new_events = vec![
        PersonEvent::AddressAdded(AddressAdded {
            person_id,
            address: PhysicalAddress {
                street_lines: vec!["123 Main St".to_string()],
                city: "Anytown".to_string(),
                state_province: Some("CA".to_string()),
                postal_code: Some("12345".to_string()),
                country: "USA".to_string(),
            },
            address_type: AddressType::Home,
            added_at: Utc::now(),
        }),
        PersonEvent::TagAdded(TagAdded {
            person_id,
            tag: Tag {
                name: "VIP".to_string(),
                category: "Customer".to_string(),
                added_by: uuid::Uuid::new_v4(),
                added_at: Utc::now(),
            },
            added_at: Utc::now(),
        }),
    ];

    // Replay from snapshot
    let final_person = Person::replay_from_snapshot(
        snapshot,
        new_events,
        snapshot_version
    ).unwrap();

    assert_eq!(final_person.emails.len(), 1);
    assert_eq!(final_person.addresses.len(), 1);
    assert_eq!(final_person.tags.len(), 1);
    assert_eq!(final_person.version, 4); // 2 initial + 2 new events
}

#[test]
fn test_event_replay_empty_stream_error() {
    let result = Person::replay_events(vec![]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot replay empty event stream");
}

#[test]
fn test_event_replay_missing_created_event_error() {
    let person_id = PersonId::new();
    let events = vec![
        PersonEvent::EmailAdded(EmailAdded {
            person_id,
            email: EmailAddress {
                address: "test@example.com".to_string(),
                verified: false,
            },
            primary: true,
            added_at: Utc::now(),
        }),
    ];

    let result = Person::replay_events(events);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Event stream must start with PersonCreated event");
}

#[test]
fn test_event_replay_version_tracking() {
    let person_id = PersonId::new();
    let mut events = vec![
        PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: PersonName::new("Test".to_string(), "User".to_string()),
            source: "Test".to_string(),
            created_at: Utc::now(),
        }),
    ];

    // Add 10 more events
    for i in 0..10 {
        events.push(PersonEvent::TagAdded(TagAdded {
            person_id,
            tag: Tag {
                name: format!("Tag{}", i),
                category: "Test".to_string(),
                added_by: uuid::Uuid::new_v4(),
                added_at: Utc::now(),
            },
            added_at: Utc::now(),
        }));
    }

    let person = Person::replay_events(events).unwrap();
    assert_eq!(person.version, 11);
    assert_eq!(person.tags.len(), 10);
}

#[test]
fn test_snapshot_version_mismatch() {
    let person_id = PersonId::new();
    let events = vec![
        PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: PersonName::new("Test".to_string(), "User".to_string()),
            source: "Test".to_string(),
            created_at: Utc::now(),
        }),
    ];

    let mut snapshot = Person::replay_events(events).unwrap();
    snapshot.version = 99; // Corrupt the version

    let result = Person::replay_from_snapshot(
        snapshot,
        vec![],
        1 // Expected version doesn't match
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not match expected version"));
} 