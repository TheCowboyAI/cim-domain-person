//! Tests for adding attributes through proper event sourcing
//!
//! These tests demonstrate and verify the correct path for adding attributes:
//! 1. Create RecordAttribute command
//! 2. Process command (produces AttributeRecorded event)
//! 3. Apply event to update Person state (pure functional)

use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::{PersonCommand, RecordAttribute},
    value_objects::{
        PersonName, PersonAttribute, AttributeType, AttributeValue,
        IdentifyingAttributeType, PhysicalAttributeType, HealthcareAttributeType,
        TemporalValidity, Provenance, AttributeSource, ConfidenceLevel,
    },
};
use chrono::{Utc, NaiveDate};
use cim_domain::formal_domain::MealyStateMachine;

#[test]
fn test_add_single_attribute_via_command() {
    // Create person
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    assert_eq!(person.attributes.attributes.len(), 0);

    // Create attribute
    let attribute = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
    );

    // Create command
    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute,
    });

    // Process command
    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);

    assert_eq!(events.len(), 1);

    // Apply event
    for event in &events {
        person = person.apply_event_pure(event).unwrap();
    }

    assert_eq!(person.attributes.attributes.len(), 1);
    assert_eq!(person.version, 1); // Version incremented
}

#[test]
fn test_add_multiple_attributes_via_commands() {
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Add 3 different attributes
    let attributes = vec![
        PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        ),
        PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Weight),
            AttributeValue::Mass(70.0),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
        ),
        PersonAttribute::new(
            AttributeType::Healthcare(HealthcareAttributeType::BloodType),
            AttributeValue::Text("A+".to_string()),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        ),
    ];

    for attribute in attributes {
        let command = PersonCommand::RecordAttribute(RecordAttribute {
            person_id,
            attribute,
        });

        let current_state = person.lifecycle.clone();
        let events = MealyStateMachine::output(&person, current_state.into(), command);

        for event in &events {
            person = person.apply_event_pure(event).unwrap();
        }
    }

    assert_eq!(person.attributes.attributes.len(), 3);
    assert_eq!(person.version, 3); // Version incremented for each event
}

#[test]
fn test_cannot_add_attribute_to_deactivated_person() {
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Deactivate the person
    use cim_domain_person::commands::DeactivatePerson;
    let deactivate = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id,
        reason: "Test deactivation".to_string(),
    });

    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), deactivate);
    for event in &events {
        person = person.apply_event_pure(event).unwrap();
    }

    assert!(!person.is_active());

    // Try to add attribute (should produce no events)
    let attribute = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
    );

    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute,
    });

    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);

    assert_eq!(events.len(), 0); // No events produced for deactivated person
    assert_eq!(person.attributes.attributes.len(), 0); // No attributes added
}

#[test]
fn test_attribute_types_are_preserved() {
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Add attribute with specific type
    let original_attribute = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
    );

    let original_type = original_attribute.attribute_type.clone();

    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute: original_attribute,
    });

    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);

    for event in &events {
        person = person.apply_event_pure(event).unwrap();
    }

    // Verify the attribute type is preserved
    assert_eq!(person.attributes.attributes[0].attribute_type, original_type);
}

#[test]
fn test_provenance_is_preserved() {
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    // Add attribute with specific provenance
    let attribute = PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::BloodType),
        AttributeValue::Text("O+".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::Imported { system: "hospital_sys".to_string() },
            ConfidenceLevel::Certain
        ),
    );

    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute,
    });

    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);

    for event in &events {
        person = person.apply_event_pure(event).unwrap();
    }

    // Verify provenance is preserved
    let added_attr = &person.attributes.attributes[0];
    assert_eq!(added_attr.provenance.confidence, ConfidenceLevel::Certain);
    match &added_attr.provenance.source {
        AttributeSource::Imported { system } => {
            assert_eq!(system, "hospital_sys");
        }
        _ => panic!("Expected Imported source"),
    }
}

#[test]
fn test_version_increments_with_each_attribute() {
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name);

    let initial_version = person.version;

    // Add first attribute
    let attr1 = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
    );

    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute: attr1,
    });

    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);
    for event in &events {
        person = person.apply_event_pure(event).unwrap();
    }

    assert_eq!(person.version, initial_version + 1);

    // Add second attribute
    let attr2 = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(70.0),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
    );

    let command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute: attr2,
    });

    let current_state = person.lifecycle.clone();
    let events = MealyStateMachine::output(&person, current_state.into(), command);
    for event in &events {
        person = person.apply_event_pure(event).unwrap();
    }

    assert_eq!(person.version, initial_version + 2);
}
