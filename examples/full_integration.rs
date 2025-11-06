//! Full Integration Example
//!
//! This example demonstrates a complete workflow using:
//! - Person creation
//! - Attribute management
//! - Lifecycle transitions
//! - Event sourcing patterns

use cim_domain_person::{
    aggregate::{Person, PersonId},
    events::{PersonEvent, PersonCreated, AttributeRecorded},
    value_objects::{
        PersonName, PersonAttribute, AttributeType, AttributeValue,
        IdentifyingAttributeType, PhysicalAttributeType, HealthcareAttributeType,
        DemographicAttributeType, TemporalValidity, Provenance, AttributeSource,
        ConfidenceLevel,
    },
};
use chrono::{Utc, NaiveDate};

fn main() {
    println!("=== Full Integration Demo ===\n");

    demo_complete_workflow();

    println!("\n✅ Integration demo completed!");
}

fn demo_complete_workflow() {
    println!("--- Complete Person Management Workflow ---");

    // Step 1: Create person
    let person_id = PersonId::new();
    let name = PersonName::parse("María García López").unwrap();

    println!("Creating person with parsed name...");
    println!("  Convention: {:?}", name.naming_convention);
    println!("  Given names: {:?}", name.components.given_names);
    println!("  Family names: {:?}", name.components.family_names);

    let create_event = PersonEvent::PersonCreated(PersonCreated {
        person_id,
        name: name.clone(),
        source: "registration".to_string(),
        created_at: Utc::now(),
    });

    let mut person = Person::empty();
    person.id = person_id;
    person = person.apply_event_pure(&create_event)
        .expect("Failed to create person");

    println!("\n✓ Person created");

    // Step 2: Record identifying attributes
    let birth_date_event = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: PersonAttribute::new(
            AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
            AttributeValue::Date(NaiveDate::from_ymd_opt(1985, 3, 15).unwrap()),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
        ),
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&birth_date_event)
        .expect("Failed to record birth date");

    println!("✓ Birth date recorded");

    // Step 3: Record physical attributes
    let height_event = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.65),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        ),
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&height_event)
        .expect("Failed to record height");

    println!("✓ Height recorded");

    // Step 4: Record healthcare attributes
    let blood_type_event = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: PersonAttribute::new(
            AttributeType::Healthcare(HealthcareAttributeType::BloodType),
            AttributeValue::Text("O+".to_string()),
            TemporalValidity::of(Utc::now()),
            Provenance::new(
                AttributeSource::Imported { system: "hospital".to_string() },
                ConfidenceLevel::Certain,
            ),
        ),
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&blood_type_event)
        .expect("Failed to record blood type");

    println!("✓ Blood type recorded");

    // Step 5: Record demographic attributes
    let nationality_event = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: PersonAttribute::new(
            AttributeType::Demographic(DemographicAttributeType::Nationality),
            AttributeValue::Text("Spain".to_string()),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
        ),
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&nationality_event)
        .expect("Failed to record nationality");

    println!("✓ Nationality recorded");

    // Step 6: Query attributes
    println!("\n--- Querying Attributes ---");
    println!("Total attributes: {}", person.attributes.attributes.len());

    let identifying = person.attributes.identifying_attributes();
    println!("Identifying attributes: {}", identifying.attributes.len());

    let healthcare = person.attributes.healthcare_attributes();
    println!("Healthcare attributes: {}", healthcare.attributes.len());

    let high_confidence: Vec<_> = person.attributes.attributes.iter()
        .filter(|attr| attr.provenance.confidence == ConfidenceLevel::Certain)
        .collect();
    println!("High confidence attributes: {}", high_confidence.len());

    // Step 7: Display final state
    println!("\n--- Final State ---");
    println!("Person ID: {}", person.id);
    println!("Name: {}", person.core_identity.legal_name);
    println!("Lifecycle: {:?}", person.lifecycle);
    println!("Total attributes: {}", person.attributes.attributes.len());
}
