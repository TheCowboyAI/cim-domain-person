//! Adding Attributes Example
//!
//! This example demonstrates the proper way to add attributes to a Person
//! using the event sourcing pattern with commands and events.
//!
//! # The Correct Path for Adding Attributes
//!
//! 1. Create a RecordAttribute command
//! 2. Handle the command (produces AttributeRecorded event)
//! 3. Apply the event to update the Person state
//!
//! This follows pure functional event sourcing - NO direct mutation!

use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::{PersonCommand, RecordAttribute},
    value_objects::{
        PersonName, PersonAttribute, AttributeType, AttributeValue,
        IdentifyingAttributeType, PhysicalAttributeType, HealthcareAttributeType,
        DemographicAttributeType, TemporalValidity, Provenance,
        AttributeSource, ConfidenceLevel,
    },
};
use chrono::{Utc, NaiveDate};
use cim_domain::formal_domain::MealyStateMachine;

fn main() {
    println!("=== Adding Attributes to Person (Event Sourcing Pattern) ===\n");

    demo_add_single_attribute();
    demo_add_multiple_attributes();
    demo_complete_workflow();

    println!("\n✅ All attribute addition examples completed!");
}

/// Demonstrate adding a single attribute
fn demo_add_single_attribute() {
    println!("--- Adding a Single Attribute ---");

    // Step 1: Create a person
    let person_id = PersonId::new();
    let name = PersonName::new("Alice".to_string(), "Johnson".to_string());
    let mut person = Person::new(person_id, name);

    println!("Initial person created");
    println!("  ID: {}", person.id);
    println!("  Attributes: {}", person.attributes.attributes.len());

    // Step 2: Create the attribute to add
    let birth_date = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
    );

    // Step 3: Create RecordAttribute command
    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute: birth_date,
    });

    // Step 4: Process command through state machine (produces events)
    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);

    println!("\nCommand processed, events produced: {}", events.len());

    // Step 5: Apply events to update person (pure functional)
    for event in &events {
        person = person.apply_event_pure(event)
            .expect("Failed to apply event");
    }

    println!("After applying event:");
    println!("  Attributes: {}", person.attributes.attributes.len());
    println!("  Version: {}", person.version);

    // Verify the attribute was added
    let identifying = person.identifying_attributes();
    assert_eq!(identifying.attributes.len(), 1);
    println!("  ✓ Birth date attribute successfully added");

    println!();
}

/// Demonstrate adding multiple attributes
fn demo_add_multiple_attributes() {
    println!("--- Adding Multiple Attributes ---");

    let person_id = PersonId::new();
    let name = PersonName::new("Bob".to_string(), "Smith".to_string());
    let mut person = Person::new(person_id, name);

    // List of attributes to add
    let attributes_to_add = vec![
        PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        ),
        PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Weight),
            AttributeValue::Mass(75.0),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
        ),
        PersonAttribute::new(
            AttributeType::Healthcare(HealthcareAttributeType::BloodType),
            AttributeValue::Text("O+".to_string()),
            TemporalValidity::of(Utc::now()),
            Provenance::new(
                AttributeSource::Imported { system: "hospital".to_string() },
                ConfidenceLevel::Certain
            ),
        ),
    ];

    println!("Adding {} attributes...", attributes_to_add.len());

    // Process each attribute addition as a separate command/event cycle
    for (i, attribute) in attributes_to_add.into_iter().enumerate() {
        let command = PersonCommand::RecordAttribute(RecordAttribute {
            person_id,
            attribute,
        });

        let current_state = person.lifecycle.clone();
        let events = MealyStateMachine::output(&person, current_state.into(), command);

        for event in &events {
            person = person.apply_event_pure(event)
                .expect("Failed to apply event");
        }

        println!("  {} - Attribute added (total: {})", i + 1, person.attributes.attributes.len());
    }

    println!("\nFinal state:");
    println!("  Total attributes: {}", person.attributes.attributes.len());
    println!("  Physical attributes: {}", person.attributes.physical_attributes().attributes.len());
    println!("  Healthcare attributes: {}", person.attributes.healthcare_attributes().attributes.len());
    println!("  Version: {}", person.version);

    println!();
}

/// Demonstrate complete workflow with validation
fn demo_complete_workflow() {
    println!("--- Complete Workflow with Validation ---");

    let person_id = PersonId::new();
    let name = PersonName::new("Charlie".to_string(), "Brown".to_string());
    let mut person = Person::new(person_id, name);

    // Add birth date
    let birth_date_attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1985, 3, 15).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
    );

    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute: birth_date_attr,
    });

    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);

    for event in &events {
        person = person.apply_event_pure(event).expect("Failed to apply event");
    }

    println!("Added birth date");

    // Add nationality
    let nationality_attr = PersonAttribute::new(
        AttributeType::Demographic(DemographicAttributeType::Nationality),
        AttributeValue::Text("USA".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
    );

    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute: nationality_attr,
    });

    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);

    for event in &events {
        person = person.apply_event_pure(event).expect("Failed to apply event");
    }

    println!("Added nationality");

    // Verify all attributes
    println!("\nVerification:");
    println!("  Total attributes: {}", person.attributes.attributes.len());

    let identifying = person.identifying_attributes();
    println!("  Identifying attributes: {}", identifying.attributes.len());

    let demographic = person.attributes.demographic_attributes();
    println!("  Demographic attributes: {}", demographic.attributes.len());

    // Verify we can query attributes
    for (i, attr) in person.attributes.attributes.iter().enumerate() {
        println!("  Attribute {}: {:?}", i + 1, attr.attribute_type);
    }

    println!("\n✓ Complete workflow executed successfully");
}
