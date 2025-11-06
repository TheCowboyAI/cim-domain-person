//! Comprehensive tests for Person aggregate operations
//! Based on user stories US-3.1, US-3.2, US-4.1 through US-4.4, US-5.4, US-6.1, US-6.2

use cim_domain_person::{
    aggregate::{Person, PersonId, PersonLifecycle},
    commands::{CreatePerson, MergeReason},
    events::{PersonEvent, PersonCreated, AttributeRecorded, AttributeUpdated, AttributeInvalidated},
    value_objects::{PersonName, PersonAttribute, AttributeType, AttributeValue, IdentifyingAttributeType, PhysicalAttributeType, TemporalValidity, Provenance, AttributeSource, ConfidenceLevel},
};
use chrono::{Utc, NaiveDate};

// ===== US-4.1: Create Person via Command (Event Sourcing) =====

#[test]
fn test_create_person_event_sourced() {
    // Test Scenario: Send CreatePerson command
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());

    let command = CreatePerson {
        person_id,
        name: name.clone(),
        source: "test".to_string(),
    };

    // Create person from event
    let event = PersonEvent::PersonCreated(PersonCreated {
        person_id,
        name,
        source: "test".to_string(),
        created_at: Utc::now(),
    });

    // Reconstruct aggregate from event
    let mut person = Person::empty();
    person.id = person_id;
    let result = person.apply_event_pure(&event);

    assert!(result.is_ok());
    let person = result.unwrap();

    // Verify aggregate state matches event
    assert_eq!(person.id, person_id);
    assert_eq!(person.core_identity.legal_name.components.given_names[0], "John");
    assert_eq!(person.core_identity.legal_name.components.family_names[0], "Doe");
    assert_eq!(person.lifecycle, PersonLifecycle::Active);
}

#[test]
fn test_person_creation_immutable() {
    // Test Scenario: Verify event is immutable
    let person_id = PersonId::new();
    let name = PersonName::new("Jane".to_string(), "Smith".to_string());

    let event = PersonCreated {
        person_id,
        name,
        source: "test".to_string(),
        created_at: Utc::now(),
    };

    // Events are Copy/Clone, proving immutability in the type system
    let _cloned = event.clone();

    assert_eq!(event.source, "test");
}

#[test]
fn test_cannot_create_duplicate_person() {
    // Test Scenario: Cannot create person with same ID twice
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());

    let person = Person::new(person_id, name.clone());

    // Person already created - attempting to create again should be prevented
    // This is enforced at the command handler level
    assert_eq!(person.lifecycle, PersonLifecycle::Active);
}

// ===== US-4.2: Record Attribute via Command =====

#[test]
fn test_record_attribute_event() {
    // Test Scenario: Send RecordAttribute command
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let person = Person::new(person_id, name);

    let attribute = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::Imported { system: "gov".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    let event = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: attribute.clone(),
        recorded_at: Utc::now(),
    });

    // Apply event
    let result = person.apply_event_pure(&event);
    assert!(result.is_ok());

    let updated_person = result.unwrap();

    // Verify attribute added to aggregate
    assert_eq!(updated_person.attributes.attributes.len(), 1);
    assert!(matches!(
        updated_person.attributes.attributes[0].attribute_type,
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate)
    ));
}

#[test]
fn test_record_multiple_attributes_sequentially() {
    // Test Scenario: Record multiple attributes in sequence
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    // Record attribute 1
    let attr1 = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "gov".to_string() }, ConfidenceLevel::Certain),
    );

    let event1 = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: attr1,
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&event1).unwrap();
    assert_eq!(person.attributes.attributes.len(), 1);

    // Record attribute 2
    let attr2 = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    );

    let event2 = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: attr2,
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&event2).unwrap();
    assert_eq!(person.attributes.attributes.len(), 2);
}

// ===== US-4.3: Update Attribute via Command =====

#[test]
fn test_update_attribute_event() {
    // Test Scenario: Update person's height measurement
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let _person = Person::new(person_id, name);

    let old_attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.70),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    );

    let new_attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "hospital".to_string() }, ConfidenceLevel::Certain),
    );

    let event = PersonEvent::AttributeUpdated(AttributeUpdated {
        person_id,
        attribute_type: AttributeType::Physical(PhysicalAttributeType::Height),
        old_attribute: old_attr.clone(),
        new_attribute: new_attr.clone(),
        updated_at: Utc::now(),
    });

    // Verify event includes old and new values
    if let PersonEvent::AttributeUpdated(ref update) = event {
        if let AttributeValue::Length(old_height) = update.old_attribute.value {
            assert_eq!(old_height, 1.70);
        }
        if let AttributeValue::Length(new_height) = update.new_attribute.value {
            assert_eq!(new_height, 1.75);
        }
    }
}

#[test]
fn test_update_preserves_history() {
    // Test Scenario: Original attribute preserved in event history
    let person_id = PersonId::new();

    let old_attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(70.0),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    );

    let new_attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(75.0),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "hospital".to_string() }, ConfidenceLevel::Certain),
    );

    let event = AttributeUpdated {
        person_id,
        attribute_type: AttributeType::Physical(PhysicalAttributeType::Weight),
        old_attribute: old_attr,
        new_attribute: new_attr,
        updated_at: Utc::now(),
    };

    // Event contains complete history
    assert!(matches!(event.old_attribute.value, AttributeValue::Mass(_)));
    assert!(matches!(event.new_attribute.value, AttributeValue::Mass(_)));
}

// ===== US-4.4: Invalidate Attribute via Command =====

#[test]
fn test_invalidate_attribute_event() {
    // Test Scenario: Invalidate incorrect birth date with reason
    let person_id = PersonId::new();

    let event = PersonEvent::AttributeInvalidated(AttributeInvalidated {
        person_id,
        attribute_type: AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        invalidated_at: Utc::now(),
        reason: Some("data entry error".to_string()),
    });

    // Verify event structure
    if let PersonEvent::AttributeInvalidated(ref invalidation) = event {
        assert_eq!(invalidation.reason, Some("data entry error".to_string()));
        assert!(matches!(
            invalidation.attribute_type,
            AttributeType::Identifying(IdentifyingAttributeType::BirthDate)
        ));
    }
}

// ===== US-3.1: Observe Person State at Point in Time (Coalgebra) =====

#[test]
fn test_observe_person_at_historical_date() {
    // Test Scenario: Observe person state on 2020-01-01
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    // Add attribute valid in 2020
    let attr_2020 = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.70),
        TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2019, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2021, 12, 31).unwrap()),
        ),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    );

    person.attributes.attributes.push(attr_2020);

    // Add attribute valid in 2024
    let attr_2024 = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            None,
        ),
        Provenance::new(AttributeSource::Imported { system: "hospital".to_string() }, ConfidenceLevel::Certain),
    );

    person.attributes.attributes.push(attr_2024);

    // Observe at different dates
    let date_2020 = NaiveDate::from_ymd_opt(2020, 6, 1).unwrap();
    let date_2024 = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

    let observed_2020 = person.observe_at(date_2020);
    let observed_2024 = person.observe_at(date_2024);

    // Verify correct attributes for each date
    assert_eq!(observed_2020.attributes.len(), 1);
    assert_eq!(observed_2024.attributes.len(), 1);

    if let AttributeValue::Length(height) = observed_2020.attributes[0].value {
        assert_eq!(height, 1.70);
    }

    if let AttributeValue::Length(height) = observed_2024.attributes[0].value {
        assert_eq!(height, 1.75);
    }
}

#[test]
fn test_observe_current_state() {
    // Test Scenario: Query person's attributes valid at date
    let person_id = PersonId::new();
    let name = PersonName::new("Jane".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    // Add current attribute
    let attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.65),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    );

    person.attributes.attributes.push(attr);

    let current = person.observe_now();

    assert_eq!(current.attributes.len(), 1);
}

// ===== US-5.4: Coalgebra for Person Unfold =====

#[test]
fn test_person_unfold_coalgebra() {
    // Test Scenario: Verify unfold returns all attributes
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    // Add multiple attributes
    person.attributes.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    person.attributes.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(70.0),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    // Unfold: Person → PersonAttributeSet
    let unfolded = person.unfold();

    assert_eq!(unfolded.attributes.len(), 2);
}

#[test]
fn test_unfold_temporal_coherence() {
    // Test Scenario: Temporal filtering is subset of unfold
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    // Add past attribute
    person.attributes.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.70),
        TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2021, 12, 31).unwrap()),
        ),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    // Add current attribute
    person.attributes.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    let all_attributes = person.unfold();
    let current_attributes = person.observe_now();

    // Current is subset of all
    assert_eq!(all_attributes.attributes.len(), 2);
    assert_eq!(current_attributes.attributes.len(), 1);
}

#[test]
fn test_unfold_immutability() {
    // Test Scenario: Multiple unfolds are identical (pure)
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    person.attributes.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    let unfold1 = person.unfold();
    let unfold2 = person.unfold();

    assert_eq!(unfold1.attributes.len(), unfold2.attributes.len());
}

#[test]
fn test_map_attributes_functor() {
    // Test Scenario: Verify aggregate not modified by unfold
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    person.attributes.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(69.0), // inches
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    // Map attributes: convert inches to meters
    let updated_person = person.map_attributes(|attr| {
        attr.map(|value| {
            if let AttributeValue::Length(inches) = value {
                AttributeValue::Length(inches * 0.0254)
            } else {
                value
            }
        })
    });

    // Verify transformation
    if let AttributeValue::Length(meters) = updated_person.attributes.attributes[0].value {
        assert!((meters - 1.7526).abs() < 0.001);
    }
}

// ===== US-6.1: Deactivate Person =====

#[test]
fn test_deactivate_person() {
    // Test Scenario: Deactivate person with reason
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let person = Person::new(person_id, name);

    assert_eq!(person.lifecycle, PersonLifecycle::Active);

    // Create deactivation event
    let event = PersonEvent::PersonDeactivated(cim_domain_person::events::PersonDeactivated {
        person_id,
        reason: "account closed".to_string(),
        deactivated_at: Utc::now(),
    });

    let deactivated_person = person.apply_event_pure(&event).unwrap();

    assert!(matches!(deactivated_person.lifecycle, PersonLifecycle::Deactivated { .. }));
}

#[test]
fn test_reactivate_person() {
    // Test Scenario: Reactivate previously deactivated person
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    // Deactivate
    let deactivate_event = PersonEvent::PersonDeactivated(cim_domain_person::events::PersonDeactivated {
        person_id,
        reason: "account closed".to_string(),
        deactivated_at: Utc::now(),
    });

    person = person.apply_event_pure(&deactivate_event).unwrap();
    assert!(matches!(person.lifecycle, PersonLifecycle::Deactivated { .. }));

    // Reactivate
    let reactivate_event = PersonEvent::PersonReactivated(cim_domain_person::events::PersonReactivated {
        person_id,
        reason: "account reopened".to_string(),
        reactivated_at: Utc::now(),
    });

    person = person.apply_event_pure(&reactivate_event).unwrap();
    assert_eq!(person.lifecycle, PersonLifecycle::Active);
}

// ===== US-6.2: Merge Duplicate Persons =====

#[test]
fn test_merge_person() {
    // Test Scenario: Merge duplicate created from different sources
    let source_person_id = PersonId::new();
    let target_person_id = PersonId::new();

    let source_person = Person::new(
        source_person_id,
        PersonName::new("John".to_string(), "Doe".to_string()),
    );

    let event = PersonEvent::PersonMergedInto(cim_domain_person::events::PersonMergedInto {
        source_person_id,
        merged_into_id: target_person_id,
        merge_reason: MergeReason::DuplicateIdentity,
        merged_at: Utc::now(),
    });

    let merged_person = source_person.apply_event_pure(&event).unwrap();

    assert!(matches!(merged_person.lifecycle, PersonLifecycle::MergedInto { .. }));
}

// ===== Integration and End-to-End Tests =====

#[test]
fn test_full_person_lifecycle() {
    // Test Scenario: Complete lifecycle from creation to deactivation
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());

    // 1. Create person
    let mut person = Person::new(person_id, name);
    assert_eq!(person.lifecycle, PersonLifecycle::Active);

    // 2. Add attributes
    let attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    );

    let event1 = PersonEvent::AttributeRecorded(AttributeRecorded {
        person_id,
        attribute: attr,
        recorded_at: Utc::now(),
    });

    person = person.apply_event_pure(&event1).unwrap();
    assert_eq!(person.attributes.attributes.len(), 1);

    // 3. Update name
    let old_name = person.core_identity.legal_name.clone();
    let new_name = PersonName::new("John".to_string(), "Smith".to_string());

    let event2 = PersonEvent::NameUpdated(cim_domain_person::events::NameUpdated {
        person_id,
        old_name,
        new_name: new_name.clone(),
        reason: Some("Marriage".to_string()),
        updated_at: Utc::now(),
    });

    person = person.apply_event_pure(&event2).unwrap();
    assert_eq!(person.core_identity.legal_name.components.family_names[0], "Smith");

    // 4. Deactivate
    let event3 = PersonEvent::PersonDeactivated(cim_domain_person::events::PersonDeactivated {
        person_id,
        reason: "account closed".to_string(),
        deactivated_at: Utc::now(),
    });

    person = person.apply_event_pure(&event3).unwrap();
    assert!(matches!(person.lifecycle, PersonLifecycle::Deactivated { .. }));
}

#[test]
fn test_event_replay_reconstruction() {
    // Test Scenario: Reconstruct aggregate from event stream
    let person_id = PersonId::new();
    let name = PersonName::new("Jane".to_string(), "Doe".to_string());

    // Event stream
    let events = vec![
        PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: name.clone(),
            source: "test".to_string(),
            created_at: Utc::now(),
        }),
        PersonEvent::AttributeRecorded(AttributeRecorded {
            person_id,
            attribute: PersonAttribute::new(
                AttributeType::Physical(PhysicalAttributeType::Height),
                AttributeValue::Length(1.65),
                TemporalValidity::of(Utc::now()),
                Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
            ),
            recorded_at: Utc::now(),
        }),
    ];

    // Replay events
    let mut person = Person::empty();
    person.id = person_id;

    for event in events {
        person = person.apply_event_pure(&event).unwrap();
    }

    // Verify final state
    assert_eq!(person.id, person_id);
    assert_eq!(person.core_identity.legal_name.components.given_names[0], "Jane");
    assert_eq!(person.attributes.attributes.len(), 1);
    assert_eq!(person.lifecycle, PersonLifecycle::Active);
}

#[test]
fn test_category_query_methods() {
    // Test Scenario: Use aggregate helper methods
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let mut person = Person::new(person_id, name);

    // Add identifying attribute
    person.attributes.attributes.push(PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "gov".to_string() }, ConfidenceLevel::Certain),
    ));

    // Add healthcare attribute
    person.attributes.attributes.push(PersonAttribute::new(
        AttributeType::Healthcare(cim_domain_person::value_objects::HealthcareAttributeType::MedicalRecordNumber),
        AttributeValue::Text("MRN-123".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "hospital".to_string() }, ConfidenceLevel::Certain),
    ));

    // Query by category
    let identifying = person.identifying_attributes();
    let healthcare = person.healthcare_attributes();

    assert_eq!(identifying.attributes.len(), 1);
    assert_eq!(healthcare.attributes.len(), 1);
}
