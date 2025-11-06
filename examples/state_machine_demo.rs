//! State Machine Demo
//!
//! This example demonstrates lifecycle state transitions using the Person aggregate.
//! Shows the different states a person can be in (Active, Deactivated, MergedInto, Deceased)

use cim_domain_person::{
    aggregate::{Person, PersonId, PersonLifecycle},
    events::{PersonEvent, PersonCreated, PersonDeactivated, PersonReactivated, DeathRecorded},
    value_objects::PersonName,
};
use chrono::{Utc, NaiveDate};

fn main() {
    println!("=== Person Lifecycle State Machine Demo ===\n");

    demo_basic_lifecycle();
    demo_merge_workflow();
    demo_death_workflow();

    println!("\n✅ State machine demo completed!");
}

fn demo_basic_lifecycle() {
    println!("--- Basic Lifecycle State Transitions ---");

    let person_id = PersonId::new();
    let name = PersonName::new("Alice".to_string(), "Smith".to_string());

    // Create event
    let create_event = PersonEvent::PersonCreated(PersonCreated {
        person_id,
        name: name.clone(),
        source: "demo".to_string(),
        created_at: Utc::now(),
    });

    // Apply event to create person
    let mut person = Person::empty();
    person.id = person_id;
    person = person.apply_event_pure(&create_event)
        .expect("Failed to create person");

    println!("Initial state: {:?}", person.lifecycle);
    assert!(matches!(person.lifecycle, PersonLifecycle::Active));

    // Transition: Active → Deactivated
    let deactivate_event = PersonEvent::PersonDeactivated(PersonDeactivated {
        person_id,
        reason: "Account suspended".to_string(),
        deactivated_at: Utc::now(),
    });

    person = person.apply_event_pure(&deactivate_event)
        .expect("Failed to deactivate");

    println!("After deactivation: {:?}", person.lifecycle);
    assert!(matches!(person.lifecycle, PersonLifecycle::Deactivated { .. }));
    assert!(!person.is_active());

    // Transition: Deactivated → Active
    let reactivate_event = PersonEvent::PersonReactivated(PersonReactivated {
        person_id,
        reason: "Review completed".to_string(),
        reactivated_at: Utc::now(),
    });

    person = person.apply_event_pure(&reactivate_event)
        .expect("Failed to reactivate");

    println!("After reactivation: {:?}", person.lifecycle);
    assert!(matches!(person.lifecycle, PersonLifecycle::Active));
    assert!(person.is_active());

    println!();
}

fn demo_merge_workflow() {
    println!("--- Merge Workflow ---");

    // Create two persons
    let person1_id = PersonId::new();
    let person2_id = PersonId::new();

    let name1 = PersonName::new("Bob".to_string(), "Jones".to_string());
    let mut person1 = Person::new(person1_id, name1);

    println!("Person 1 (source): {} - {:?}", person1_id, person1.lifecycle);

    // Merge person1 into person2
    use cim_domain_person::events::PersonMergedInto;
    use cim_domain_person::commands::MergeReason;

    let merge_event = PersonEvent::PersonMergedInto(PersonMergedInto {
        source_person_id: person1_id,
        merged_into_id: person2_id,
        merge_reason: MergeReason::DuplicateIdentity,
        merged_at: Utc::now(),
    });

    person1 = person1.apply_event_pure(&merge_event)
        .expect("Failed to merge");

    println!("After merge: {:?}", person1.lifecycle);
    assert!(matches!(person1.lifecycle, PersonLifecycle::MergedInto { .. }));

    println!();
}

fn demo_death_workflow() {
    println!("--- Death Recording Workflow ---");

    let person_id = PersonId::new();
    let name = PersonName::new("Charlie".to_string(), "Brown".to_string());
    let mut person = Person::new(person_id, name);

    println!("Initial state: {:?}", person.lifecycle);

    // Record death
    let death_event = PersonEvent::DeathRecorded(DeathRecorded {
        person_id,
        date_of_death: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&death_event)
        .expect("Failed to record death");

    println!("After death recorded: {:?}", person.lifecycle);
    assert!(matches!(person.lifecycle, PersonLifecycle::Deceased { .. }));

    println!();
}
