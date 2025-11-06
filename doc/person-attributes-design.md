# Person Attributes: Extensible Value System

## Problem Statement

### 1. Identity Disambiguation
Two people can have identical names but be different persons:
- John Smith born 1985-03-15 in Boston, MA
- John Smith born 1985-03-15 in London, UK
- John Smith born 1990-07-20 in Boston, MA

We need additional identifying attributes to disambiguate.

### 2. Physical Descriptors (Temporal)
Human physical attributes change over time:
- Height: Grows from childhood, can shrink with age
- Weight: Fluctuates constantly
- Hair color: Changes naturally (graying) or artificially (dyed)
- Scars/tattoos: Added over time, can fade/be removed
- Eye color: Generally stable but can change (contacts, medical conditions)

### 3. Context-Specific Requirements
Different domains need different attributes:
- **Healthcare**: Height, weight, blood type, allergies, scars, birthmarks
- **Law Enforcement**: Physical descriptors, identifying marks, tattoos
- **Business/CRM**: Just basic demographics
- **Gaming/Entertainment**: Avatar preferences, completely different attributes

### 4. Extensibility Challenge
We can't predict all future attribute needs. The system must allow:
- Adding new attribute types without code changes
- Domain-specific attributes
- Custom attributes per organization

## Core Design Principle

> **Everything that describes the human being belongs in Person domain**
>
> BUT interpretation/use of those attributes belongs in context-specific policies

## Architecture: Entity-Attribute-Value Pattern

### Core Concept

```rust
pub struct Person {
    id: PersonId,
    core_identity: CoreIdentity,      // Name, vital dates
    attributes: Vec<PersonAttribute>,  // Extensible collection of values
}

pub struct PersonAttribute {
    attribute_type: AttributeType,     // What is being measured
    value: AttributeValue,             // The actual value
    recorded_at: DateTime<Utc>,        // When was this recorded
    valid_from: Option<NaiveDate>,     // When did this become true
    valid_until: Option<NaiveDate>,    // When did this stop being true
    source: AttributeSource,           // Who/what recorded this
    confidence: ConfidenceLevel,       // How certain are we
}
```

## Attribute Type Taxonomy

### Category 1: Identifying Attributes (Help Disambiguate)

These help distinguish between persons with similar names:

```rust
pub enum IdentifyingAttributeType {
    // Temporal identity (precision varies)
    BirthDateTime,     // Date and time if known (most precise)
    BirthDate,         // Just the date if time unknown
    BirthYear,         // Only year if that's all we know

    // Location-based (references external Location domain)
    BirthPlace,        // City/region of birth
    BirthCountry,      // Country of birth

    // Physical (relatively stable)
    EyeColor,          // Generally stable
    NaturalHairColor,  // Original color

    // Biological
    Sex,               // Biological sex at birth (distinct from gender identity)

    // Governmental (separate domain for official IDs)
    // SSN, Passport, National ID → cim-domain-identity
}
```

### Category 2: Physical Descriptors (Temporal)

These change over time and should be versioned:

```rust
pub enum PhysicalAttributeType {
    // Measurements
    Height,            // In cm, changes over lifetime
    Weight,            // In kg, fluctuates frequently

    // Appearance
    CurrentHairColor,  // Can be dyed, changes
    CurrentHairStyle,  // Changes frequently
    FacialHair,        // Beard, mustache, clean-shaven

    // Distinguishing marks (additive over time)
    Scar(String),      // Location/description
    Tattoo(String),    // Location/description
    Birthmark(String), // Location/description
    Piercing(String),  // Location

    // Medical/Biological
    BloodType,         // Generally stable
    Handedness,        // Left/Right/Ambidextrous
}
```

### Category 3: Healthcare-Specific

```rust
pub enum HealthcareAttributeType {
    // Critical medical info
    Allergies(String),
    MedicalConditions(String),
    Medications(String),

    // Physical capabilities
    MobilityAids,      // Wheelchair, cane, etc.
    VisionCorrection,  // Glasses, contacts
    HearingAids,

    // These might belong in separate medical domain
    // but Person needs to reference them for identification
}
```

### Category 4: Demographic (Stable)

```rust
pub enum DemographicAttributeType {
    // These are generally stable but can change
    PreferredLanguage,
    Citizenship,       // Can have multiple
    Ethnicity,         // Self-identified
    Religion,          // Self-identified, can change
}
```

### Category 5: Custom/Extensible

```rust
pub enum CustomAttributeType {
    String(String),    // Domain defines the key
    // Examples: "favorite_color", "gaming_level", "loyalty_tier"
}
```

## Attribute Value Types

Strongly typed values:

```rust
pub enum AttributeValue {
    // Primitive types
    Text(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),

    // Temporal types (with varying precision)
    DateTime(DateTime<Utc>),           // Full date and time
    Date(NaiveDate),                   // Just the date
    YearMonth(i32, u32),               // Year and month only
    Year(i32),                         // Just the year
    ApproximateDate {                  // "Around March 1985"
        date: NaiveDate,
        precision: DatePrecision,
    },

    // Measurements (with units)
    Length { value: f64, unit: LengthUnit },      // Height
    Mass { value: f64, unit: MassUnit },          // Weight
    Temperature { value: f64, unit: TempUnit },   // Body temp, etc.

    // Enumerations
    BloodType(BloodType),
    EyeColor(EyeColor),
    HairColor(HairColor),
    BiologicalSex(BiologicalSex),

    // Complex types
    Location(LocationReference),  // External reference
    Custom(serde_json::Value),    // For extensibility
}

// Date precision for approximate dates
pub enum DatePrecision {
    Exact,        // We know the exact date
    Month,        // We know month and year
    Year,         // We only know the year
    Decade,       // We know the decade (1980s)
    Century,      // We only know the century
}

pub enum LengthUnit { Centimeters, Inches, Feet }
pub enum MassUnit { Kilograms, Pounds }
pub enum TempUnit { Celsius, Fahrenheit }

pub enum BloodType {
    APositive, ANegative, BPositive, BNegative,
    ABPositive, ABNegative, OPositive, ONegative
}

pub enum EyeColor {
    Brown, Blue, Green, Hazel, Gray, Amber, Other(String)
}

pub enum HairColor {
    Black, Brown, Blonde, Red, Gray, White, Other(String)
}

pub enum BiologicalSex {
    Male,
    Female,
    Intersex,
    Unknown,  // Not recorded/not disclosed
}
```

## Temporal Tracking

Attributes change over time, so we track validity periods:

```rust
pub struct PersonAttribute {
    attribute_type: AttributeType,
    value: AttributeValue,

    // Temporal validity
    recorded_at: DateTime<Utc>,        // When we learned about this
    valid_from: Option<NaiveDate>,     // When this became true
    valid_until: Option<NaiveDate>,    // When this stopped being true

    // Provenance
    source: AttributeSource,
    confidence: ConfidenceLevel,
    recorded_by: Option<String>,       // Who recorded this
}

pub enum AttributeSource {
    SelfReported,        // Person told us
    Measured,            // We measured it (healthcare)
    Observed,            // We saw it
    DocumentVerified,    // From official document
    ThirdParty(String),  // External system
}

pub enum ConfidenceLevel {
    Certain,       // 100% sure
    Likely,        // 75-99%
    Possible,      // 50-74%
    Uncertain,     // < 50%
}
```

## Identity Disambiguation Strategy

To distinguish between persons with the same name, we use a hierarchical approach:

```rust
pub struct PersonDisambiguationCriteria {
    // PRIMARY: Birth date/time + birth location (most precise)
    birth_datetime: Option<DateTime<Utc>>,  // If we know time
    birth_date: Option<NaiveDate>,          // If we only know date
    birth_year: Option<i32>,                // If we only know year
    birth_place: Option<LocationReference>,

    // SECONDARY: Stable biological attributes
    biological_sex: Option<BiologicalSex>,
    eye_color: Option<EyeColor>,
    natural_hair_color: Option<HairColor>,
    blood_type: Option<BloodType>,

    // TERTIARY: Parent/family relationships
    mother_id: Option<PersonId>,
    father_id: Option<PersonId>,

    // QUATERNARY: Physical measurements (less reliable, change over time)
    // These are snapshot at time of disambiguation
}

impl Person {
    /// Get attributes suitable for disambiguation
    pub fn disambiguation_profile(&self) -> PersonDisambiguationCriteria {
        // Extract relevant attributes from the attributes collection
    }

    /// Calculate similarity score with another person
    /// Returns score from 0.0 (definitely different) to 1.0 (definitely same)
    pub fn similarity_score(&self, other: &Person) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Name similarity (30% weight)
        let name_score = self.core_identity.legal_name.similarity(&other.core_identity.legal_name);
        score += name_score * 0.30;
        total_weight += 0.30;

        // Birth datetime (40% weight - most important)
        if let (Some(dt1), Some(dt2)) = (self.get_birth_datetime(), other.get_birth_datetime()) {
            if dt1 == dt2 {
                score += 0.40;
            }
            total_weight += 0.40;
        } else if let (Some(d1), Some(d2)) = (self.get_birth_date(), other.get_birth_date()) {
            if d1 == d2 {
                score += 0.35;  // Slightly less weight without time
            }
            total_weight += 0.35;
        }

        // Birth place (20% weight)
        if let (Some(p1), Some(p2)) = (self.get_birth_place(), other.get_birth_place()) {
            if p1 == p2 {
                score += 0.20;
            }
            total_weight += 0.20;
        }

        // Biological attributes (10% weight)
        if let (Some(s1), Some(s2)) = (self.get_biological_sex(), other.get_biological_sex()) {
            if s1 == s2 {
                score += 0.05;
            } else {
                // Different biological sex = definitely different people
                return 0.0;
            }
            total_weight += 0.05;
        }

        // Normalize score
        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        }
    }
}
```

### Disambiguation Examples

```rust
// Example 1: Same name, same date, DIFFERENT time and place = DIFFERENT people
let john1 = Person::builder()
    .name(PersonName::new("John", "Smith"))
    .attribute(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDateTime),
        AttributeValue::DateTime(
            DateTime::parse_from_rfc3339("1985-03-15T08:30:00Z").unwrap()
        ),  // 8:30 AM in Boston
    )
    .attribute(
        AttributeType::Identifying(IdentifyingAttributeType::BirthPlace),
        AttributeValue::Location(LocationReference::new("Boston, MA, USA")),
    )
    .build();

let john2 = Person::builder()
    .name(PersonName::new("John", "Smith"))
    .attribute(
        AttributeType::Identifying(IdentifyingAttributeType::BirthDateTime),
        AttributeValue::DateTime(
            DateTime::parse_from_rfc3339("1985-03-15T13:45:00Z").unwrap()
        ),  // 1:45 PM in London (same day, different time)
    )
    .attribute(
        AttributeType::Identifying(IdentifyingAttributeType::BirthPlace),
        AttributeValue::Location(LocationReference::new("London, UK")),
    )
    .build();

// Similarity score will be low due to different time and place
assert!(john1.similarity_score(&john2) < 0.5);

// Example 2: Same name, ONLY know year = Need more attributes
let john3 = Person::builder()
    .name(PersonName::new("John", "Smith"))
    .attribute(
        AttributeType::Identifying(IdentifyingAttributeType::BirthYear),
        AttributeValue::Year(1985),
    )
    .attribute(
        AttributeType::Physical(PhysicalAttributeType::EyeColor),
        AttributeValue::EyeColor(EyeColor::Blue),
    )
    .attribute(
        AttributeType::Physical(PhysicalAttributeType::BloodType),
        AttributeValue::BloodType(BloodType::OPositive),
    )
    .build();

// Less precise birth info requires more physical attributes for disambiguation
```

## Attribute Policies

Different contexts need different attributes:

```rust
pub enum AttributePolicy {
    /// Minimum to identify a unique person
    IdentificationMinimum,
    // Required: Name, birth date, birth location

    /// Healthcare context
    Healthcare,
    // Required: Height, weight, blood type, allergies
    // Optional: Scars, birthmarks, medical conditions

    /// Law enforcement
    LawEnforcement,
    // Required: Height, weight, hair, eyes, scars, tattoos

    /// Business/CRM
    Business,
    // Required: Just name
    // Optional: Everything else

    /// Custom policy
    Custom(Vec<AttributeType>),
}

impl Person {
    /// Check if person meets policy requirements
    pub fn satisfies_policy(&self, policy: AttributePolicy) -> bool {
        // Check if all required attributes are present
    }

    /// Get missing attributes for policy
    pub fn missing_attributes(&self, policy: AttributePolicy) -> Vec<AttributeType> {
        // Return what's needed to satisfy policy
    }
}
```

## Event Sourcing for Attributes

Attributes are recorded/updated via events:

```rust
pub enum PersonEvent {
    // Existing events...
    PersonCreated(PersonCreated),

    // Attribute events
    AttributeRecorded {
        person_id: PersonId,
        attribute_type: AttributeType,
        value: AttributeValue,
        recorded_at: DateTime<Utc>,
        source: AttributeSource,
    },

    AttributeUpdated {
        person_id: PersonId,
        attribute_type: AttributeType,
        old_value: AttributeValue,
        new_value: AttributeValue,
        recorded_at: DateTime<Utc>,
        source: AttributeSource,
    },

    AttributeInvalidated {
        person_id: PersonId,
        attribute_type: AttributeType,
        reason: String,
        invalidated_at: DateTime<Utc>,
    },
}
```

## Real-World Examples

### Example 1: Healthcare System

```rust
// Record initial patient attributes
person.record_attribute(
    AttributeType::Physical(PhysicalAttributeType::Height),
    AttributeValue::Length { value: 175.0, unit: LengthUnit::Centimeters },
    AttributeSource::Measured,
)?;

person.record_attribute(
    AttributeType::Physical(PhysicalAttributeType::Weight),
    AttributeValue::Mass { value: 70.0, unit: MassUnit::Kilograms },
    AttributeSource::Measured,
)?;

person.record_attribute(
    AttributeType::Physical(PhysicalAttributeType::BloodType),
    AttributeValue::BloodType(BloodType::OPositive),
    AttributeSource::DocumentVerified,
)?;

person.record_attribute(
    AttributeType::Healthcare(HealthcareAttributeType::Allergies("Penicillin".to_string())),
    AttributeValue::Text("Severe reaction to penicillin".to_string()),
    AttributeSource::SelfReported,
)?;

// Check if meets healthcare policy
assert!(person.satisfies_policy(AttributePolicy::Healthcare));
```

### Example 2: Identity Disambiguation

```rust
// Two people named "John Smith"
let john1 = Person::builder()
    .name(PersonName::new("John", "Smith"))
    .birth_date(NaiveDate::from_ymd_opt(1985, 3, 15))
    .record_attribute(
        AttributeType::Identifying(IdentifyingAttributeType::BirthPlace),
        AttributeValue::Location(LocationReference::new("Boston, MA, USA")),
    )
    .record_attribute(
        AttributeType::Physical(PhysicalAttributeType::EyeColor),
        AttributeValue::EyeColor(EyeColor::Blue),
    )
    .build();

let john2 = Person::builder()
    .name(PersonName::new("John", "Smith"))
    .birth_date(NaiveDate::from_ymd_opt(1985, 3, 15))  // Same date!
    .record_attribute(
        AttributeType::Identifying(IdentifyingAttributeType::BirthPlace),
        AttributeValue::Location(LocationReference::new("London, UK")),  // Different place
    )
    .record_attribute(
        AttributeType::Physical(PhysicalAttributeType::EyeColor),
        AttributeValue::EyeColor(EyeColor::Brown),
    )
    .build();

// Calculate similarity
let similarity = john1.similarity_score(&john2);
// Result: Low similarity due to different birth place despite same name/date
```

### Example 3: Temporal Attributes (Weight Changes)

```rust
// Initial weight
person.record_attribute_on(
    AttributeType::Physical(PhysicalAttributeType::Weight),
    AttributeValue::Mass { value: 80.0, unit: MassUnit::Kilograms },
    NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
    AttributeSource::Measured,
)?;

// Weight changes over time
person.record_attribute_on(
    AttributeType::Physical(PhysicalAttributeType::Weight),
    AttributeValue::Mass { value: 75.0, unit: MassUnit::Kilograms },
    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    AttributeSource::Measured,
)?;

// Query historical weight
let weight_jan_2023 = person.get_attribute_on(
    AttributeType::Physical(PhysicalAttributeType::Weight),
    NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
)?;
// Returns: 80.0 kg

let weight_jan_2024 = person.get_attribute_on(
    AttributeType::Physical(PhysicalAttributeType::Weight),
    NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
)?;
// Returns: 75.0 kg
```

### Example 4: Distinguishing Marks (Additive)

```rust
// Person gets a tattoo
person.record_attribute(
    AttributeType::Physical(PhysicalAttributeType::Tattoo("Left forearm".to_string())),
    AttributeValue::Text("Dragon design, 15cm x 10cm".to_string()),
    AttributeSource::Observed,
)?;

// Later, another tattoo
person.record_attribute(
    AttributeType::Physical(PhysicalAttributeType::Tattoo("Right shoulder".to_string())),
    AttributeValue::Text("Rose design, 8cm diameter".to_string()),
    AttributeSource::Observed,
)?;

// Get all tattoos
let tattoos = person.get_all_attributes(|attr| {
    matches!(attr.attribute_type, AttributeType::Physical(PhysicalAttributeType::Tattoo(_)))
});
```

## Domain Boundaries

### In Person Domain ✅
- Physical attributes intrinsic to the human
- Identifying characteristics
- Demographic self-identification
- Healthcare attributes describing the body

### External Domains ❌
- **Location Domain**: Birth place, current address (Person references these)
- **Identity Domain**: Government IDs (SSN, passport, driver's license)
- **Medical Domain**: Diagnoses, treatments, medical records (Person has attributes, Medical has interpretations)
- **Organization Domain**: Employment details
- **Contacts Domain**: Email, phone (communication methods, not intrinsic)

## Implementation Strategy

### Phase 1: Core Attribute System
- [ ] Define `AttributeType` enum hierarchy
- [ ] Define `AttributeValue` enum with strong typing
- [ ] Create `PersonAttribute` value object
- [ ] Add temporal tracking (valid_from, valid_until)

### Phase 2: Integrate with Person Aggregate
- [ ] Add `attributes: Vec<PersonAttribute>` to Person
- [ ] Implement commands: `RecordAttribute`, `UpdateAttribute`, `InvalidateAttribute`
- [ ] Implement events: `AttributeRecorded`, `AttributeUpdated`, `AttributeInvalidated`
- [ ] Add query methods: `get_attribute()`, `get_all_attributes()`, `get_attribute_history()`

### Phase 3: Disambiguation Support
- [ ] Implement `disambiguation_profile()`
- [ ] Implement `similarity_score()` algorithm
- [ ] Add duplicate detection queries

### Phase 4: Policy Framework
- [ ] Define `AttributePolicy` enum
- [ ] Implement `satisfies_policy()`
- [ ] Implement `missing_attributes()`
- [ ] Create policy validators

### Phase 5: Extensibility
- [ ] Allow custom attribute types
- [ ] Support custom attribute value types
- [ ] Create attribute registry for organization-specific types

## Benefits

1. **Flexibility**: Add new attributes without changing core model
2. **Temporal**: Track how attributes change over time
3. **Provenance**: Know where data came from and how certain we are
4. **Disambiguation**: Distinguish between persons with identical names
5. **Context-Specific**: Different domains can require different attributes
6. **Extensible**: Organizations can add custom attributes
7. **Event Sourced**: Full audit trail of attribute changes
8. **Type Safe**: Strong typing prevents errors

## Migration Strategy

Existing code continues to work:
```rust
// Old way (still works)
let person = Person::new(person_id, name);

// New way (with attributes)
let person = Person::builder()
    .name(name)
    .birth_date(date)
    .attribute(height_attribute)
    .attribute(weight_attribute)
    .build();
```

## References

- Entity-Attribute-Value (EAV) pattern
- Temporal database design
- Duplicate detection algorithms (Jaro-Winkler, Levenshtein)
- Healthcare HL7 FHIR Person resource
