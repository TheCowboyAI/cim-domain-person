# CIM Domain: Person

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/tests-194%20passing-brightgreen)](./tests)

> Pure functional event-sourced Person domain for CIM (Composable Information Machine), providing comprehensive person identity management with Category Theory foundations and CQRS architecture.

## Overview

The Person domain implements a **pure functional reactive** (FRP) architecture for managing person identities, attributes, and lifecycle events. It follows rigorous mathematical foundations with 100% FRP/Category Theory compliance.

### Key Features

- 🎯 **Pure Event Sourcing**: All state changes through immutable events, zero CRUD operations
- 📐 **Category Theory Foundations**: Formal Functor, Monad, Coalgebra, and Natural Transformation implementations
- 🔄 **CQRS Architecture**: Explicit command/query separation with type-safe specifications
- 🧮 **EAV Pattern**: Extensible attribute-value model for flexible person data
- 📊 **Pure Projections**: (State, Event) → NewState functions with zero side effects
- 🔐 **Lifecycle Management**: Active, Deactivated, Deceased, and Merged states
- ✨ **100% FRP Compliant**: Zero side effects in domain logic, infrastructure at boundaries only

## Architecture

### Person Aggregate Structure

The Person aggregate follows a minimalist design with extensible attributes:

```rust
Person {
    id: PersonId,                    // Unique identifier
    core_identity: CoreIdentity {    // Core immutable identity
        legal_name: PersonName,      // Current legal name
        birth_date: Option<NaiveDate>,
        death_date: Option<NaiveDate>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
    attributes: PersonAttributeSet { // Extensible attributes (EAV)
        attributes: Vec<PersonAttribute>,
    },
    lifecycle: PersonLifecycle,      // Active | Deactivated | Deceased | Merged
    version: u64,                    // Event sourcing version
}
```

### Attribute System (EAV Pattern)

Person attributes are stored using an Entity-Attribute-Value pattern with full provenance tracking:

```rust
PersonAttribute {
    attribute_type: AttributeType,   // What kind of attribute
    value: AttributeValue,           // The actual value
    temporal: TemporalValidity,      // When this attribute is valid
    provenance: Provenance {         // Where this data came from
        source: AttributeSource,     // DocumentVerified, Measured, SelfReported, etc.
        confidence: ConfidenceLevel, // Certain, Likely, Possible, Speculative
        recorded_at: DateTime<Utc>,
        transformation_history: Vec<TransformationRecord>,
    },
}
```

**Supported Attribute Types**:
- **Identifying**: BirthDate, NationalId, Passport, etc.
- **Physical**: Height, Weight, EyeColor, HairColor, BloodType, etc.
- **Healthcare**: MedicalRecordNumber, OrganDonor, HealthcareProvider, etc.
- **Demographic**: Nationality, Ethnicity, Language, etc.
- **Custom**: Extensible for domain-specific needs

### Event Sourcing Pattern

All state changes follow the pure functional Command → Event → State pattern:

```rust
// 1. Create a command
let command = PersonCommand::RecordAttribute(RecordAttribute {
    person_id,
    attribute: PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
    ),
});

// 2. Process command (pure function - produces events)
let current_state = person.lifecycle.clone();
let events = MealyStateMachine::output(&person, current_state.into(), command);

// 3. Apply events (pure function - new state)
for event in &events {
    person = person.apply_event_pure(event)?;
}
```

## CQRS Architecture

The domain implements explicit Command-Query Responsibility Segregation:

### Command Side (Writes)

Commands modify state through event generation:

```rust
// PersonService handles all writes
let service = PersonService::new(command_processor, query_service);

// Execute a command
let command = PersonCommand::CreatePerson(CreatePerson {
    person_id,
    name,
    source,
});

let result = service.execute_command(command).await?;
```

**Available Commands**:
- `CreatePerson` - Create a new person
- `UpdateName` - Update legal name
- `RecordAttribute` - Add/update an attribute
- `UpdateAttribute` - Modify existing attribute
- `InvalidateAttribute` - Mark attribute as invalid
- `DeactivatePerson` - Deactivate person
- `ReactivatePerson` - Reactivate person
- `MergePerson` - Merge duplicate persons

### Query Side (Reads)

Queries use immutable specifications against read models:

```rust
// Query person summaries
let query = PersonSummaryQuery::all()
    .paginate(0, 20);
let summaries = service.query_summaries(&query).await?;

// Search persons
let query = PersonSearchQuery::new("John Doe")
    .with_employer("Acme Corp")
    .with_min_relevance(0.8);
let results = service.search_persons(&query).await?;

// Query skills
let query = SkillsQuery::for_person(person_id)
    .with_category("Programming");
let skills = service.query_skills(&query).await?;
```

**Query Specifications**:
- `PersonSummaryQuery` - Basic person information with pagination
- `PersonSearchQuery` - Full-text search with filters
- `SkillsQuery` - Skills and certifications
- `NetworkQuery` - Relationships and connections
- `TimelineQuery` - Event timeline with date ranges

**Read Models**:
- `PersonSummary` - Aggregated person view
- `PersonSearchResult` - Search results with relevance scores
- `SkillSummary` - Skills with proficiency levels
- `PersonRelationship` - Network connections
- `TimelineEntry` - Activity history

## Category Theory Foundations

The domain implements formal Category Theory traits for mathematical rigor:

### Functor

Structure-preserving transformations:

```rust
// Transform attribute values while preserving structure
let transformed = attribute.fmap(|value| enhance(value));

// Composition law: fmap(g ∘ f) = fmap(g) ∘ fmap(f)
assert_eq!(
    attr.fmap(|v| g(f(v))),
    attr.fmap(f).fmap(g)
);
```

### Monad

Composition of operations that might fail:

```rust
// Compose attribute operations
let result = PersonAttributeSet::pure(attr)
    .bind(|a| validate(a))
    .bind(|a| transform(a))
    .bind(|a| enrich(a));

// Monad laws ensure compositional safety
```

### Coalgebra

State observation without mutation:

```rust
// Unfold person state for observation
let attributes = person.unfold();  // Returns PersonAttributeSet

// Observation doesn't mutate original
assert_eq!(person, original_person);
```

### Natural Transformations

Cross-domain mappings that preserve structure:

```rust
// Transform Person to HealthcarePatient (different domain)
let patient = PersonToHealthcareFunctor::apply(person);

// Structure preservation: relationships maintained
```

## Pure Functional Projections

All projections are pure functions with zero side effects:

```rust
// Pure projection function
fn project_person_summary(
    current: Option<PersonSummary>,
    event: &PersonEvent,
) -> Option<PersonSummary> {
    match event {
        PersonEvent::PersonCreated(e) => Some(PersonSummary {
            person_id: e.person_id,
            name: e.name.display_name(),
            created_at: Utc::now(),
            // ... pure computation only
        }),
        PersonEvent::AttributeRecorded(e) => {
            current.map(|mut summary| {
                summary.add_attribute(e.attribute.clone());
                summary  // No mutations, returns new value
            })
        }
        // ...
    }
}

// Infrastructure applies projections (side effects isolated)
let updated = project_person_summary(current, &event);
read_model.save(person_id, updated).await;  // I/O only at boundary
```

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cim-domain-person = "0.7"
cim-domain = "0.7"
```

### Basic Usage

```rust
use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::{PersonCommand, CreatePerson, RecordAttribute},
    value_objects::{
        PersonName, PersonAttribute, AttributeType, AttributeValue,
        IdentifyingAttributeType, TemporalValidity, Provenance,
        AttributeSource, ConfidenceLevel,
    },
};
use cim_domain::formal_domain::MealyStateMachine;
use chrono::{Utc, NaiveDate};

// Create a person
let person_id = PersonId::new();
let name = PersonName::new("Alice".to_string(), "Johnson".to_string());
let mut person = Person::new(person_id, name);

// Add an attribute through event sourcing
let attribute = PersonAttribute::new(
    AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
    AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap()),
    TemporalValidity::of(Utc::now()),
    Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
);

let command = PersonCommand::RecordAttribute(RecordAttribute {
    person_id,
    attribute,
});

let current_state = person.lifecycle.clone();
let events = MealyStateMachine::output(&person, current_state.into(), command);

for event in &events {
    person = person.apply_event_pure(event)?;
}

// Query attributes
let identifying = person.identifying_attributes();
let physical = person.attributes.physical_attributes();
let healthcare = person.attributes.healthcare_attributes();
```

### CQRS Service Usage

```rust
use cim_domain_person::{
    services::PersonService,
    queries::{PersonSummaryQuery, PersonSearchQuery},
};

// Initialize service
let service = PersonService::new(command_processor, query_service);

// Execute commands
service.execute_command(create_command).await?;

// Execute queries
let summaries = service.query_summaries(&PersonSummaryQuery::all()).await?;
let results = service.search_persons(&search_query).await?;
```

## Documentation

Comprehensive documentation is available in the `/doc` directory:

- **[FRP-CT-COMPLIANCE.md](doc/FRP-CT-COMPLIANCE.md)** - 100% FRP/Category Theory compliance details
- **[person-attributes-design.md](doc/person-attributes-design.md)** - Attribute system design and EAV pattern
- **[person-attributes-category-theory.md](doc/person-attributes-category-theory.md)** - Mathematical foundations
- **[person-names-design.md](doc/person-names-design.md)** - PersonName design and international support
- **[person-names-examples.md](doc/person-names-examples.md)** - Name handling examples
- **[USER_STORIES.md](doc/USER_STORIES.md)** - Complete user stories and requirements
- **[algebra/README.md](doc/algebra/README.md)** - Mathematical algebra foundations (56KB)
- **[DOCUMENTATION_STATUS.md](doc/DOCUMENTATION_STATUS.md)** - Documentation status and roadmap

## Examples

The `/examples` directory contains working examples:

- **[adding_attributes.rs](examples/adding_attributes.rs)** - Complete attribute addition workflows
- **[pure_event_driven_demo.rs](examples/pure_event_driven_demo.rs)** - Event sourcing patterns

Run examples with:

```bash
cargo run --example adding_attributes
cargo run --example pure_event_driven_demo
```

## Testing

Comprehensive test suite with 194 tests covering all aspects:

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test attribute_addition_tests
cargo test --test person_aggregate_tests
cargo test --test person_attribute_tests
cargo test --test person_name_tests
```

**Test Coverage**:
- 91 library unit tests
- 6 attribute addition tests
- 20 person aggregate tests
- 33 person attribute tests
- 40 person name tests
- 4 doc tests

## Performance

The domain is optimized for performance with pure functional patterns:

- Zero heap allocations in hot paths
- Lazy attribute evaluation
- Efficient event replay with snapshots
- Streaming for large result sets

Benchmarks:

```bash
cargo bench
```

## FRP/CT Compliance

The codebase maintains **100% compliance** with Functional Reactive Programming and Category Theory principles:

✅ Pure functions with no side effects in domain logic
✅ All state changes through immutable events
✅ Infrastructure at boundaries only (Hexagonal Architecture)
✅ Formal Category Theory traits (Functor, Monad, Coalgebra)
✅ Pure projection functions: (State, Event) → NewState
✅ Explicit CQRS with compile-time safety
✅ Event sourcing as single source of truth

See [FRP-CT-COMPLIANCE.md](doc/FRP-CT-COMPLIANCE.md) for complete details.

## Version

**Current Version**: 0.7.8

Compatible with:
- `cim-domain`: ^0.7.0
- Rust: 1.70+

## Contributing

Contributions are welcome! This domain follows strict architectural principles:

1. **Pure Functional**: All domain logic must be pure functions
2. **Event Sourcing**: All state changes through events
3. **CQRS**: Explicit command/query separation
4. **Category Theory**: Maintain mathematical rigor
5. **Zero Warnings**: Clean builds required

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Copyright 2025 Cowboy AI, LLC.

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Related CIM Modules

- **cim-domain** - Core domain framework and Category Theory traits
- **cim-domain-document** - Document management domain
- **cim-domain-location** - Location and spatial domain
- **cim-start** - Template for creating new CIM domains

## Support

- 📖 [Documentation](doc/README.md)
- 🐛 [Issue Tracker](https://github.com/thecowboyai/cim-domain-person/issues)
- 💬 [Discussions](https://github.com/thecowboyai/cim-domain-person/discussions)
