//! Simple example demonstrating the Person domain with EAV attributes
//!
//! This example shows:
//! - Creating a person with culturally-aware name parsing
//! - Recording attributes using the EAV (Entity-Attribute-Value) system
//! - Querying attributes by type and temporal validity
//! - Lifecycle management

use cim_domain_person::{
    aggregate::{Person, PersonId, PersonLifecycle},
    value_objects::{
        PersonName, PersonAttribute, AttributeType, AttributeValue,
        IdentifyingAttributeType, PhysicalAttributeType, HealthcareAttributeType,
        DemographicAttributeType, TemporalValidity, Provenance, AttributeSource,
        ConfidenceLevel,
    },
};
use chrono::{Utc, NaiveDate};

fn main() {
    println!("=== CIM Person Domain - Simple EAV Demo ===\n");

    // Create a person with name parsing
    demo_name_parsing();

    // Create a person and add attributes
    let person_id = PersonId::new();
    let name = PersonName::new("Alice".to_string(), "Johnson".to_string());
    let mut person = Person::new(person_id, name);

    println!("\n--- Person Created ---");
    println!("ID: {}", person_id);
    println!("Name: {}", person.core_identity.legal_name);
    println!("Lifecycle: {:?}\n", person.lifecycle);

    // Record various attributes
    demo_attribute_recording(&mut person);

    // Query attributes
    demo_attribute_queries(&person);

    // Demonstrate lifecycle
    demo_lifecycle(&mut person);
}

fn demo_name_parsing() {
    println!("--- Name Parsing Demo ---");

    // Parse a simple Western name
    let name1 = PersonName::parse("John Smith").unwrap();
    println!("Parsed: 'John Smith' → {} {}",
        name1.components.given_names[0],
        name1.components.family_names[0]
    );

    // Parse a name with particle
    let name2 = PersonName::parse("Ludwig van Beethoven").unwrap();
    println!("Parsed: 'Ludwig van Beethoven' → {} {} {}",
        name2.components.given_names[0],
        name2.components.prefixes[0],
        name2.components.family_names[0]
    );

    // Parse a Spanish name
    let name3 = PersonName::parse("Pablo Ruiz y Picasso").unwrap();
    println!("Parsed: 'Pablo Ruiz y Picasso' → {} (paternal: {}, maternal: {})",
        name3.components.given_names[0],
        name3.components.family_names[0],
        name3.components.family_names[1]
    );

    // Parse an East Asian name
    let name4 = PersonName::parse("山田太郎").unwrap();
    println!("Parsed: '山田太郎' → Convention: {:?}", name4.naming_convention);
}

fn demo_attribute_recording(person: &mut Person) {
    println!("\n--- Recording Attributes ---");

    // Record birth date (identifying attribute)
    let birth_date_attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1985, 3, 15).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::DocumentVerified,
            ConfidenceLevel::Certain,
        ),
    );
    person.attributes.attributes.push(birth_date_attr);
    println!("✓ Recorded birth date (from official document)");

    // Record height (physical attribute)
    let height_attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.68), // in meters
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Likely,
        ),
    );
    person.attributes.attributes.push(height_attr);
    println!("✓ Recorded height (self-reported)");

    // Record blood type (healthcare attribute)
    let blood_type_attr = PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::BloodType),
        AttributeValue::Text("A+".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::Imported { system: "hospital".to_string() },
            ConfidenceLevel::Certain,
        ),
    );
    person.attributes.attributes.push(blood_type_attr);
    println!("✓ Recorded blood type (from hospital system)");

    // Record nationality (demographic attribute)
    let nationality_attr = PersonAttribute::new(
        AttributeType::Demographic(DemographicAttributeType::Nationality),
        AttributeValue::Text("US".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::DocumentVerified,
            ConfidenceLevel::Certain,
        ),
    );
    person.attributes.attributes.push(nationality_attr);
    println!("✓ Recorded nationality (from passport)");

    println!("\nTotal attributes: {}", person.attributes.attributes.len());
}

fn demo_attribute_queries(person: &Person) {
    println!("\n--- Querying Attributes ---");

    // Get identifying attributes
    let identifying = person.attributes.identifying_attributes();
    println!("\nIdentifying attributes: {}", identifying.attributes.len());
    for attr in &identifying.attributes {
        match &attr.value {
            AttributeValue::Date(d) => println!("  - Birth date: {}", d),
            _ => {}
        }
    }

    // Get healthcare attributes
    let healthcare = person.attributes.healthcare_attributes();
    println!("\nHealthcare attributes: {}", healthcare.attributes.len());
    for attr in &healthcare.attributes {
        match &attr.value {
            AttributeValue::Text(t) => {
                if matches!(attr.attribute_type, AttributeType::Healthcare(HealthcareAttributeType::BloodType)) {
                    println!("  - Blood type: {}", t);
                }
            }
            _ => {}
        }
    }

    // Get currently valid attributes
    let current = person.attributes.currently_valid();
    println!("\nCurrently valid attributes: {}", current.attributes.len());

    // Get high confidence attributes
    let high_confidence: Vec<_> = person.attributes.attributes.iter()
        .filter(|attr| attr.provenance.confidence == ConfidenceLevel::Certain)
        .collect();
    println!("High confidence attributes: {}", high_confidence.len());
}

fn demo_lifecycle(person: &mut Person) {
    println!("\n--- Lifecycle Management ---");

    println!("Current state: {:?}", person.lifecycle);
    println!("Is active: {}", person.is_active());

    // Deactivate person
    println!("\nDeactivating person...");
    person.lifecycle = PersonLifecycle::Deactivated {
        reason: "Account suspended for review".to_string(),
        since: Utc::now(),
    };
    println!("Current state: {:?}", person.lifecycle);
    println!("Is active: {}", person.is_active());

    // Reactivate
    println!("\nReactivating person...");
    person.lifecycle = PersonLifecycle::Active;
    println!("Current state: {:?}", person.lifecycle);
    println!("Is active: {}", person.is_active());

    println!("\n✓ Example completed successfully!");
}
