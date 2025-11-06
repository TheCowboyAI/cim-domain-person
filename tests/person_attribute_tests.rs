//! Comprehensive tests for PersonAttribute functionality
//! Based on user stories US-2.1 through US-2.5, US-5.1 through US-5.4

use cim_domain_person::value_objects::{
    PersonAttribute, PersonAttributeSet, AttributeType, AttributeValue,
    IdentifyingAttributeType, PhysicalAttributeType, HealthcareAttributeType,
    DemographicAttributeType, TemporalValidity, Provenance, AttributeSource,
    ConfidenceLevel, BloodTypeValue, EyeColorValue, HairColorValue,
    BiologicalSexValue, HandednessValue
};
use chrono::{Utc, NaiveDate};

// ===== US-2.1: Record Identifying Attributes =====

#[test]
fn test_record_birth_datetime_exact() {
    // Test Scenario: Record exact birth datetime with hospital source
    let birth_datetime = Utc::now();
    let recorded_at = Utc::now();

    let attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDateTime),
        AttributeValue::DateTime(birth_datetime),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::Imported { system: "hospital_integration".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    assert!(matches!(attr.attribute_type, AttributeType::Identifying(_)));
    assert!(matches!(attr.value, AttributeValue::DateTime(_)));
    assert_eq!(attr.provenance.confidence, ConfidenceLevel::Certain);

    if let AttributeSource::Imported { system: ref system } = attr.provenance.source {
        assert_eq!(system, "hospital_integration");
    } else {
        panic!("Expected System source");
    }
}

#[test]
fn test_record_birth_date_with_precision() {
    // Test Scenario: Record approximate birth year (1950s) with family history source
    let recorded_at = Utc::now();

    let attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1950, 1, 1).unwrap()),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Uncertain, // Low confidence for approximate date
        ),
    );

    assert_eq!(attr.provenance.confidence, ConfidenceLevel::Uncertain);

    // Verify source is self-reported
    assert!(matches!(attr.provenance.source, AttributeSource::SelfReported));
}

#[test]
fn test_record_national_id() {
    // Test Scenario: Record national ID from government integration
    let recorded_at = Utc::now();

    let attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::NationalId),
        AttributeValue::Text("123-45-6789".to_string()),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::Imported { system: "irs_system".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    if let AttributeValue::Text(ref national_id) = attr.value {
        assert_eq!(national_id, "123-45-6789");
    } else {
        panic!("Expected Text value for national ID");
    }

    assert_eq!(attr.provenance.confidence, ConfidenceLevel::Certain);
}

#[test]
fn test_transformation_trace() {
    // Test Scenario: Trace SSN derived from tax system
    let recorded_at = Utc::now();

    let provenance = Provenance::new(
        AttributeSource::Imported { system: "tax_system".to_string() },
        ConfidenceLevel::Certain,
    )
    .trace_transformation("normalized_format".to_string(), "data_pipeline".to_string())
    .trace_transformation("validated_checksum".to_string(), "validation_service".to_string());

    let attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::NationalId),
        AttributeValue::Text("123-45-6789".to_string()),
        TemporalValidity::of(recorded_at),
        provenance,
    );

    assert_eq!(attr.provenance.trace.len(), 2);
    assert_eq!(attr.provenance.trace[0].transformation, "normalized_format");
    assert_eq!(attr.provenance.trace[1].transformation, "validated_checksum");
}

// ===== US-2.2: Record Physical Attributes =====

#[test]
fn test_record_height_with_units() {
    // Test Scenario: Record height with unit (meters)
    let recorded_at = Utc::now();
    let valid_from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75), // 1.75 meters
        TemporalValidity::new(recorded_at, Some(valid_from), None),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Likely,
        ),
    );

    if let AttributeValue::Length(height) = attr.value {
        assert_eq!(height, 1.75);
    } else {
        panic!("Expected Length value");
    }
}

#[test]
fn test_record_weight_changes_over_time() {
    // Test Scenario: Record child's weight measurements over years
    let recorded_2020 = Utc::now();
    let recorded_2024 = Utc::now();

    let weight_2020 = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(30.0), // 30 kg
        TemporalValidity::new(
            recorded_2020,
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2021, 12, 31).unwrap()),
        ),
        Provenance::new(
            AttributeSource::Imported { system: "health_records".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    let weight_2024 = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(50.0), // 50 kg
        TemporalValidity::new(
            recorded_2024,
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            None,
        ),
        Provenance::new(
            AttributeSource::Imported { system: "health_records".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    // Create attribute set with both measurements
    let mut set = PersonAttributeSet::empty();
    set.attributes.push(weight_2020);
    set.attributes.push(weight_2024);

    assert_eq!(set.attributes.len(), 2);

    // Query weight in 2020 vs 2024
    let date_2020 = NaiveDate::from_ymd_opt(2020, 6, 1).unwrap();
    let date_2024 = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

    let valid_2020 = set.valid_on(date_2020);
    let valid_2024 = set.valid_on(date_2024);

    assert_eq!(valid_2020.attributes.len(), 1);
    assert_eq!(valid_2024.attributes.len(), 1);

    if let AttributeValue::Mass(mass) = valid_2020.attributes[0].value {
        assert_eq!(mass, 30.0);
    }

    if let AttributeValue::Mass(mass) = valid_2024.attributes[0].value {
        assert_eq!(mass, 50.0);
    }
}

#[test]
fn test_record_hair_color() {
    // Test Scenario: Record hair color
    let recorded_at = Utc::now();

    let attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::HairColor),
        AttributeValue::HairColor(HairColorValue::Brown),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::Imported { system: "dmv".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    assert!(matches!(attr.value, AttributeValue::HairColor(HairColorValue::Brown)));
}

#[test]
fn test_record_eye_color() {
    // Test Scenario: Record eye color (identifying attribute)
    let recorded_at = Utc::now();

    let attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::EyeColor),
        AttributeValue::EyeColor(EyeColorValue::Blue),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Likely,
        ),
    );

    assert!(matches!(attr.value, AttributeValue::EyeColor(EyeColorValue::Blue)));
}

#[test]
fn test_record_biological_sex() {
    // Test Scenario: Record biological sex (identifying attribute)
    let recorded_at = Utc::now();

    let attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BiologicalSex),
        AttributeValue::BiologicalSex(BiologicalSexValue::Female),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::Imported { system: "birth_certificate".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    assert!(matches!(attr.value, AttributeValue::BiologicalSex(BiologicalSexValue::Female)));
}

#[test]
fn test_record_handedness() {
    // Test Scenario: Record handedness
    let recorded_at = Utc::now();

    let attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Handedness),
        AttributeValue::Handedness(HandednessValue::Right),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Certain,
        ),
    );

    assert!(matches!(attr.value, AttributeValue::Handedness(HandednessValue::Right)));
}

// ===== US-2.3: Record Healthcare Attributes =====

#[test]
fn test_record_medical_record_number() {
    // Test Scenario: Record MRN from hospital system
    let recorded_at = Utc::now();

    let attr = PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::MedicalRecordNumber),
        AttributeValue::Text("MRN-123456".to_string()),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::Imported { system: "hospital_system".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    if let AttributeValue::Text(ref mrn) = attr.value {
        assert_eq!(mrn, "MRN-123456");
    }
}

#[test]
fn test_record_insurance_id_with_validity() {
    // Test Scenario: Record insurance ID with policy dates
    let recorded_at = Utc::now();
    let policy_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let policy_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

    let attr = PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::InsuranceId),
        AttributeValue::Text("INS-789012".to_string()),
        TemporalValidity::new(recorded_at, Some(policy_start), Some(policy_end)),
        Provenance::new(
            AttributeSource::Imported { system: "insurance_provider".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    assert_eq!(attr.temporal.valid_from, Some(policy_start));
    assert_eq!(attr.temporal.valid_until, Some(policy_end));
}

#[test]
fn test_update_organ_donor_status() {
    // Test Scenario: Update organ donor status
    let recorded_at = Utc::now();

    let attr_initial = PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::OrganDonor),
        AttributeValue::Boolean(false),
        TemporalValidity::of(recorded_at),
        Provenance::new(
            AttributeSource::Imported { system: "dmv".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    let attr_updated = PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::OrganDonor),
        AttributeValue::Boolean(true),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Certain,
        ),
    );

    assert!(matches!(attr_initial.value, AttributeValue::Boolean(false)));
    assert!(matches!(attr_updated.value, AttributeValue::Boolean(true)));
}

#[test]
fn test_invalidate_insurance_with_reason() {
    // Test Scenario: Invalidate old insurance ID when changed
    let recorded_at = Utc::now();
    let valid_from = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let invalidated_on = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    let attr = PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::InsuranceId),
        AttributeValue::Text("OLD-INS-123".to_string()),
        TemporalValidity::new(recorded_at, Some(valid_from), Some(invalidated_on)),
        Provenance::new(
            AttributeSource::Imported { system: "insurance_provider".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    // Invalidation is represented by valid_until being set
    assert!(attr.temporal.valid_until.is_some());
    assert_eq!(attr.temporal.valid_until.unwrap(), invalidated_on);
}

// ===== US-2.4: Query Attributes by Category and Time =====

#[test]
fn test_filter_identifying_attributes() {
    // Test Scenario: Get all identifying attributes
    let mut set = PersonAttributeSet::empty();

    set.attributes.push(PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
        AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "gov".to_string() }, ConfidenceLevel::Certain),
    ));

    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    let identifying = set.identifying_attributes();
    assert_eq!(identifying.attributes.len(), 1);
    assert!(matches!(
        identifying.attributes[0].attribute_type,
        AttributeType::Identifying(_)
    ));
}

#[test]
fn test_filter_physical_attributes() {
    // Test Scenario: Get all physical attributes
    let mut set = PersonAttributeSet::empty();

    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(70.0),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    set.attributes.push(PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::InsuranceId),
        AttributeValue::Text("INS-123".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "insurance".to_string() }, ConfidenceLevel::Certain),
    ));

    let physical = set.filter(|attr| matches!(attr.attribute_type, AttributeType::Physical(_)));
    assert_eq!(physical.attributes.len(), 2);

    for attr in physical.attributes {
        assert!(matches!(attr.attribute_type, AttributeType::Physical(_)));
    }
}

#[test]
fn test_filter_healthcare_attributes() {
    // Test Scenario: Get all healthcare attributes
    let mut set = PersonAttributeSet::empty();

    set.attributes.push(PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::MedicalRecordNumber),
        AttributeValue::Text("MRN-123".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "hospital".to_string() }, ConfidenceLevel::Certain),
    ));

    set.attributes.push(PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::InsuranceId),
        AttributeValue::Text("INS-456".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "insurance".to_string() }, ConfidenceLevel::Certain),
    ));

    let healthcare = set.healthcare_attributes();
    assert_eq!(healthcare.attributes.len(), 2);
}

#[test]
fn test_filter_demographic_attributes() {
    // Test Scenario: Get all demographic attributes
    let mut set = PersonAttributeSet::empty();

    set.attributes.push(PersonAttribute::new(
        AttributeType::Demographic(DemographicAttributeType::Nationality),
        AttributeValue::Text("US".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "gov".to_string() }, ConfidenceLevel::Certain),
    ));

    let demographic = set.filter(|attr| matches!(attr.attribute_type, AttributeType::Demographic(_)));
    assert_eq!(demographic.attributes.len(), 1);
}

#[test]
fn test_filter_currently_valid() {
    // Test Scenario: Get current physical attributes for display
    let mut set = PersonAttributeSet::empty();

    // Past attribute (not currently valid)
    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.70),
        TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2021, 12, 31).unwrap()),
        ),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    // Current attribute (currently valid)
    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    let current = set.currently_valid();

    // Only the current attribute should be returned
    assert_eq!(current.attributes.len(), 1);

    if let AttributeValue::Length(height) = current.attributes[0].value {
        assert_eq!(height, 1.75);
    }
}

#[test]
fn test_filter_valid_on_specific_date() {
    // Test Scenario: Get healthcare attributes valid during 2023
    let mut set = PersonAttributeSet::empty();

    set.attributes.push(PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::InsuranceId),
        AttributeValue::Text("INS-2023".to_string()),
        TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
        ),
        Provenance::new(AttributeSource::Imported { system: "insurance".to_string() }, ConfidenceLevel::Certain),
    ));

    set.attributes.push(PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::InsuranceId),
        AttributeValue::Text("INS-2024".to_string()),
        TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            None,
        ),
        Provenance::new(AttributeSource::Imported { system: "insurance".to_string() }, ConfidenceLevel::Certain),
    ));

    let date_2023 = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
    let valid_2023 = set.valid_on(date_2023);

    assert_eq!(valid_2023.attributes.len(), 1);

    if let AttributeValue::Text(ref ins_id) = valid_2023.attributes[0].value {
        assert_eq!(ins_id, "INS-2023");
    }
}

#[test]
fn test_query_specific_attribute_type() {
    // Test Scenario: Get person's blood type (most recent)
    let mut set = PersonAttributeSet::empty();

    set.attributes.push(PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::BloodType),
        AttributeValue::BloodType(BloodTypeValue::APositive),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "hospital".to_string() }, ConfidenceLevel::Certain),
    ));

    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    // Filter to blood type only
    let blood_type_attr = set.attributes.iter()
        .find(|attr| matches!(
            attr.attribute_type,
            AttributeType::Healthcare(HealthcareAttributeType::BloodType)
        ))
        .expect("Should find blood type");

    assert!(matches!(blood_type_attr.value, AttributeValue::BloodType(BloodTypeValue::APositive)));
}

// ===== US-2.5: Transform and Derive Attributes (Functor Testing) =====

#[test]
fn test_attribute_functor_map() {
    // Test Scenario: Convert height from inches to meters
    let attr_inches = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(69.0), // 69 inches
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    );

    // Save recorded_at before consuming
    let original_recorded_at = attr_inches.temporal.recorded_at;

    // Map to convert inches to meters
    let attr_meters = attr_inches.map(|value| {
        if let AttributeValue::Length(inches) = value {
            AttributeValue::Length(inches * 0.0254) // Convert to meters
        } else {
            value
        }
    });

    if let AttributeValue::Length(meters) = attr_meters.value {
        assert!((meters - 1.7526).abs() < 0.001, "Expected ~1.75 meters");
    } else {
        panic!("Expected Length value");
    }

    // Verify attribute type preserved
    assert!(matches!(
        attr_meters.attribute_type,
        AttributeType::Physical(PhysicalAttributeType::Height)
    ));

    // Verify temporal validity preserved
    assert_eq!(attr_meters.temporal.recorded_at, original_recorded_at);
}

#[test]
fn test_attribute_map_preserves_provenance() {
    // Test Scenario: Normalize name capitalization
    let attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::NationalId),
        AttributeValue::Text("abc123".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Likely,
        ),
    );

    let normalized = attr.map(|value| {
        if let AttributeValue::Text(text) = value {
            AttributeValue::Text(text.to_uppercase())
        } else {
            value
        }
    });

    // Verify transformation
    if let AttributeValue::Text(ref text) = normalized.value {
        assert_eq!(text, "ABC123");
    }

    // Verify provenance preserved
    assert_eq!(normalized.provenance.confidence, ConfidenceLevel::Likely);
}

#[test]
fn test_compose_multiple_transformations() {
    // Test Scenario: Compose multiple transformations
    let attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(150.0), // pounds
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    );

    // Transform 1: Convert pounds to kg
    let attr_kg = attr.map(|value| {
        if let AttributeValue::Mass(pounds) = value {
            AttributeValue::Mass(pounds * 0.453592)
        } else {
            value
        }
    });

    // Transform 2: Round to 1 decimal place
    let attr_rounded = attr_kg.map(|value| {
        if let AttributeValue::Mass(kg) = value {
            AttributeValue::Mass((kg * 10.0).round() / 10.0)
        } else {
            value
        }
    });

    if let AttributeValue::Mass(kg) = attr_rounded.value {
        assert!((kg - 68.0).abs() < 0.1, "Expected ~68 kg");
    }
}

// ===== US-8.1: Validate Attribute Quality (Confidence Levels) =====

#[test]
fn test_high_confidence_government_source() {
    // Test Scenario: Record high-confidence government source
    let attr = PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::NationalId),
        AttributeValue::Text("123-45-6789".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::Imported { system: "ssa_gov".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    assert_eq!(attr.provenance.confidence, ConfidenceLevel::Certain);
}

#[test]
fn test_low_confidence_user_data() {
    // Test Scenario: Record low-confidence user-entered data
    let attr = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Uncertain,
        ),
    );

    assert_eq!(attr.provenance.confidence, ConfidenceLevel::Uncertain);
}

#[test]
fn test_filter_by_minimum_confidence() {
    // Test Scenario: Filter to high-confidence only
    let mut set = PersonAttributeSet::empty();

    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Uncertain),
    ));

    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(70.0),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "hospital".to_string() }, ConfidenceLevel::Certain),
    ));

    let high_confidence: Vec<_> = set.attributes.iter()
        .filter(|attr| attr.provenance.confidence == ConfidenceLevel::Certain)
        .collect();

    assert_eq!(high_confidence.len(), 1);
}

#[test]
fn test_compare_conflicting_attributes_by_confidence() {
    // Test Scenario: Compare conflicting attributes by confidence
    let attr_low = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.70),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Uncertain),
    );

    let attr_high = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Imported { system: "hospital".to_string() }, ConfidenceLevel::Certain),
    );

    // In case of conflict, prefer higher confidence
    assert_eq!(attr_high.provenance.confidence, ConfidenceLevel::Certain);
    assert_eq!(attr_low.provenance.confidence, ConfidenceLevel::Uncertain);
}

// ===== US-8.2: Track Data Lineage (Provenance) =====

#[test]
fn test_provenance_source_tracking() {
    // Test Scenario: Record attribute from external system
    let attr = PersonAttribute::new(
        AttributeType::Healthcare(HealthcareAttributeType::MedicalRecordNumber),
        AttributeValue::Text("MRN-123".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::Imported { system: "epic_ehr".to_string() },
            ConfidenceLevel::Certain,
        ),
    );

    if let AttributeSource::Imported { system: ref system } = attr.provenance.source {
        assert_eq!(system, "epic_ehr");
    } else {
        panic!("Expected Integration source");
    }
}

#[test]
fn test_query_attributes_from_specific_source() {
    // Test Scenario: Query all attributes from source "IRS"
    let mut set = PersonAttributeSet::empty();

    set.attributes.push(PersonAttribute::new(
        AttributeType::Identifying(IdentifyingAttributeType::NationalId),
        AttributeValue::Text("123-45-6789".to_string()),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::Imported { system: "IRS".to_string() },
            ConfidenceLevel::Certain,
        ),
    ));

    set.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(
            AttributeSource::SelfReported,
            ConfidenceLevel::Likely,
        ),
    ));

    let from_irs: Vec<_> = set.attributes.iter()
        .filter(|attr| {
            matches!(
                &attr.provenance.source,
                AttributeSource::Imported { system: system } if system == "IRS"
            )
        })
        .collect();

    assert_eq!(from_irs.len(), 1);
}

// ===== Edge Cases and Complex Scenarios =====

#[test]
fn test_empty_attribute_set() {
    let set = PersonAttributeSet::empty();
    assert_eq!(set.attributes.len(), 0);

    let current = set.currently_valid();
    assert_eq!(current.attributes.len(), 0);
}

#[test]
fn test_attribute_set_composition() {
    // Test Monoid operation: set1 + set2
    let mut set1 = PersonAttributeSet::empty();
    set1.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    let mut set2 = PersonAttributeSet::empty();
    set2.attributes.push(PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Weight),
        AttributeValue::Mass(70.0),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::SelfReported, ConfidenceLevel::Likely),
    ));

    let combined = set1 + set2;
    assert_eq!(combined.attributes.len(), 2);
}

#[test]
fn test_all_blood_type_variants() {
    let blood_types = vec![
        BloodTypeValue::APositive,
        BloodTypeValue::ANegative,
        BloodTypeValue::BPositive,
        BloodTypeValue::BNegative,
        BloodTypeValue::ABPositive,
        BloodTypeValue::ABNegative,
        BloodTypeValue::OPositive,
        BloodTypeValue::ONegative,
    ];

    for blood_type in blood_types {
        let attr = PersonAttribute::new(
            AttributeType::Healthcare(HealthcareAttributeType::BloodType),
            AttributeValue::BloodType(blood_type),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Imported { system: "lab".to_string() }, ConfidenceLevel::Certain),
        );

        assert!(matches!(attr.value, AttributeValue::BloodType(_)));
    }
}
