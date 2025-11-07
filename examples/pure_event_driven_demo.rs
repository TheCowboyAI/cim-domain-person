//! Demonstration of pure event-driven architecture
//!
//! This example shows:
//! - Commands triggering events
//! - Pure functional event application with apply_event_pure
//! - Event replay to reconstruct aggregate state
//! - Event sourcing lifecycle

use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::CreatePerson,
    events::{PersonEvent, PersonCreated, AttributeRecorded, PersonDeactivated, PersonReactivated},
    value_objects::{
        PersonName, PersonAttribute, AttributeType, AttributeValue,
        IdentifyingAttributeType, PhysicalAttributeType,
        TemporalValidity, Provenance, AttributeSource, ConfidenceLevel,
    },
};
use chrono::{Utc, NaiveDate};

fn main() {
    println!("=== Pure Event-Driven Architecture Demo ===\n");

    // Demo 1: Command → Event → State
    demo_command_to_event();

    // Demo 2: Pure event application
    demo_pure_event_application();

    // Demo 3: Event replay
    demo_event_replay();

    // Demo 4: Complete lifecycle with events
    demo_lifecycle_events();

    println!("\n✅ Demo completed successfully!");
}

fn demo_command_to_event() {
    println!("--- Demo 1: Command → Event → State ---");

    // Command: Create a person
    let person_id = PersonId::new();
    let command = CreatePerson {
        person_id,
        name: PersonName::new("Alice".to_string(), "Johnson".to_string()),
        source: "demo".to_string(),
    };

    println!("Command: CreatePerson");
    println!("  Person ID: {}", command.person_id);
    println!("  Name: {} {}",
        command.name.components.given_names[0],
        command.name.components.family_names[0]
    );

    // Generate event from command
    let event = PersonEvent::PersonCreated(PersonCreated {
        person_id: command.person_id,
        name: command.name.clone(),
        source: command.source.clone(),
        created_at: Utc::now(),
    });

    println!("\nEvent: PersonCreated");
    println!("  Person ID: {}", person_id);
    println!("  Timestamp: {:?}", Utc::now());

    // Apply event to create aggregate
    let mut person = Person::empty();
    person.id = person_id;
    person = person.apply_event_pure(&event).expect("Failed to apply event");

    println!("\nResulting State:");
    println!("  ID: {}", person.id);
    println!("  Name: {}", person.core_identity.legal_name);
    println!("  Lifecycle: {:?}", person.lifecycle);
    println!();
}

fn demo_pure_event_application() {
    println!("--- Demo 2: Pure Functional Event Application ---");

    // Create initial person
    let person_id = PersonId::new();
    let name = PersonName::new("Bob".to_string(), "Smith".to_string());
    let mut person = Person::new(person_id, name);

    println!("Initial state: {} attributes", person.attributes.attributes.len());

    // Event 1: Record birth date
    let birth_date_event = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: PersonAttribute::new(
            AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
            AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 5, 20).unwrap()),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
        ),
        recorded_at: Utc::now(),
    });

    // Apply event - PURE FUNCTION: doesn't mutate, returns new state
    person = person.apply_event_pure(&birth_date_event)
        .expect("Failed to apply birth date event");

    println!("After recording birth date: {} attributes", person.attributes.attributes.len());

    // Event 2: Record height
    let height_event = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.80),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
        ),
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&height_event)
        .expect("Failed to apply height event");

    println!("After recording height: {} attributes", person.attributes.attributes.len());
    println!();
}

fn demo_event_replay() {
    println!("--- Demo 3: Event Replay to Reconstruct State ---");

    let person_id = PersonId::new();

    // Create event stream (this would normally come from event store)
    let event_stream = vec![
        PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: PersonName::new("Charlie".to_string(), "Brown".to_string()),
            source: "demo".to_string(),
            created_at: Utc::now(),
        }),
        PersonEvent::AttributeRecorded(AttributeRecorded {
            person_id,
            attribute: PersonAttribute::new(
                AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
                AttributeValue::Date(NaiveDate::from_ymd_opt(1985, 12, 1).unwrap()),
                TemporalValidity::of(Utc::now()),
                Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
            ),
            recorded_at: Utc::now(),
        }),
        PersonEvent::AttributeRecorded(AttributeRecorded {
            person_id,
            attribute: PersonAttribute::new(
                AttributeType::Physical(PhysicalAttributeType::Weight),
                AttributeValue::Mass(75.0),
                TemporalValidity::of(Utc::now()),
                Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
            ),
            recorded_at: Utc::now(),
        }),
    ];

    println!("Event stream has {} events", event_stream.len());

    // Replay events to reconstruct aggregate
    let mut person = Person::empty();
    person.id = person_id;

    println!("\nReplaying events:");
    for (i, event) in event_stream.iter().enumerate() {
        println!("  Event {}: {:?}", i + 1, event_type_name(event));
        person = person.apply_event_pure(event)
            .expect("Failed to replay event");
    }

    println!("\nReconstructed state:");
    println!("  ID: {}", person.id);
    println!("  Name: {}", person.core_identity.legal_name);
    println!("  Attributes: {}", person.attributes.attributes.len());
    println!("  Lifecycle: {:?}", person.lifecycle);
    println!();
}

fn demo_lifecycle_events() {
    println!("--- Demo 4: Lifecycle Management via Events ---");

    // Create person
    let person_id = PersonId::new();
    let name = PersonName::new("Diana".to_string(), "Prince".to_string());
    let mut person = Person::new(person_id, name);

    println!("Initial lifecycle: {:?}", person.lifecycle);

    // Event: Deactivate person
    let deactivate_event = PersonEvent::PersonDeactivated(PersonDeactivated {
        person_id,
        reason: "Compliance review required".to_string(),
        deactivated_at: Utc::now(),
    });

    person = person.apply_event_pure(&deactivate_event)
        .expect("Failed to deactivate");

    println!("After deactivation: {:?}", person.lifecycle);
    println!("Is active: {}", person.is_active());

    // Event: Reactivate person
    let reactivate_event = PersonEvent::PersonReactivated(PersonReactivated {
        person_id,
        reason: "Review completed successfully".to_string(),
        reactivated_at: Utc::now(),
    });

    person = person.apply_event_pure(&reactivate_event)
        .expect("Failed to reactivate");

    println!("After reactivation: {:?}", person.lifecycle);
    println!("Is active: {}", person.is_active());
    println!();
}

fn event_type_name(event: &PersonEvent) -> &'static str {
    match event {
        PersonEvent::PersonCreated(_) => "PersonCreated",
        PersonEvent::PersonUpdated(_) => "PersonUpdated",
        PersonEvent::NameUpdated(_) => "NameUpdated",
        PersonEvent::BirthDateSet(_) => "BirthDateSet",
        PersonEvent::DeathRecorded(_) => "DeathRecorded",
        PersonEvent::AttributeRecorded(_) => "AttributeRecorded",
        PersonEvent::AttributeUpdated(_) => "AttributeUpdated",
        PersonEvent::AttributeInvalidated(_) => "AttributeInvalidated",
        PersonEvent::PersonDeactivated(_) => "PersonDeactivated",
        PersonEvent::PersonReactivated(_) => "PersonReactivated",
        PersonEvent::PersonMergedInto(_) => "PersonMergedInto",
    }
}
