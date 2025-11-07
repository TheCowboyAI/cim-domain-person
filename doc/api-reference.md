# API Reference

Complete API reference for the cim-domain-person library.

## Table of Contents

- [Core Types](#core-types)
- [Commands](#commands)
- [Events](#events)
- [Queries](#queries)
- [Value Objects](#value-objects)
- [Projections](#projections)
- [Services](#services)
- [Usage Examples](#usage-examples)

## Core Types

### Person Aggregate

The core domain aggregate representing a person.

```rust
pub struct Person {
    pub id: PersonId,
    pub core_identity: CoreIdentity,
    pub attributes: PersonAttributeSet,
    pub lifecycle: PersonLifecycle,
    pub version: u64,
}

impl Person {
    /// Create a new person
    pub fn new(id: PersonId, name: PersonName) -> Self;

    /// Apply an event to get new state (pure function)
    pub fn apply_event_pure(&self, event: &PersonEvent) -> DomainResult<Self>;

    /// Get identifying attributes
    pub fn identifying_attributes(&self) -> PersonAttributeSet;

    /// Unfold state for observation (Coalgebra)
    pub fn unfold(&self) -> PersonAttributeSet;
}
```

### PersonId

Unique identifier for persons.

```rust
pub type PersonId = EntityId<PersonMarker>;

impl PersonId {
    /// Create a new UUID v7 identifier
    pub fn new() -> Self;

    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, ParseError>;
}
```

### CoreIdentity

Core immutable identity information.

```rust
pub struct CoreIdentity {
    pub legal_name: PersonName,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### PersonLifecycle

Lifecycle state of a person.

```rust
pub enum PersonLifecycle {
    Active,
    Deactivated { reason: String, since: DateTime<Utc> },
    Deceased { death_date: NaiveDate },
    Merged { into_person_id: PersonId, merged_at: DateTime<Utc> },
}

impl PersonLifecycle {
    pub fn is_active(&self) -> bool;
    pub fn can_transition_to(&self, new_state: &PersonLifecycle) -> bool;
}
```

## Commands

All commands follow the pure functional pattern: Command → Events → New State.

### PersonCommand

Top-level command enumeration.

```rust
pub enum PersonCommand {
    CreatePerson(CreatePerson),
    UpdateName(UpdateName),
    RecordAttribute(RecordAttribute),
    UpdateAttribute(UpdateAttribute),
    InvalidateAttribute(InvalidateAttribute),
    DeactivatePerson(DeactivatePerson),
    ReactivatePerson(ReactivatePerson),
    MergePerson(MergePerson),
}
```

### CreatePerson

Create a new person entity.

```rust
pub struct CreatePerson {
    pub person_id: PersonId,
    pub name: PersonName,
    pub source: String,
}
```

**Example:**
```rust
use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::{PersonCommand, CreatePerson},
    value_objects::PersonName,
};

let person_id = PersonId::new();
let name = PersonName::new("Alice".to_string(), "Johnson".to_string());

let command = PersonCommand::CreatePerson(CreatePerson {
    person_id,
    name,
    source: "registration_api".to_string(),
});
```

**Events Produced:** `PersonEvent::PersonCreated`

### UpdateName

Update a person's legal name.

```rust
pub struct UpdateName {
    pub person_id: PersonId,
    pub new_name: PersonName,
}
```

**Example:**
```rust
let command = PersonCommand::UpdateName(UpdateName {
    person_id,
    new_name: PersonName::new("Alice".to_string(), "Smith".to_string()),
});
```

**Events Produced:** `PersonEvent::NameUpdated`

### RecordAttribute

Add or update a person attribute.

```rust
pub struct RecordAttribute {
    pub person_id: PersonId,
    pub attribute: PersonAttribute,
}
```

**Example:**
```rust
use cim_domain_person::{
    commands::RecordAttribute,
    value_objects::{
        PersonAttribute, AttributeType, AttributeValue,
        PhysicalAttributeType, TemporalValidity, Provenance,
        AttributeSource, ConfidenceLevel,
    },
};
use chrono::{Utc, NaiveDate};

// Record height attribute
let attribute = PersonAttribute::new(
    AttributeType::Physical(PhysicalAttributeType::Height),
    AttributeValue::Length(1.75),  // meters
    TemporalValidity::of(Utc::now()),
    Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
);

let command = PersonCommand::RecordAttribute(RecordAttribute {
    person_id,
    attribute,
});
```

**Events Produced:** `PersonEvent::AttributeRecorded`

### UpdateAttribute

Update an existing attribute.

```rust
pub struct UpdateAttribute {
    pub person_id: PersonId,
    pub attribute_type: AttributeType,
    pub new_value: AttributeValue,
    pub provenance: Provenance,
}
```

**Example:**
```rust
let command = PersonCommand::UpdateAttribute(UpdateAttribute {
    person_id,
    attribute_type: AttributeType::Physical(PhysicalAttributeType::Weight),
    new_value: AttributeValue::Mass(72.0),
    provenance: Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
});
```

**Events Produced:** `PersonEvent::AttributeUpdated`

### InvalidateAttribute

Mark an attribute as invalid.

```rust
pub struct InvalidateAttribute {
    pub person_id: PersonId,
    pub attribute_type: AttributeType,
    pub reason: Option<String>,
}
```

**Example:**
```rust
let command = PersonCommand::InvalidateAttribute(InvalidateAttribute {
    person_id,
    attribute_type: AttributeType::Healthcare(HealthcareAttributeType::BloodType),
    reason: Some("Test error - need reconfirmation".to_string()),
});
```

**Events Produced:** `PersonEvent::AttributeInvalidated`

### DeactivatePerson

Deactivate a person account.

```rust
pub struct DeactivatePerson {
    pub person_id: PersonId,
    pub reason: String,
}
```

**Example:**
```rust
let command = PersonCommand::DeactivatePerson(DeactivatePerson {
    person_id,
    reason: "User requested account closure".to_string(),
});
```

**Events Produced:** `PersonEvent::PersonDeactivated`

### ReactivatePerson

Reactivate a previously deactivated person.

```rust
pub struct ReactivatePerson {
    pub person_id: PersonId,
}
```

**Events Produced:** `PersonEvent::PersonReactivated`

### MergePerson

Merge duplicate person records.

```rust
pub struct MergePerson {
    pub person_id: PersonId,
    pub into_person_id: PersonId,
    pub merge_reason: String,
}
```

**Events Produced:** `PersonEvent::PersonMergedInto`

## Events

All events are immutable records of state changes.

### PersonEvent

Core event enumeration.

```rust
pub enum PersonEvent {
    PersonCreated {
        person_id: PersonId,
        name: PersonName,
        source: String,
        created_at: DateTime<Utc>,
    },

    NameUpdated {
        person_id: PersonId,
        old_name: PersonName,
        new_name: PersonName,
        updated_at: DateTime<Utc>,
    },

    AttributeRecorded {
        person_id: PersonId,
        attribute: PersonAttribute,
        recorded_at: DateTime<Utc>,
    },

    AttributeUpdated {
        person_id: PersonId,
        attribute_type: AttributeType,
        old_value: AttributeValue,
        new_value: AttributeValue,
        updated_at: DateTime<Utc>,
    },

    AttributeInvalidated {
        person_id: PersonId,
        attribute_type: AttributeType,
        reason: Option<String>,
        invalidated_at: DateTime<Utc>,
    },

    BirthDateSet {
        person_id: PersonId,
        birth_date: NaiveDate,
        set_at: DateTime<Utc>,
    },

    PersonDeactivated {
        person_id: PersonId,
        reason: String,
        deactivated_at: DateTime<Utc>,
    },

    PersonReactivated {
        person_id: PersonId,
        reactivated_at: DateTime<Utc>,
    },

    PersonDeceased {
        person_id: PersonId,
        death_date: NaiveDate,
    },

    PersonMergedInto {
        person_id: PersonId,
        target_id: PersonId,
        merge_reason: String,
        merged_at: DateTime<Utc>,
    },
}
```

### Event Application

Events are applied using pure functions:

```rust
let person = person.apply_event_pure(&event)?;
```

## Queries

All queries use immutable specification objects and return read models.

### PersonService

Top-level CQRS service.

```rust
pub struct PersonService {
    commands: Arc<PersonCommandProcessor>,
    queries: Arc<PersonQueryService>,
}

impl PersonService {
    // Command Side (Writes)
    pub async fn execute_command(
        &self,
        command: PersonCommand
    ) -> DomainResult<CommandResult>;

    // Query Side (Reads)
    pub async fn query_summaries(
        &self,
        query: &PersonSummaryQuery
    ) -> DomainResult<Vec<PersonSummary>>;

    pub async fn search_persons(
        &self,
        query: &PersonSearchQuery
    ) -> DomainResult<Vec<PersonSearchResult>>;

    pub async fn query_skills(
        &self,
        query: &SkillsQuery
    ) -> DomainResult<Vec<SkillSummary>>;

    pub async fn query_network(
        &self,
        query: &NetworkQuery
    ) -> DomainResult<Vec<PersonRelationship>>;

    pub async fn query_timeline(
        &self,
        query: &TimelineQuery
    ) -> DomainResult<Vec<TimelineEntry>>;
}
```

### Query Specifications

#### PersonSummaryQuery

Query person summaries with pagination.

```rust
pub struct PersonSummaryQuery {
    pub person_ids: Option<Vec<PersonId>>,
    pub employer: Option<String>,
    pub page: usize,
    pub page_size: usize,
}

impl PersonSummaryQuery {
    pub fn all() -> Self;
    pub fn for_person(person_id: PersonId) -> Self;
    pub fn by_employer(employer: String) -> Self;
    pub fn paginate(self, page: usize, page_size: usize) -> Self;
}
```

**Example:**
```rust
use cim_domain_person::queries::PersonSummaryQuery;

// Get all persons, paginated
let query = PersonSummaryQuery::all()
    .paginate(0, 20);

let summaries = service.query_summaries(&query).await?;
```

#### PersonSearchQuery

Full-text search with filters.

```rust
pub struct PersonSearchQuery {
    pub query_text: Option<String>,
    pub employer_filter: Option<String>,
    pub skill_filter: Option<String>,
    pub location_filter: Option<String>,
    pub min_relevance: f64,
    pub limit: usize,
}

impl PersonSearchQuery {
    pub fn new(query: &str) -> Self;
    pub fn with_employer(self, employer: &str) -> Self;
    pub fn with_skill(self, skill: &str) -> Self;
    pub fn with_location(self, location: &str) -> Self;
    pub fn with_min_relevance(self, score: f64) -> Self;
    pub fn limit(self, limit: usize) -> Self;
}
```

**Example:**
```rust
use cim_domain_person::queries::PersonSearchQuery;

let query = PersonSearchQuery::new("software engineer")
    .with_employer("Acme Corp")
    .with_skill("Rust")
    .with_min_relevance(0.8)
    .limit(50);

let results = service.search_persons(&query).await?;
```

#### SkillsQuery

Query skills and certifications.

```rust
pub struct SkillsQuery {
    pub person_id: Option<PersonId>,
    pub skill_name: Option<String>,
    pub category: Option<String>,
}

impl SkillsQuery {
    pub fn for_person(person_id: PersonId) -> Self;
    pub fn by_skill(skill_name: String) -> Self;
    pub fn with_category(self, category: String) -> Self;
}
```

#### NetworkQuery

Query relationships and connections.

```rust
pub struct NetworkQuery {
    pub person_id: PersonId,
    pub relationship_type: Option<RelationshipType>,
    pub include_outgoing: bool,
    pub include_incoming: bool,
}

impl NetworkQuery {
    pub fn for_person(person_id: PersonId) -> Self;
    pub fn of_type(self, rel_type: RelationshipType) -> Self;
    pub fn outgoing_only(self) -> Self;
    pub fn incoming_only(self) -> Self;
}
```

#### TimelineQuery

Query event timeline.

```rust
pub struct TimelineQuery {
    pub person_id: PersonId,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub event_types: Option<Vec<String>>,
    pub limit: usize,
    pub ascending: bool,
}

impl TimelineQuery {
    pub fn for_person(person_id: PersonId) -> Self;
    pub fn date_range(self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self;
    pub fn event_types(self, types: Vec<String>) -> Self;
    pub fn limit(self, limit: usize) -> Self;
    pub fn ascending(self) -> Self;
    pub fn descending(self) -> Self;
}
```

## Value Objects

### PersonName

Represents a person's name with international support.

```rust
pub struct PersonName {
    pub given_names: Vec<String>,
    pub family_names: Vec<String>,
    pub particles: Vec<String>,
    pub suffixes: Vec<String>,
    pub titles: Vec<TitleWithValidity>,
    pub convention: NamingConvention,
}

impl PersonName {
    /// Create a simple Western name
    pub fn new(given_name: String, family_name: String) -> Self;

    /// Create a mononym (single name)
    pub fn mononym(name: String) -> Self;

    /// Parse from string with convention detection
    pub fn parse(full_name: &str) -> DomainResult<Self>;

    /// Parse with explicit convention
    pub fn parse_with_convention(
        full_name: &str,
        convention: NamingConvention
    ) -> DomainResult<Self>;

    /// Get display name
    pub fn display_name(&self) -> String;

    /// Get full formal name
    pub fn full_name(&self) -> String;
}
```

**Example:**
```rust
use cim_domain_person::value_objects::PersonName;

// Simple Western name
let name = PersonName::new("Alice".to_string(), "Johnson".to_string());

// Parse from string
let name = PersonName::parse("Pablo Ruiz Picasso")?;

// Mononym
let name = PersonName::mononym("Cher".to_string());
```

### PersonAttribute

Represents a person attribute with provenance.

```rust
pub struct PersonAttribute {
    pub attribute_type: AttributeType,
    pub value: AttributeValue,
    pub temporal: TemporalValidity,
    pub provenance: Provenance,
}

impl PersonAttribute {
    pub fn new(
        attribute_type: AttributeType,
        value: AttributeValue,
        temporal: TemporalValidity,
        provenance: Provenance,
    ) -> Self;

    /// Transform value (Functor)
    pub fn map<F>(self, f: F) -> Self
    where F: Fn(AttributeValue) -> AttributeValue;

    /// Check if currently valid
    pub fn is_currently_valid(&self) -> bool;

    /// Check if valid on specific date
    pub fn is_valid_on(&self, date: NaiveDate) -> bool;
}
```

### AttributeType

Categories of person attributes.

```rust
pub enum AttributeType {
    Identifying(IdentifyingAttributeType),
    Physical(PhysicalAttributeType),
    Healthcare(HealthcareAttributeType),
    Demographic(DemographicAttributeType),
    Custom(CustomAttributeType),
}

pub enum IdentifyingAttributeType {
    BirthDate,
    NationalId,
    PassportNumber,
    DriverLicense,
    SocialSecurityNumber,
}

pub enum PhysicalAttributeType {
    Height,
    Weight,
    EyeColor,
    HairColor,
    BloodType,
    Handedness,
}

pub enum HealthcareAttributeType {
    MedicalRecordNumber,
    InsuranceId,
    OrganDonor,
    HealthcareProvider,
}

pub enum DemographicAttributeType {
    Nationality,
    Ethnicity,
    Language,
    Religion,
    MaritalStatus,
}
```

### AttributeValue

Strongly-typed attribute values.

```rust
pub enum AttributeValue {
    Text(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Length(f64),      // meters
    Mass(f64),        // kilograms
    Temperature(f64), // celsius
    Structured(serde_json::Value),
}
```

### Provenance

Data provenance and confidence tracking.

```rust
pub struct Provenance {
    pub source: AttributeSource,
    pub confidence: ConfidenceLevel,
    pub recorded_at: DateTime<Utc>,
    pub transformation_history: Vec<TransformationRecord>,
}

pub enum AttributeSource {
    DocumentVerified,
    Measured,
    SelfReported,
    Imported { system: String },
    Inferred { method: String },
}

pub enum ConfidenceLevel {
    Certain,
    Likely,
    Possible,
    Speculative,
}
```

### PersonAttributeSet

Collection of attributes with filtering.

```rust
pub struct PersonAttributeSet {
    pub attributes: Vec<PersonAttribute>,
}

impl PersonAttributeSet {
    pub fn new() -> Self;
    pub fn empty() -> Self;

    /// Add attribute
    pub fn add(mut self, attribute: PersonAttribute) -> Self;

    /// Filter attributes
    pub fn filter<F>(self, predicate: F) -> Self
    where F: Fn(&PersonAttribute) -> bool;

    /// Get attributes by category
    pub fn identifying_attributes(&self) -> Self;
    pub fn physical_attributes(&self) -> Self;
    pub fn healthcare_attributes(&self) -> Self;
    pub fn demographic_attributes(&self) -> Self;

    /// Find specific attribute
    pub fn find_by_type(&self, attr_type: &AttributeType) -> Option<&PersonAttribute>;

    /// Count attributes
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

## Projections

### Read Models

#### PersonSummary

Aggregated person view for listings.

```rust
pub struct PersonSummary {
    pub person_id: PersonId,
    pub name: String,
    pub is_active: bool,
    pub attribute_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### PersonSearchResult

Search result with relevance scoring.

```rust
pub struct PersonSearchResult {
    pub person_id: PersonId,
    pub name: String,
    pub employer: Option<String>,
    pub location: Option<String>,
    pub relevance_score: f64,
}
```

#### SkillSummary

Skills with proficiency.

```rust
pub struct SkillSummary {
    pub person_id: PersonId,
    pub skill_name: String,
    pub category: String,
    pub proficiency: Option<f64>,
    pub years_experience: Option<u32>,
}
```

#### PersonRelationship

Network connections.

```rust
pub struct PersonRelationship {
    pub from_person_id: PersonId,
    pub to_person_id: PersonId,
    pub relationship_type: RelationshipType,
    pub established_at: DateTime<Utc>,
}
```

#### TimelineEntry

Event timeline entry.

```rust
pub struct TimelineEntry {
    pub person_id: PersonId,
    pub event_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

## Services

### PersonCommandProcessor

Infrastructure adapter for command processing.

```rust
pub struct PersonCommandProcessor {
    event_store: Arc<dyn EventStore>,
    nats_client: Arc<Client>,
}

impl PersonCommandProcessor {
    pub async fn process_command(
        &self,
        command: PersonCommand
    ) -> DomainResult<CommandResult>;

    pub async fn process_command_with_correlation(
        &self,
        command: PersonCommand,
        correlation_id: Uuid,
    ) -> DomainResult<CommandResult>;
}
```

### PersonQueryService

Query service for read models.

```rust
pub struct PersonQueryService {
    summaries: Arc<PersonSummaryProjection>,
    search: Arc<PersonSearchProjection>,
    skills: Arc<PersonSkillsProjection>,
    network: Arc<PersonNetworkProjection>,
    timeline: Arc<PersonTimelineProjection>,
}

impl PersonQueryService {
    pub async fn get_person_summary(
        &self,
        person_id: &PersonId
    ) -> Option<PersonSummary>;

    pub async fn get_all_summaries(&self) -> Vec<PersonSummary>;

    pub async fn search_with_filters(
        &self,
        query: Option<&str>,
        employer: Option<&str>,
        skill: Option<&str>,
        location: Option<&str>,
        limit: usize,
    ) -> Vec<PersonSearchResult>;
}
```

## Usage Examples

### Basic Person Creation

```rust
use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::{PersonCommand, CreatePerson},
    value_objects::PersonName,
};
use cim_domain::formal_domain::MealyStateMachine;

// Create person
let person_id = PersonId::new();
let name = PersonName::new("Alice".to_string(), "Johnson".to_string());
let mut person = Person::new(person_id, name);

println!("Created person: {}", person.id);
println!("Version: {}", person.version);
```

### Adding Attributes via Event Sourcing

```rust
use cim_domain_person::{
    aggregate::Person,
    commands::{PersonCommand, RecordAttribute},
    value_objects::{
        PersonAttribute, AttributeType, AttributeValue,
        IdentifyingAttributeType, TemporalValidity, Provenance,
        AttributeSource, ConfidenceLevel,
    },
};
use chrono::{Utc, NaiveDate};
use cim_domain::formal_domain::MealyStateMachine;

// Create attribute
let attribute = PersonAttribute::new(
    AttributeType::Identifying(IdentifyingAttributeType::BirthDate),
    AttributeValue::Date(NaiveDate::from_ymd_opt(1990, 5, 15).unwrap()),
    TemporalValidity::of(Utc::now()),
    Provenance::new(AttributeSource::DocumentVerified, ConfidenceLevel::Certain),
);

// Create command
let command = PersonCommand::RecordAttribute(RecordAttribute {
    person_id: person.id,
    attribute,
});

// Process command → events
let current_state = person.lifecycle.clone();
let events = MealyStateMachine::output(&person, current_state.into(), command);

// Apply events → new state
for event in &events {
    person = person.apply_event_pure(event)?;
}

println!("Attributes: {}", person.attributes.len());
```

### Using CQRS Service

```rust
use cim_domain_person::{
    services::PersonService,
    commands::{PersonCommand, CreatePerson},
    queries::PersonSummaryQuery,
    value_objects::PersonName,
    aggregate::PersonId,
};

// Initialize service
let service = PersonService::new(command_processor, query_service);

// Execute command (write)
let person_id = PersonId::new();
let command = PersonCommand::CreatePerson(CreatePerson {
    person_id,
    name: PersonName::new("Alice".to_string(), "Johnson".to_string()),
    source: "api".to_string(),
});

service.execute_command(command).await?;

// Execute query (read)
let query = PersonSummaryQuery::for_person(person_id);
let summaries = service.query_summaries(&query).await?;

for summary in summaries {
    println!("Person: {} ({})", summary.name, summary.person_id);
}
```

### Querying with Filters

```rust
use cim_domain_person::queries::PersonSearchQuery;

// Complex search query
let query = PersonSearchQuery::new("software engineer")
    .with_employer("Acme Corp")
    .with_skill("Rust")
    .with_location("San Francisco")
    .with_min_relevance(0.7)
    .limit(100);

let results = service.search_persons(&query).await?;

for result in results {
    println!(
        "{} at {} - relevance: {:.2}",
        result.name,
        result.employer.unwrap_or_default(),
        result.relevance_score
    );
}
```

### Working with Attribute Filters

```rust
// Get all physical attributes
let physical = person.attributes.physical_attributes();
println!("Physical attributes: {}", physical.len());

// Get specific attribute
let height = person.attributes.find_by_type(
    &AttributeType::Physical(PhysicalAttributeType::Height)
);

if let Some(attr) = height {
    if let AttributeValue::Length(meters) = attr.value {
        println!("Height: {:.2}m", meters);
    }
}

// Filter by validity
let current_attrs = person.attributes.filter(|a| a.is_currently_valid());
println!("Currently valid: {}", current_attrs.len());
```

### Category Theory Operations

```rust
use cim_domain_person::category_theory::{Functor, Monad, Coalgebra};

// Functor: Transform attribute values
let enhanced = attribute.fmap(|value| {
    // Transform value while preserving structure
    enhance(value)
});

// Monad: Compose operations
let result = PersonAttributeSet::pure(attribute)
    .bind(|a| validate(a))
    .bind(|a| transform(a))
    .bind(|a| enrich(a));

// Coalgebra: Observe state
let attributes = person.unfold();  // Non-destructive observation
```

## Error Handling

### DomainError

Core error type.

```rust
use cim_domain::DomainError;

match result {
    Ok(person) => println!("Success: {}", person.id),
    Err(DomainError::ValidationError { field, message }) => {
        eprintln!("Validation failed on {}: {}", field, message);
    }
    Err(DomainError::NotFound { entity_type, id }) => {
        eprintln!("{} not found: {}", entity_type, id);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## See Also

- [Architecture Guide](architecture.md) - System architecture and design
- [FRP-CT Compliance](FRP-CT-COMPLIANCE.md) - 100% compliance details
- [Person Attributes Design](person-attributes-design.md) - Attribute system design
- [Person Names Design](person-names-design.md) - Name handling and international support
- [User Stories](USER_STORIES.md) - Complete user stories and requirements
