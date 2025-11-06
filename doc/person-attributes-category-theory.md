# Person Attributes: Category Theory Compliance

## Critical Requirement: Structure-Preserving Functors for Cross-Domain Composition

**Person domain is a dependency for many other domains**. Therefore, all Person structures must be:

1. **Category Theory (CT) compliant**
2. **Structure-preserving functors**
3. **Composable across domain boundaries**
4. **Immutable value objects** (FRP compliance)

## Functor Laws for PersonAttribute

### Core Principle: PersonAttribute as a Functor

A `PersonAttribute` is a functor `F` from the category of attribute values to the category of attributed entities:

```
F: AttributeValue → AttributedEntity
```

### Functor Laws

**Law 1: Identity**
```rust
attribute.map(|x| x) == attribute
```

**Law 2: Composition**
```rust
attribute.map(f).map(g) == attribute.map(|x| g(f(x)))
```

**Law 3: Structure Preservation**
```rust
// Temporal structure must be preserved
attribute1.compose(attribute2).temporal_ordering() ==
    attribute1.temporal_ordering().compose(attribute2.temporal_ordering())
```

## Category Theory Implementation

### 1. PersonAttribute as Morphism

PersonAttribute represents a morphism in the category of person identities:

```rust
/// PersonAttribute is a morphism: Identity → Attribution
/// This preserves the categorical structure of person identity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonAttribute {
    /// The attribute type (domain of the morphism)
    pub attribute_type: AttributeType,

    /// The attribute value (codomain of the morphism)
    pub value: AttributeValue,

    /// Temporal validity (temporal category)
    pub temporal: TemporalValidity,

    /// Provenance (source category)
    pub provenance: Provenance,
}

impl PersonAttribute {
    /// Functor map: Apply a function to the attribute value
    /// while preserving structure
    pub fn map<F>(self, f: F) -> Self
    where
        F: FnOnce(AttributeValue) -> AttributeValue,
    {
        PersonAttribute {
            attribute_type: self.attribute_type,
            value: f(self.value),
            temporal: self.temporal,
            provenance: self.provenance,
        }
    }

    /// Functor compose: Chain attribute transformations
    pub fn and_then<F>(self, f: F) -> Option<Self>
    where
        F: FnOnce(AttributeValue) -> Option<AttributeValue>,
    {
        f(self.value).map(|new_value| PersonAttribute {
            attribute_type: self.attribute_type,
            value: new_value,
            temporal: self.temporal,
            provenance: self.provenance,
        })
    }

    /// Natural transformation: Convert between attribute types
    /// while preserving categorical structure
    pub fn transform<F>(self, f: F) -> Self
    where
        F: FnOnce(AttributeType, AttributeValue) -> (AttributeType, AttributeValue),
    {
        let (new_type, new_value) = f(self.attribute_type, self.value);
        PersonAttribute {
            attribute_type: new_type,
            value: new_value,
            temporal: self.temporal,
            provenance: self.provenance,
        }
    }
}
```

### 2. Temporal Validity as Monad

Temporal validity forms a monad for composing time-based transformations:

```rust
/// Temporal validity monad for composing time-dependent attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalValidity {
    /// When this attribute was recorded
    pub recorded_at: DateTime<Utc>,

    /// When this attribute became valid
    pub valid_from: Option<NaiveDate>,

    /// When this attribute stopped being valid
    pub valid_until: Option<NaiveDate>,
}

impl TemporalValidity {
    /// Monad unit (return): Lift a value into the temporal context
    pub fn of(time: DateTime<Utc>) -> Self {
        Self {
            recorded_at: time,
            valid_from: None,
            valid_until: None,
        }
    }

    /// Monad bind (flatMap): Compose temporal transformations
    pub fn flat_map<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }

    /// Check if valid at a given time (predicate functor)
    pub fn is_valid_at(&self, time: NaiveDate) -> bool {
        let after_start = self.valid_from
            .map(|start| time >= start)
            .unwrap_or(true);

        let before_end = self.valid_until
            .map(|end| time < end)
            .unwrap_or(true);

        after_start && before_end
    }

    /// Compose temporal validities (preserve ordering)
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
}
```

### 3. Provenance as Traced Category

Provenance forms a traced category for tracking attribute sources:

```rust
/// Provenance tracking (traced category)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: AttributeSource,
    pub confidence: ConfidenceLevel,
    pub recorded_by: Option<String>,

    /// Trace: Chain of transformations applied to this attribute
    pub trace: Vec<Transformation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transformation {
    pub operation: String,
    pub timestamp: DateTime<Utc>,
    pub applied_by: String,
}

impl Provenance {
    /// Add a transformation to the trace
    pub fn trace_transformation(mut self, operation: String, applied_by: String) -> Self {
        self.trace.push(Transformation {
            operation,
            timestamp: chrono::Utc::now(),
            applied_by,
        });
        self
    }

    /// Compose provenances (preserve trace)
    pub fn compose(mut self, other: Self) -> Self {
        self.trace.extend(other.trace);
        self
    }
}
```

## Cross-Domain Composition

### Structure-Preserving Functors for Other Domains

Other domains can compose with Person attributes while preserving structure:

```rust
/// Healthcare domain composes with Person domain
/// This is a structure-preserving functor: Person → Healthcare
pub struct HealthcarePatient {
    /// Reference to Person (structure preserved)
    person_ref: PersonReference,

    /// Healthcare-specific attributes (derived functor)
    medical_attributes: Vec<MedicalAttribute>,
}

impl HealthcarePatient {
    /// Map from Person attributes to Healthcare attributes
    /// This is a natural transformation between functors
    pub fn from_person_attributes(
        person_ref: PersonReference,
        person_attrs: Vec<PersonAttribute>,
    ) -> Self {
        // Extract healthcare-relevant attributes
        let medical_attributes = person_attrs
            .into_iter()
            .filter_map(|attr| attr.to_medical_attribute())
            .collect();

        Self {
            person_ref,
            medical_attributes,
        }
    }
}

impl PersonAttribute {
    /// Natural transformation: PersonAttribute → MedicalAttribute
    /// Preserves categorical structure
    fn to_medical_attribute(self) -> Option<MedicalAttribute> {
        match self.attribute_type {
            AttributeType::Physical(PhysicalAttributeType::Height) => {
                Some(MedicalAttribute::Height(self.value, self.temporal))
            }
            AttributeType::Physical(PhysicalAttributeType::Weight) => {
                Some(MedicalAttribute::Weight(self.value, self.temporal))
            }
            AttributeType::Physical(PhysicalAttributeType::BloodType) => {
                Some(MedicalAttribute::BloodType(self.value))
            }
            AttributeType::Healthcare(_) => {
                Some(MedicalAttribute::from_person_attribute(self))
            }
            _ => None,  // Not healthcare-relevant
        }
    }
}
```

### Location Domain Composition

```rust
/// Location domain references Person birth place
/// This is a pullback in the category of domains
pub struct PersonLocationLink {
    person_ref: PersonReference,
    location_ref: LocationReference,
    relationship: LocationRelationship,
}

impl PersonLocationLink {
    /// Create a structure-preserving link
    pub fn birth_place(
        person_ref: PersonReference,
        birth_place_attr: PersonAttribute,
    ) -> Option<Self> {
        // Extract location reference from attribute
        if let AttributeValue::Location(location_ref) = birth_place_attr.value {
            Some(Self {
                person_ref,
                location_ref,
                relationship: LocationRelationship::BirthPlace {
                    temporal: birth_place_attr.temporal,
                },
            })
        } else {
            None
        }
    }
}
```

### Identity Domain Composition

```rust
/// Identity domain (SSN, Passport, etc.) composes with Person
/// This is a coproduct (sum type) in the category
pub struct IdentityDocument {
    person_ref: PersonReference,
    document_type: DocumentType,
    document_number: String,
    issuing_authority: String,
    temporal: TemporalValidity,
}

impl IdentityDocument {
    /// Link identity document to person
    /// Preserves person identity structure
    pub fn link_to_person(
        self,
        person: &Person,
    ) -> Result<LinkedIdentity, LinkError> {
        // Verify person attributes match document
        let birth_date_matches = person
            .get_attribute(AttributeType::Identifying(IdentifyingAttributeType::BirthDate))
            .map(|attr| self.verify_birth_date(attr))
            .unwrap_or(false);

        if birth_date_matches {
            Ok(LinkedIdentity {
                person_ref: PersonReference::from(person.id),
                document: self,
            })
        } else {
            Err(LinkError::IdentityMismatch)
        }
    }
}
```

## Attribute Collection as Free Monad

The collection of attributes forms a free monad for composing attribute operations:

```rust
/// PersonAttributeSet is a free monad over attribute operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonAttributeSet {
    attributes: Vec<PersonAttribute>,
}

impl PersonAttributeSet {
    /// Monad unit: Empty attribute set
    pub fn empty() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    /// Monad unit: Single attribute
    pub fn of(attribute: PersonAttribute) -> Self {
        Self {
            attributes: vec![attribute],
        }
    }

    /// Monad bind: Compose attribute sets
    pub fn flat_map<F>(self, f: F) -> Self
    where
        F: Fn(PersonAttribute) -> PersonAttributeSet,
    {
        let attributes = self.attributes
            .into_iter()
            .flat_map(|attr| f(attr).attributes)
            .collect();

        Self { attributes }
    }

    /// Functor map: Apply transformation to all attributes
    pub fn map<F>(self, f: F) -> Self
    where
        F: Fn(PersonAttribute) -> PersonAttribute,
    {
        let attributes = self.attributes
            .into_iter()
            .map(f)
            .collect();

        Self { attributes }
    }

    /// Filter (predicate functor)
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: Fn(&PersonAttribute) -> bool,
    {
        let attributes = self.attributes
            .into_iter()
            .filter(|attr| predicate(attr))
            .collect();

        Self { attributes }
    }

    /// Compose attribute sets (monoid operation)
    pub fn compose(mut self, other: Self) -> Self {
        self.attributes.extend(other.attributes);
        self
    }
}

/// Monoid instance for PersonAttributeSet
impl std::ops::Add for PersonAttributeSet {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.compose(other)
    }
}
```

## Person as Coalgebra

Person aggregate is a coalgebra for the attribute functor:

```rust
/// Person is a coalgebra: Person → F(Person)
/// where F is the attribute functor
pub struct Person {
    id: PersonId,
    core_identity: CoreIdentity,

    /// Coalgebra structure: unfold into attributes
    attributes: PersonAttributeSet,
}

impl Person {
    /// Coalgebra unfold: Extract attribute structure
    pub fn unfold(&self) -> PersonAttributeSet {
        self.attributes.clone()
    }

    /// Coalgebra map: Apply attribute transformation
    pub fn map_attributes<F>(mut self, f: F) -> Self
    where
        F: Fn(PersonAttribute) -> PersonAttribute,
    {
        self.attributes = self.attributes.map(f);
        self
    }

    /// Coalgebra observe: Query attributes at a point in time
    pub fn observe_at(&self, time: NaiveDate) -> PersonAttributeSet {
        self.attributes.clone().filter(|attr| {
            attr.temporal.is_valid_at(time)
        })
    }

    /// Anamorphism: Build person from attribute stream
    pub fn from_attributes(
        id: PersonId,
        core_identity: CoreIdentity,
        attributes: impl Iterator<Item = PersonAttribute>,
    ) -> Self {
        let attribute_set = attributes.fold(
            PersonAttributeSet::empty(),
            |acc, attr| acc + PersonAttributeSet::of(attr),
        );

        Self {
            id,
            core_identity,
            attributes: attribute_set,
        }
    }
}
```

## Cross-Domain Functors

### Functor: Person → Healthcare

```rust
pub struct PersonToHealthcareFunctor;

impl PersonToHealthcareFunctor {
    /// Apply functor: Person → Healthcare
    pub fn apply(person: &Person) -> HealthcarePatient {
        let person_ref = PersonReference::from(person.id);

        // Extract healthcare-relevant attributes (structure preserved)
        let healthcare_attrs = person
            .unfold()
            .filter(|attr| attr.is_healthcare_relevant())
            .map(|attr| attr.to_medical_attribute().unwrap());

        HealthcarePatient::from_person_attributes(
            person_ref,
            healthcare_attrs.attributes,
        )
    }
}
```

### Functor: Person → Location

```rust
pub struct PersonToLocationFunctor;

impl PersonToLocationFunctor {
    /// Apply functor: Person → Location
    pub fn apply(person: &Person) -> Vec<PersonLocationLink> {
        let person_ref = PersonReference::from(person.id);

        // Extract location attributes (structure preserved)
        person
            .unfold()
            .filter(|attr| attr.is_location_reference())
            .attributes
            .into_iter()
            .filter_map(|attr| {
                PersonLocationLink::birth_place(person_ref.clone(), attr)
            })
            .collect()
    }
}
```

### Functor: Person → Identity

```rust
pub struct PersonToIdentityFunctor;

impl PersonToIdentityFunctor {
    /// Apply functor: Person → Identity
    pub fn apply(person: &Person) -> IdentityProfile {
        let person_ref = PersonReference::from(person.id);

        // Extract identity attributes (structure preserved)
        let birth_date = person.get_attribute(
            AttributeType::Identifying(IdentifyingAttributeType::BirthDate)
        );

        let birth_place = person.get_attribute(
            AttributeType::Identifying(IdentifyingAttributeType::BirthPlace)
        );

        IdentityProfile {
            person_ref,
            birth_date,
            birth_place,
        }
    }
}
```

## Laws and Invariants

### Functor Laws (MUST HOLD)

```rust
#[cfg(test)]
mod functor_laws {
    use super::*;

    #[test]
    fn test_identity_law() {
        let attr = PersonAttribute::new(/* ... */);
        let result = attr.clone().map(|x| x);
        assert_eq!(attr, result);
    }

    #[test]
    fn test_composition_law() {
        let attr = PersonAttribute::new(/* ... */);

        let f = |x: AttributeValue| x.transform_a();
        let g = |x: AttributeValue| x.transform_b();

        let result1 = attr.clone().map(f).map(g);
        let result2 = attr.map(|x| g(f(x)));

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_temporal_preservation() {
        let attr1 = PersonAttribute::with_temporal(/* ... */);
        let attr2 = PersonAttribute::with_temporal(/* ... */);

        let composed = attr1.clone().compose(attr2.clone());

        // Temporal ordering must be preserved
        assert!(composed.temporal.preserves_ordering(&attr1.temporal, &attr2.temporal));
    }
}
```

### Monad Laws (MUST HOLD)

```rust
#[cfg(test)]
mod monad_laws {
    use super::*;

    #[test]
    fn test_left_identity() {
        // return a >>= f ≡ f a
        let a = PersonAttribute::new(/* ... */);
        let f = |x: PersonAttribute| PersonAttributeSet::of(x.transform());

        let result1 = PersonAttributeSet::of(a.clone()).flat_map(&f);
        let result2 = f(a);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_right_identity() {
        // m >>= return ≡ m
        let m = PersonAttributeSet::of(PersonAttribute::new(/* ... */));

        let result = m.clone().flat_map(|x| PersonAttributeSet::of(x));

        assert_eq!(m, result);
    }

    #[test]
    fn test_associativity() {
        // (m >>= f) >>= g ≡ m >>= (\x -> f x >>= g)
        let m = PersonAttributeSet::new(/* ... */);
        let f = |x: PersonAttribute| PersonAttributeSet::of(x.transform_a());
        let g = |x: PersonAttribute| PersonAttributeSet::of(x.transform_b());

        let result1 = m.clone().flat_map(&f).flat_map(&g);
        let result2 = m.flat_map(|x| f(x).flat_map(&g));

        assert_eq!(result1, result2);
    }
}
```

## Summary: CT Compliance Checklist

✅ **Functor**: PersonAttribute implements `map` with identity and composition laws
✅ **Monad**: TemporalValidity and PersonAttributeSet implement `unit` and `bind`
✅ **Coalgebra**: Person unfolds into attribute structure
✅ **Natural Transformation**: Cross-domain mappings preserve structure
✅ **Traced Category**: Provenance tracks transformation chains
✅ **Monoid**: PersonAttributeSet has identity (empty) and composition
✅ **Structure Preservation**: Temporal ordering and categorical structure preserved
✅ **Cross-Domain Composition**: Other domains can compose via functors
✅ **Immutability**: All value objects are immutable (FRP compliant)
✅ **Event Sourcing**: Changes produce events (category of state transitions)

## For Other Domains Using Person

When your domain depends on Person:

1. **Use PersonReference** (don't copy Person data)
2. **Map via Functors** (preserve structure)
3. **Compose Attributes** (use monoid operations)
4. **Respect Temporal Validity** (filter by time)
5. **Preserve Provenance** (track transformations)
6. **Event-Driven Integration** (react to PersonEvents)

Example:
```rust
// ✅ CORRECT: Structure-preserving functor
let healthcare_patient = PersonToHealthcareFunctor::apply(&person);

// ❌ WRONG: Copying person data directly
let patient = Patient {
    name: person.core_identity.legal_name.clone(),  // Don't do this!
    // ...
};
```
