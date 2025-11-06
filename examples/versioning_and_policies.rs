//! Event Versioning Example
//!
//! This example demonstrates event versioning concepts

use cim_domain_person::{
    aggregate::{Person, PersonId},
    events::{PersonEvent, PersonCreated},
    value_objects::PersonName,
};
use chrono::Utc;

fn main() {
    println!("=== Event Versioning Demo ===\n");

    demo_event_structure();

    println!("\n✅ Versioning demo completed!");
}

fn demo_event_structure() {
    println!("--- Event Structure ---");

    let person_id = PersonId::new();
    let name = PersonName::new("Alice".to_string(), "Johnson".to_string());

    // Create a PersonCreated event (current version)
    let event = PersonEvent::PersonCreated(PersonCreated {
        person_id,
        name: name.clone(),
        source: "api".to_string(),
        created_at: Utc::now(),
    });

    println!("PersonCreated event structure:");
    println!("  person_id: {}", person_id);
    println!("  name: {} {}",
        name.components.given_names[0],
        name.components.family_names[0]
    );
    println!("  source: api");
    println!("  created_at: {:?}", Utc::now());

    // Apply event
    let mut person = Person::empty();
    person.id = person_id;
    person = person.apply_event_pure(&event)
        .expect("Failed to apply event");

    println!("\nEvent successfully applied to aggregate");
    println!("Person state: {:?}", person.lifecycle);
}
