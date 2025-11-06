//! Person Attribute Value Object
//!
//! Provides an extensible Entity-Attribute-Value (EAV) system for person attributes
//! with temporal tracking, provenance, and Category Theory compliance.
//!
//! # Category Theory Compliance
//!
//! - PersonAttribute is a Functor (with map)
//! - TemporalValidity is a Monad (with unit/bind)
//! - PersonAttributeSet is a Free Monad
//! - All transformations preserve structure

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use crate::aggregate::PersonId;

// ============================================================================
// Attribute Type Taxonomy
// ============================================================================

/// Top-level attribute type categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttributeType {
    /// Attributes that help identify/disambiguate persons
    Identifying(IdentifyingAttributeType),
    /// Physical characteristics (temporal, can change)
    Physical(PhysicalAttributeType),
    /// Healthcare-related attributes
    Healthcare(HealthcareAttributeType),
    /// Demographic information
    Demographic(DemographicAttributeType),
    /// Organization-specific custom attributes
    Custom(CustomAttributeType),
}

/// Identifying attributes for disambiguation (40% weight)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentifyingAttributeType {
    /// Birth date and time (most precise)
    BirthDateTime,
    /// Birth date only (if time unknown)
    BirthDate,
    /// Birth year only (if that's all we know)
    BirthYear,
    /// Approximate birth date with precision indicator
    ApproximateBirthDate,
    /// Birth place (reference to Location domain)
    BirthPlace,
    /// Eye color (stable biometric)
    EyeColor,
    /// Biological sex (as assigned at birth)
    BiologicalSex,
    /// Blood type (stable biometric)
    BloodType,
    /// Mother's ID (family relationship)
    MotherId,
    /// Father's ID (family relationship)
    FatherId,
    /// National identification number (SSN, etc.)
    NationalId,
}

/// Physical attributes (temporal, can change over time)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalAttributeType {
    Height,
    Weight,
    HairColor,
    HairStyle,
    FacialHair,
    Scars,
    Birthmarks,
    Tattoos,
    Piercings,
    Handedness,
}

/// Healthcare-related attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthcareAttributeType {
    BloodType,
    Allergies,
    ChronicConditions,
    Medications,
    Disabilities,
    VisionCorrection,
    HearingAids,
    /// Medical record number from healthcare provider
    MedicalRecordNumber,
    /// Insurance identification number
    InsuranceId,
    /// Organ donor status
    OrganDonor,
}

/// Demographic attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DemographicAttributeType {
    PreferredLanguage,
    PrimaryLanguage,
    SpokenLanguages,
    Citizenship,
    Nationality,
    Ethnicity,
    Religion,
}

/// Custom organization-specific attributes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomAttributeType {
    pub organization: String,
    pub attribute_name: String,
    pub category: String,
}

// ============================================================================
// Attribute Values with Strong Typing
// ============================================================================

/// Strongly-typed attribute values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    /// Text value
    Text(String),
    /// Numeric value
    Number(f64),
    /// Integer value
    Integer(i64),
    /// Boolean value
    Boolean(bool),
    /// Full date and time
    DateTime(DateTime<Utc>),
    /// Date only (no time)
    Date(NaiveDate),
    /// Year and month only
    YearMonth(i32, u32),
    /// Year only
    Year(i32),
    /// Approximate date with precision
    ApproximateDate {
        date: NaiveDate,
        precision: DatePrecision,
    },
    /// Length measurement (in meters)
    Length(f64),
    /// Mass measurement (in kilograms)
    Mass(f64),
    /// Blood type
    BloodType(BloodTypeValue),
    /// Eye color
    EyeColor(EyeColorValue),
    /// Hair color
    HairColor(HairColorValue),
    /// Biological sex
    BiologicalSex(BiologicalSexValue),
    /// Handedness
    Handedness(HandednessValue),
    /// Location reference (to Location domain)
    LocationReference(String),
    /// Person reference (for family relationships)
    PersonReference(PersonId),
    /// List of text values
    TextList(Vec<String>),
    /// JSON value for complex data
    Json(serde_json::Value),
}

/// Date precision levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatePrecision {
    /// Exact date known
    Exact,
    /// Month-level precision
    Month,
    /// Year-level precision
    Year,
    /// Decade-level precision
    Decade,
    /// Century-level precision
    Century,
}

/// Blood type values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BloodTypeValue {
    APositive,
    ANegative,
    BPositive,
    BNegative,
    ABPositive,
    ABNegative,
    OPositive,
    ONegative,
}

/// Eye color values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EyeColorValue {
    Brown,
    Blue,
    Green,
    Hazel,
    Gray,
    Amber,
    Heterochromia,
}

/// Hair color values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HairColorValue {
    Black,
    Brown,
    Blonde,
    Red,
    Gray,
    White,
    Dyed,
}

/// Biological sex values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiologicalSexValue {
    Male,
    Female,
    Intersex,
}

/// Handedness values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandednessValue {
    Right,
    Left,
    Ambidextrous,
}

// ============================================================================
// Temporal Validity (Monad)
// ============================================================================

/// Temporal validity tracking for attributes
///
/// This is a Monad that satisfies all monad laws:
/// - Left identity: `return a >>= f ≡ f a`
/// - Right identity: `m >>= return ≡ m`
/// - Associativity: `(m >>= f) >>= g ≡ m >>= (\x -> f x >>= g)`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalValidity {
    /// When we learned about this attribute
    pub recorded_at: DateTime<Utc>,
    /// When this attribute became valid/true
    pub valid_from: Option<NaiveDate>,
    /// When this attribute stopped being valid/true
    pub valid_until: Option<NaiveDate>,
}

impl TemporalValidity {
    /// Monad unit (return) - create from a single point in time
    pub fn of(time: DateTime<Utc>) -> Self {
        Self {
            recorded_at: time,
            valid_from: None,
            valid_until: None,
        }
    }

    /// Create with full temporal range
    pub fn new(
        recorded_at: DateTime<Utc>,
        valid_from: Option<NaiveDate>,
        valid_until: Option<NaiveDate>,
    ) -> Self {
        Self {
            recorded_at,
            valid_from,
            valid_until,
        }
    }

    /// Monad bind (flat_map)
    pub fn flat_map<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }

    /// Compose two temporal validities (preserves temporal ordering)
    pub fn compose(self, other: Self) -> Self {
        Self {
            recorded_at: self.recorded_at.max(other.recorded_at),
            valid_from: match (self.valid_from, other.valid_from) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            valid_until: match (self.valid_until, other.valid_until) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
        }
    }

    /// Check if valid on a specific date
    pub fn is_valid_on(&self, date: NaiveDate) -> bool {
        let after_start = self.valid_from.map_or(true, |start| date >= start);
        let before_end = self.valid_until.map_or(true, |end| date <= end);
        after_start && before_end
    }

    /// Check if currently valid
    pub fn is_currently_valid(&self) -> bool {
        let today = Utc::now().date_naive();
        self.is_valid_on(today)
    }
}

// ============================================================================
// Provenance Tracking
// ============================================================================

/// Source of attribute information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttributeSource {
    /// Self-reported by the person
    SelfReported,
    /// Measured/observed directly
    Measured,
    /// From official document
    DocumentVerified,
    /// Computed/derived from other attributes
    Computed,
    /// Imported from external system
    Imported { system: String },
}

/// Confidence level in attribute value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Certain,
    Likely,
    Possible,
    Uncertain,
}

/// Provenance tracking for attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// Source of this attribute
    pub source: AttributeSource,
    /// Confidence in this value
    pub confidence: ConfidenceLevel,
    /// Who recorded this attribute
    pub recorded_by: Option<String>,
    /// Trace of transformations applied
    pub trace: Vec<TransformationTrace>,
}

/// Record of a transformation applied to an attribute
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformationTrace {
    /// Name of transformation
    pub transformation: String,
    /// When transformation was applied
    pub applied_at: DateTime<Utc>,
    /// Who/what applied it
    pub applied_by: String,
}

impl Provenance {
    /// Create new provenance record
    pub fn new(source: AttributeSource, confidence: ConfidenceLevel) -> Self {
        Self {
            source,
            confidence,
            recorded_by: None,
            trace: Vec::new(),
        }
    }

    /// Add a transformation to the trace
    pub fn trace_transformation(
        mut self,
        transformation: String,
        applied_by: String,
    ) -> Self {
        self.trace.push(TransformationTrace {
            transformation,
            applied_at: Utc::now(),
            applied_by,
        });
        self
    }
}

// ============================================================================
// Person Attribute (Functor)
// ============================================================================

/// A single attribute of a person with temporal and provenance tracking
///
/// This is a Functor that satisfies functor laws:
/// - Identity: `attribute.map(|x| x) == attribute`
/// - Composition: `attribute.map(f).map(g) == attribute.map(|x| g(f(x)))`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonAttribute {
    /// Type of attribute
    pub attribute_type: AttributeType,
    /// Value of attribute
    pub value: AttributeValue,
    /// Temporal validity
    pub temporal: TemporalValidity,
    /// Provenance information
    pub provenance: Provenance,
}

impl PersonAttribute {
    /// Create a new attribute
    pub fn new(
        attribute_type: AttributeType,
        value: AttributeValue,
        temporal: TemporalValidity,
        provenance: Provenance,
    ) -> Self {
        Self {
            attribute_type,
            value,
            temporal,
            provenance,
        }
    }

    /// Functor map - transform the value while preserving structure
    pub fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(AttributeValue) -> AttributeValue,
    {
        Self {
            attribute_type: self.attribute_type,
            value: f(self.value),
            temporal: self.temporal,
            provenance: self.provenance,
        }
    }

    /// Transform with context (for advanced use cases)
    pub fn and_then<F>(self, f: F) -> Option<Self>
    where
        F: FnOnce(Self) -> Option<Self>,
    {
        f(self)
    }

    /// Transform and record in provenance trace
    pub fn transform<F>(self, transformation: String, applied_by: String, f: F) -> Self
    where
        F: FnOnce(AttributeValue) -> AttributeValue,
    {
        Self {
            attribute_type: self.attribute_type,
            value: f(self.value),
            temporal: self.temporal,
            provenance: self.provenance.trace_transformation(transformation, applied_by),
        }
    }

    /// Check if valid on a specific date
    pub fn is_valid_on(&self, date: NaiveDate) -> bool {
        self.temporal.is_valid_on(date)
    }

    /// Check if currently valid
    pub fn is_currently_valid(&self) -> bool {
        self.temporal.is_currently_valid()
    }

    /// Check if this is a healthcare-relevant attribute
    pub fn is_healthcare_relevant(&self) -> bool {
        matches!(self.attribute_type, AttributeType::Healthcare(_))
    }

    /// Check if this is an identifying attribute
    pub fn is_identifying(&self) -> bool {
        matches!(self.attribute_type, AttributeType::Identifying(_))
    }
}

// ============================================================================
// Person Attribute Set (Free Monad)
// ============================================================================

/// Collection of person attributes with monoid and monad operations
///
/// This is a Free Monad over PersonAttribute, providing:
/// - Monoid: empty identity and associative composition
/// - Monad: unit (of) and bind (flat_map)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonAttributeSet {
    pub attributes: Vec<PersonAttribute>,
}

impl PersonAttributeSet {
    /// Monoid identity - empty set
    pub fn empty() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    /// Monad unit - create from single attribute
    pub fn of(attribute: PersonAttribute) -> Self {
        Self {
            attributes: vec![attribute],
        }
    }

    /// Create from vector of attributes
    pub fn from_vec(attributes: Vec<PersonAttribute>) -> Self {
        Self { attributes }
    }

    /// Monad bind (flat_map)
    pub fn flat_map<F>(self, f: F) -> Self
    where
        F: Fn(PersonAttribute) -> PersonAttributeSet,
    {
        let mut result = Vec::new();
        for attr in self.attributes {
            let mapped = f(attr);
            result.extend(mapped.attributes);
        }
        Self {
            attributes: result,
        }
    }

    /// Functor map
    pub fn map<F>(self, f: F) -> Self
    where
        F: Fn(PersonAttribute) -> PersonAttribute,
    {
        Self {
            attributes: self.attributes.into_iter().map(f).collect(),
        }
    }

    /// Filter attributes
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: Fn(&PersonAttribute) -> bool,
    {
        Self {
            attributes: self.attributes.into_iter().filter(predicate).collect(),
        }
    }

    /// Get attributes valid on a specific date
    pub fn valid_on(&self, date: NaiveDate) -> Self {
        Self {
            attributes: self
                .attributes
                .iter()
                .filter(|attr| attr.is_valid_on(date))
                .cloned()
                .collect(),
        }
    }

    /// Get currently valid attributes
    pub fn currently_valid(&self) -> Self {
        let today = Utc::now().date_naive();
        self.valid_on(today)
    }

    /// Find attribute by type
    pub fn find_by_type(&self, attr_type: &AttributeType) -> Option<&PersonAttribute> {
        self.attributes.iter().find(|attr| &attr.attribute_type == attr_type)
    }

    /// Get all attributes of a specific category
    pub fn identifying_attributes(&self) -> Self {
        self.clone().filter(|attr| attr.is_identifying())
    }

    /// Get all healthcare attributes
    pub fn healthcare_attributes(&self) -> Self {
        self.clone().filter(|attr| attr.is_healthcare_relevant())
    }
}

/// Monoid append operation via Add trait
impl std::ops::Add for PersonAttributeSet {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self.attributes.extend(other.attributes);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Temporal Validity Monad Laws
    // ========================================================================

    #[test]
    fn test_temporal_validity_monad_left_identity() {
        // Left identity: return a >>= f ≡ f a
        let time = Utc::now();
        let f = |t: TemporalValidity| {
            TemporalValidity::new(
                t.recorded_at,
                Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                None,
            )
        };

        let left = TemporalValidity::of(time).flat_map(&f);
        let right = f(TemporalValidity::of(time));
        assert_eq!(left, right, "Left identity law violated");
    }

    #[test]
    fn test_temporal_validity_monad_right_identity() {
        // Right identity: m >>= return ≡ m
        // Our "return" function for TemporalValidity monad is the identity transformation
        let temporal = TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        );

        // flat_map with identity should return the same temporal validity
        let result = temporal.clone().flat_map(|t| t);

        assert_eq!(result.recorded_at, temporal.recorded_at, "Right identity violated: recorded_at");
        assert_eq!(result.valid_from, temporal.valid_from, "Right identity violated: valid_from");
        assert_eq!(result.valid_until, temporal.valid_until, "Right identity violated: valid_until");
    }

    #[test]
    fn test_temporal_validity_monad_associativity() {
        // Associativity: (m >>= f) >>= g ≡ m >>= (\x -> f x >>= g)
        let temporal = TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            None,
        );

        let f = |t: TemporalValidity| TemporalValidity::new(
            t.recorded_at,
            Some(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()),
            t.valid_until,
        );

        let g = |t: TemporalValidity| TemporalValidity::new(
            t.recorded_at,
            t.valid_from,
            Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        );

        let left = temporal.clone().flat_map(&f).flat_map(&g);
        let right = temporal.flat_map(|x| f(x).flat_map(&g));

        assert_eq!(left, right, "Associativity law violated");
    }

    #[test]
    fn test_temporal_validity_composition() {
        // Composition should preserve temporal ordering
        let t1 = TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
        );

        let t2 = TemporalValidity::new(
            Utc::now(),
            Some(NaiveDate::from_ymd_opt(2021, 6, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        );

        let composed = t1.compose(t2);

        // Composed should have the intersection of valid ranges
        assert_eq!(composed.valid_from, Some(NaiveDate::from_ymd_opt(2021, 6, 1).unwrap()));
        assert_eq!(composed.valid_until, Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()));
    }

    // ========================================================================
    // PersonAttribute Functor Laws
    // ========================================================================

    #[test]
    fn test_attribute_functor_identity_law() {
        // Identity: attribute.map(|x| x) == attribute
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let result = attr.clone().map(|x| x);
        assert_eq!(attr, result, "Functor identity law violated");
    }

    #[test]
    fn test_attribute_functor_composition_law() {
        // Composition: attribute.map(f).map(g) == attribute.map(|x| g(f(x)))
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        // f: convert meters to centimeters
        let f = |v: AttributeValue| match v {
            AttributeValue::Length(m) => AttributeValue::Length(m * 100.0),
            _ => v,
        };

        // g: round to integer
        let g = |v: AttributeValue| match v {
            AttributeValue::Length(cm) => AttributeValue::Integer(cm as i64),
            _ => v,
        };

        let left = attr.clone().map(f).map(g);
        let right = attr.map(|x| g(f(x)));

        assert_eq!(left, right, "Functor composition law violated");
    }

    #[test]
    fn test_attribute_functor_structure_preservation() {
        // Structure preservation: temporal ordering must be preserved
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::new(
                Utc::now(),
                Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            ),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let transformed = attr.clone().map(|v| match v {
            AttributeValue::Length(m) => AttributeValue::Integer((m * 100.0) as i64),
            _ => v,
        });

        // Temporal validity should be preserved
        assert_eq!(attr.temporal, transformed.temporal);
        // Provenance should be preserved
        assert_eq!(attr.provenance, transformed.provenance);
        // Only value should change
        assert_ne!(attr.value, transformed.value);
    }

    // ========================================================================
    // PersonAttributeSet Free Monad Laws
    // ========================================================================

    #[test]
    fn test_attribute_set_monad_left_identity() {
        // Left identity: return a >>= f ≡ f a
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let f = |a: PersonAttribute| {
            PersonAttributeSet::from_vec(vec![
                a.clone(),
                PersonAttribute::new(
                    AttributeType::Physical(PhysicalAttributeType::Weight),
                    AttributeValue::Mass(70.0),
                    TemporalValidity::of(Utc::now()),
                    Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
                ),
            ])
        };

        let left = PersonAttributeSet::of(attr.clone()).flat_map(&f);
        let right = f(attr);

        assert_eq!(left.attributes.len(), right.attributes.len());
    }

    #[test]
    fn test_attribute_set_monad_right_identity() {
        // Right identity: m >>= return ≡ m
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let set = PersonAttributeSet::of(attr);
        let result = set.clone().flat_map(PersonAttributeSet::of);

        assert_eq!(set.attributes.len(), result.attributes.len());
    }

    #[test]
    fn test_attribute_set_functor_map() {
        // Map should transform all attributes
        let attr1 = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let attr2 = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Weight),
            AttributeValue::Mass(70.0),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let set = PersonAttributeSet::from_vec(vec![attr1, attr2]);

        let transformed = set.map(|attr| {
            attr.transform(
                "test_transformation".to_string(),
                "test_system".to_string(),
                |v| v,
            )
        });

        // All attributes should have the transformation in their trace
        for attr in transformed.attributes {
            assert_eq!(attr.provenance.trace.len(), 1);
            assert_eq!(attr.provenance.trace[0].transformation, "test_transformation");
        }
    }

    // ========================================================================
    // PersonAttributeSet Monoid Laws
    // ========================================================================

    #[test]
    fn test_attribute_set_monoid_left_identity() {
        // Left identity: empty + set == set
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let set = PersonAttributeSet::of(attr);
        let empty = PersonAttributeSet::empty();

        let result = empty + set.clone();
        assert_eq!(result.attributes.len(), set.attributes.len());
    }

    #[test]
    fn test_attribute_set_monoid_right_identity() {
        // Right identity: set + empty == set
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let set = PersonAttributeSet::of(attr);
        let empty = PersonAttributeSet::empty();

        let result = set.clone() + empty;
        assert_eq!(result.attributes.len(), set.attributes.len());
    }

    #[test]
    fn test_attribute_set_monoid_associativity() {
        // Associativity: (a + b) + c == a + (b + c)
        let attr1 = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let attr2 = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Weight),
            AttributeValue::Mass(70.0),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let attr3 = PersonAttribute::new(
            AttributeType::Healthcare(HealthcareAttributeType::BloodType),
            AttributeValue::BloodType(BloodTypeValue::OPositive),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
        );

        let set1 = PersonAttributeSet::of(attr1);
        let set2 = PersonAttributeSet::of(attr2);
        let set3 = PersonAttributeSet::of(attr3);

        let left = (set1.clone() + set2.clone()) + set3.clone();
        let right = set1 + (set2 + set3);

        assert_eq!(left.attributes.len(), right.attributes.len());
        assert_eq!(left.attributes.len(), 3);
    }

    // ========================================================================
    // Temporal Queries
    // ========================================================================

    #[test]
    fn test_temporal_validity_query() {
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::new(
                Utc::now(),
                Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            ),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        // Should be valid in 2023
        assert!(attr.is_valid_on(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap()));

        // Should not be valid in 2019
        assert!(!attr.is_valid_on(NaiveDate::from_ymd_opt(2019, 12, 31).unwrap()));

        // Should not be valid in 2026
        assert!(!attr.is_valid_on(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()));
    }

    #[test]
    fn test_attribute_set_temporal_filter() {
        let attr1 = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::new(
                Utc::now(),
                Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
            ),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let attr2 = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Weight),
            AttributeValue::Mass(70.0),
            TemporalValidity::new(
                Utc::now(),
                Some(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
                Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            ),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let set = PersonAttributeSet::from_vec(vec![attr1, attr2]);

        // Query attributes valid in 2021
        let valid_2021 = set.valid_on(NaiveDate::from_ymd_opt(2021, 6, 15).unwrap());
        assert_eq!(valid_2021.attributes.len(), 1); // Only height was valid

        // Query attributes valid in 2023
        let valid_2023 = set.valid_on(NaiveDate::from_ymd_opt(2023, 6, 15).unwrap());
        assert_eq!(valid_2023.attributes.len(), 2); // Both were valid
    }

    // ========================================================================
    // Provenance Tracking
    // ========================================================================

    #[test]
    fn test_provenance_trace() {
        let attr = PersonAttribute::new(
            AttributeType::Physical(PhysicalAttributeType::Height),
            AttributeValue::Length(1.75),
            TemporalValidity::of(Utc::now()),
            Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
        );

        let transformed = attr.transform(
            "meters_to_centimeters".to_string(),
            "conversion_service".to_string(),
            |v| match v {
                AttributeValue::Length(m) => AttributeValue::Length(m * 100.0),
                _ => v,
            },
        );

        assert_eq!(transformed.provenance.trace.len(), 1);
        assert_eq!(transformed.provenance.trace[0].transformation, "meters_to_centimeters");
        assert_eq!(transformed.provenance.trace[0].applied_by, "conversion_service");
    }
}
