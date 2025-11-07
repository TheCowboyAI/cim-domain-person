# Architecture Guide

## System Architecture

The CIM Domain Person implements a **pure functional reactive** (FRP) architecture with event sourcing, CQRS, and Category Theory foundations. The design emphasizes mathematical rigor, zero side effects in domain logic, and complete separation of concerns.

## Core Design Principles

### 1. Pure Functional Event Sourcing
All state changes flow through immutable events with pure functions:
```
Command → MealyStateMachine → Events → Event Store
                                 ↓
                            apply_event_pure → New State
                                 ↓
                         Pure Projections → Read Models
```

**Key Properties**:
- Zero mutations in domain logic
- All functions are pure: same inputs → same outputs
- Side effects only at infrastructure boundaries
- Event replay reconstructs exact state

### 2. CQRS Pattern
Complete separation of reads and writes:
```
┌──────────────────────────────────────┐
│         PersonService                │
│  ┌────────────┐    ┌──────────────┐ │
│  │  Commands  │    │   Queries    │ │
│  │  (Writes)  │    │   (Reads)    │ │
│  └──────┬─────┘    └──────┬───────┘ │
└─────────┼──────────────────┼─────────┘
          │                  │
          v                  v
   ┌─────────────┐    ┌────────────┐
   │   Domain    │    │ Read Models│
   │ Aggregates  │    │(Projections)│
   │  + Events   │    │            │
   └──────┬──────┘    └─────▲──────┘
          │                  │
          v                  │
   ┌─────────────────────────┼──────┐
   │  Infrastructure Layer   │      │
   │   (Event Store, NATS)   ├──────┘
   └─────────────────────────┘
```

- **Commands**: Modify state through event generation (write side)
- **Queries**: Read from optimized projections (read side)
- **Event Store**: Single source of truth
- **No shared models**: Complete separation enforced at compile time

### 3. Category Theory Foundations
Formal mathematical structures ensure compositional correctness:

- **Functors**: Structure-preserving transformations
- **Monads**: Compositional operations that might fail
- **Coalgebras**: State observation without mutation
- **Natural Transformations**: Cross-domain mappings

See [person-attributes-category-theory.md](person-attributes-category-theory.md) for mathematical details.

## Domain Model

### Person Aggregate (`aggregate/person_ecs.rs`)

The Person aggregate follows a minimalist design with extensible attributes:

```rust
pub struct Person {
    /// Unique identifier
    pub id: PersonId,

    /// Core immutable identity
    pub core_identity: CoreIdentity {
        legal_name: PersonName,
        birth_date: Option<NaiveDate>,
        death_date: Option<NaiveDate>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },

    /// Extensible attributes (EAV pattern)
    pub attributes: PersonAttributeSet {
        attributes: Vec<PersonAttribute>,
    },

    /// Lifecycle state
    pub lifecycle: PersonLifecycle,

    /// Event sourcing version
    pub version: u64,
}
```

**Design Philosophy**:
- **Minimalist Core**: Only intrinsic person properties (ID, name, dates)
- **Extensible Attributes**: EAV pattern for flexible data model
- **Pure Functions**: All methods return new instances, never mutate
- **Event Sourced**: State reconstructed from events

### Attribute System (EAV Pattern)

Person attributes use Entity-Attribute-Value pattern with full provenance:

```rust
pub struct PersonAttribute {
    /// Type of attribute (Identifying, Physical, Healthcare, Demographic, Custom)
    attribute_type: AttributeType,

    /// The actual value
    value: AttributeValue,

    /// Temporal validity (when this value is valid)
    temporal: TemporalValidity,

    /// Data provenance (where it came from)
    provenance: Provenance {
        source: AttributeSource,
        confidence: ConfidenceLevel,
        recorded_at: DateTime<Utc>,
        transformation_history: Vec<TransformationRecord>,
    },
}
```

**Attribute Categories**:
- **Identifying**: BirthDate, NationalId, Passport
- **Physical**: Height, Weight, EyeColor, BloodType
- **Healthcare**: MedicalRecordNumber, OrganDonor
- **Demographic**: Nationality, Ethnicity, Language
- **Custom**: Domain-specific extensions

See [person-attributes-design.md](person-attributes-design.md) for complete design.

### Lifecycle States

```rust
pub enum PersonLifecycle {
    Active,
    Deactivated { reason: String, since: DateTime<Utc> },
    Deceased { death_date: NaiveDate },
    Merged { into_person_id: PersonId, merged_at: DateTime<Utc> },
}
```

## Event System

### Event Types (`events/mod.rs`)

All state changes are captured as immutable events:

```rust
pub enum PersonEvent {
    // Core lifecycle
    PersonCreated { person_id, name, source, created_at },
    NameUpdated { person_id, old_name, new_name, updated_at },

    // Attribute management
    AttributeRecorded { person_id, attribute, recorded_at },
    AttributeUpdated { person_id, attribute_type, old_value, new_value, updated_at },
    AttributeInvalidated { person_id, attribute_type, reason, invalidated_at },

    // Lifecycle changes
    BirthDateSet { person_id, birth_date, set_at },
    PersonDeactivated { person_id, reason, deactivated_at },
    PersonReactivated { person_id, reactivated_at },
    PersonDeceased { person_id, death_date },
    PersonMergedInto { person_id, target_id, merge_reason, merged_at },
}
```

### Event Sourcing Pattern

Pure functional state transitions:

```rust
// 1. Command → Events (pure function via MealyStateMachine)
let current_state = person.lifecycle.clone();
let events = MealyStateMachine::output(&person, current_state.into(), command);

// 2. Events → New State (pure function)
for event in &events {
    person = person.apply_event_pure(event)?;
}

// No mutations, only new values returned
```

### Event Versioning (`events/versioning.rs`)
- Automatic schema evolution
- Backward compatibility maintained
- Migration registry for version transitions
- V1 → V2 → V3 migration chains supported

## Command Processing

### Command Types (`commands/mod.rs`)

Commands represent intentions to change state:

```rust
pub enum PersonCommand {
    // Core operations
    CreatePerson(CreatePerson),
    UpdateName(UpdateName),

    // Attribute operations
    RecordAttribute(RecordAttribute),
    UpdateAttribute(UpdateAttribute),
    InvalidateAttribute(InvalidateAttribute),

    // Lifecycle operations
    DeactivatePerson(DeactivatePerson),
    ReactivatePerson(ReactivatePerson),
    MergePerson(MergePerson),
}
```

### Command Processing Flow

```rust
// Pure command processing
impl Person {
    pub fn handle_command(
        &self,
        command: PersonCommand
    ) -> DomainResult<Vec<PersonEvent>> {
        // Validate (pure)
        self.validate_command(&command)?;

        // Generate events (pure)
        let events = MealyStateMachine::output(
            self,
            self.lifecycle.clone().into(),
            command
        );

        Ok(events)
    }
}
```

### Async Command Processor (`handlers/async_command_processor.rs`)

Infrastructure adapter for command execution:

```rust
pub struct PersonCommandProcessor {
    event_store: Arc<dyn EventStore>,
    nats_client: Arc<Client>,
}

impl PersonCommandProcessor {
    pub async fn process_command(
        &self,
        command: PersonCommand
    ) -> DomainResult<CommandResult> {
        // Load aggregate (side effect)
        let person = self.load_aggregate(command.aggregate_id()).await?;

        // Handle command (pure)
        let events = person.handle_command(command)?;

        // Persist events (side effect)
        self.event_store.append_events(person.id, events.clone()).await?;

        // Publish to NATS (side effect)
        for event in &events {
            self.publish_event(event).await?;
        }

        Ok(CommandResult { events })
    }
}
```

## Query System (CQRS)

### Query Service (`services/person_service.rs`)

Top-level CQRS service with explicit separation:

```rust
pub struct PersonService {
    commands: Arc<PersonCommandProcessor>,  // Write side
    queries: Arc<PersonQueryService>,       // Read side
}

impl PersonService {
    // Command side (writes)
    pub async fn execute_command(&self, cmd: PersonCommand)
        -> DomainResult<CommandResult>
    {
        self.commands.process_command(cmd).await
    }

    // Query side (reads)
    pub async fn query_summaries(&self, query: &PersonSummaryQuery)
        -> DomainResult<Vec<PersonSummary>>
    {
        // Read from projections
    }
}
```

### Query Specifications (`queries/specifications.rs`)

Immutable value objects describing queries:

```rust
pub struct PersonSummaryQuery {
    person_ids: Option<Vec<PersonId>>,
    employer: Option<String>,
    page: usize,
    page_size: usize,
}

pub struct PersonSearchQuery {
    query_text: Option<String>,
    employer_filter: Option<String>,
    skill_filter: Option<String>,
    min_relevance: f64,
    limit: usize,
}
```

### Read Models

Optimized projections for queries:

- **PersonSummary**: Aggregated person view
- **PersonSearchResult**: Full-text search with relevance
- **SkillSummary**: Skills with proficiency levels
- **PersonRelationship**: Network connections
- **TimelineEntry**: Activity history

## Projections

### Pure Projection Functions (`projections/pure_projections.rs`)

All projections are pure functions:

```rust
/// Pure function: (CurrentState, Event) → NewState
pub fn project_person_summary(
    current: Option<PersonSummary>,
    event: &PersonEvent,
) -> Option<PersonSummary> {
    match event {
        PersonEvent::PersonCreated(e) => {
            Some(PersonSummary {
                person_id: e.person_id,
                name: e.name.display_name(),
                created_at: e.created_at,
                // ... pure computation only
            })
        }
        PersonEvent::AttributeRecorded(e) => {
            current.map(|mut summary| {
                summary.add_attribute(e.attribute.clone());
                summary  // Returns new value
            })
        }
        // ...
    }
}
```

### Projection Infrastructure

Infrastructure applies pure functions:

```rust
pub struct PersonSummaryProjection {
    summaries: Arc<RwLock<HashMap<PersonId, PersonSummary>>>,
}

impl PersonSummaryProjection {
    pub async fn handle_event(&self, event: &PersonEvent)
        -> DomainResult<()>
    {
        let person_id = extract_person_id(event);

        // Load current state (side effect)
        let current = {
            let summaries = self.summaries.read().await;
            summaries.get(&person_id).cloned()
        };

        // Apply pure projection (no side effects)
        let new_state = project_person_summary(current, event);

        // Save new state (side effect)
        let mut summaries = self.summaries.write().await;
        match new_state {
            Some(summary) => { summaries.insert(person_id, summary); }
            None => { summaries.remove(&person_id); }
        }

        Ok(())
    }
}
```

### Available Projections (`projections/mod.rs`)

- **PersonSummaryProjection**: Basic person information
- **PersonSearchProjection**: Full-text search index
- **PersonTimelineProjection**: Activity history
- **PersonNetworkProjection**: Relationship graphs
- **PersonSkillsProjection**: Competency matrix

## Infrastructure

### Event Store (`infrastructure/event_store.rs`)
- Event persistence with append-only log
- Snapshot storage for performance
- Event streaming for projections
- Optimistic concurrency control

### NATS Integration (`infrastructure/nats_integration.rs`)
- JetStream for durable event streaming
- Subject-based routing
- Consumer groups for scalability
- Message correlation and causation tracking

### Retry & Circuit Breaker (`infrastructure/retry.rs`)
- Exponential backoff for transient failures
- Circuit breaker pattern for fault isolation
- Automatic recovery mechanisms
- Dead letter queue for failed events

## Data Flow

### Command Flow (Write Path)

1. Command received via PersonService
2. Validated by command handler
3. **Pure processing**: MealyStateMachine generates events
4. Events persisted to event store (side effect)
5. Events published to NATS (side effect)
6. Projections updated asynchronously

**Key Point**: Steps 1-3 are pure functions with zero side effects

### Query Flow (Read Path)

1. Query specification created (immutable value object)
2. Routed to PersonService query methods
3. Data retrieved from read model (projection)
4. Results returned to client

**Key Point**: Reads never touch event store or domain aggregates

## Integration Points

### Cross-Domain Communication (`cross_domain/mod.rs`)

Integration with other CIM domains:

- **Location Domain**: Address and geographic data
- **Identity Domain**: Authentication and authorization
- **Organization Domain**: Employment relationships
- **Document Domain**: Associated documents

All cross-domain calls use event-driven integration (no direct coupling).

### External Systems

- NATS JetStream for event streaming
- Databases for projection storage
- External APIs via adapters (side effects at boundaries)

## Category Theory Architecture

### Functor Operations

Structure-preserving transformations:

```rust
// Transform attribute while preserving structure
let enhanced = attribute.fmap(|value| enrich(value));

// Composition law holds
assert_eq!(
    attr.fmap(|v| g(f(v))),
    attr.fmap(f).fmap(g)
);
```

### Monad Operations

Compositional operations:

```rust
// Compose operations that might fail
let result = PersonAttributeSet::pure(attr)
    .bind(|a| validate(a))
    .bind(|a| transform(a))
    .bind(|a| enrich(a));
```

### Coalgebra Operations

State observation:

```rust
// Unfold person state for observation
let attributes = person.unfold();  // Returns PersonAttributeSet

// Original remains unchanged
assert_eq!(person, original_person);
```

See [FRP-CT-COMPLIANCE.md](FRP-CT-COMPLIANCE.md) for complete Category Theory details.

## Scalability Considerations

### Horizontal Scaling

- **Stateless command processors**: Can run on any node
- **Partitioned event streams**: Sharded by aggregate ID
- **Distributed projections**: Each projection can run independently
- **NATS clustering**: Built-in horizontal scaling

### Performance Optimizations

- **Attribute lazy evaluation**: Load only needed attributes
- **Event batching**: Process multiple events together
- **Projection caching**: Cache frequently accessed views
- **Streaming for large datasets**: Avoid loading everything in memory
- **Snapshot support**: Avoid replaying all events

### Load Patterns

```
Write Load: Commands → Event Store (append-only, fast)
Read Load: Queries → Projections (optimized read models)
```

Reads and writes scale independently due to CQRS.

## Security Architecture

### Authentication & Authorization

- Command-level authorization checks
- Event metadata includes actor information
- Audit trail via event log
- Immutable audit history

### Data Protection

- **PII handling**: Attributes marked with sensitivity levels
- **GDPR compliance**: Event sourcing enables right to erasure
- **Retention policies**: Time-based event archival
- **Encryption**: At-rest and in-transit for sensitive data

### Privacy by Design

- Attribute-level privacy controls
- Consent management via events
- Data export for portability
- Deletion workflow for right to be forgotten

## Deployment Architecture

### Container Architecture

```
┌─────────────────────────────────────┐
│     PersonService Container         │
│  ┌────────────┐  ┌────────────────┐ │
│  │  Commands  │  │    Queries     │ │
│  │ Processor  │  │   Service      │ │
│  └─────┬──────┘  └────────┬───────┘ │
└────────┼──────────────────┼─────────┘
         │                  │
         v                  v
┌─────────────────┐  ┌──────────────┐
│  Event Store    │  │  Projections │
│  (NATS Stream)  │  │  (Database)  │
└─────────────────┘  └──────────────┘
```

### Configuration Management

- Environment-based configuration (dev, staging, prod)
- Feature flags for gradual rollouts
- Dynamic policy loading without redeployment
- Configuration via environment variables

### Health Checks

- Event store connectivity
- NATS connection status
- Projection lag monitoring
- Memory and CPU metrics

## Testing Strategy

### Unit Tests
- Pure functions are trivial to test
- No mocking required for domain logic
- Property-based testing for laws

### Integration Tests
- Event store persistence
- NATS publishing
- Projection updates
- Cross-domain integration

### Test Coverage

```
194 tests total:
- 91 library unit tests
- 6 attribute addition tests
- 20 person aggregate tests
- 33 person attribute tests
- 40 person name tests
- 4 doc tests
```

See examples in `/tests` directory.

## Monitoring & Observability

### Metrics
- Command processing time
- Event persistence latency
- Projection lag
- Query response time

### Tracing
- Correlation ID tracking
- Causation chain analysis
- Distributed tracing via NATS headers

### Logging
- Structured logging with context
- Event replay for debugging
- Error aggregation

## Summary

The cim-domain-person architecture achieves:

- ✅ **100% FRP Compliance**: Pure functions throughout domain logic
- ✅ **Event Sourcing**: Complete audit trail and state reconstruction
- ✅ **CQRS**: Independent scaling of reads and writes
- ✅ **Category Theory**: Mathematical rigor and compositional correctness
- ✅ **Zero Side Effects**: Infrastructure at boundaries only
- ✅ **Type Safety**: Compile-time guarantees via marker traits
- ✅ **Scalability**: Horizontal scaling via NATS and projections
- ✅ **Maintainability**: Clear separation of concerns

For implementation details, see:
- [FRP-CT-COMPLIANCE.md](FRP-CT-COMPLIANCE.md) - 100% compliance details
- [person-attributes-design.md](person-attributes-design.md) - Attribute system
- [development.md](development.md) - Development guide
