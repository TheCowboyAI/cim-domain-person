# Person Domain

The Person domain provides a comprehensive event-driven system for managing people and their relationships within the CIM (Composable Information Machine) architecture.

## Overview

The Person domain follows Domain-Driven Design principles with a pure event-sourcing approach. All state changes flow through commands that produce events, which are then applied to aggregates. There are no direct mutations or CRUD operations - everything is event-driven.

## Event Replay and Event Sourcing

The Person domain fully supports event sourcing with the following capabilities:

### Event Stream Replay
```rust
// Replay an entire event stream to reconstruct a person
let events = vec![
    PersonEvent::PersonCreated(created_event),
    PersonEvent::EmailAdded(email_event),
    PersonEvent::PhoneAdded(phone_event),
];

let person = Person::replay_events(events)?;
```

### Snapshot Support
```rust
// Create a snapshot and replay only new events
let snapshot = Person::replay_events(initial_events)?;
let snapshot_version = snapshot.version();

// Later, replay from snapshot
let person = Person::replay_from_snapshot(
    snapshot,
    new_events,
    snapshot_version
)?;
```

### Event Store Integration
```rust
// Use the PersonRepository for event store operations
let repo = PersonRepository::new(event_store);

// Save person with events
repo.save(&person, events, expected_version).await?;

// Load person by replaying events
let person = repo.load(person_id).await?;
```

The event store integration includes:
- Automatic snapshot creation based on configurable frequency
- Optimistic concurrency control with version checking
- Support for loading events after a specific version
- In-memory event store for testing

## Architecture

### Domain Model

```rust
pub struct Person {
    pub id: PersonId,
    pub name: PersonName,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Contact information
    pub emails: HashMap<String, EmailAddress>,
    pub phones: HashMap<String, PhoneNumber>,
    pub addresses: HashMap<AddressType, PhysicalAddress>,
    
    // Professional
    pub employments: HashMap<Uuid, Employment>,
    pub skills: HashMap<String, Skill>,
    pub certifications: Vec<Certification>,
    pub education: Vec<Education>,
    
    // Relationships
    pub relationships: HashMap<Uuid, Relationship>,
    
    // Social & Business
    pub social_profiles: HashMap<SocialPlatform, SocialProfile>,
    pub customer_segment: Option<CustomerSegment>,
    pub behavioral_data: Option<BehavioralData>,
    pub communication_preferences: Option<CommunicationPreferences>,
    pub privacy_preferences: Option<PrivacyPreferences>,
    
    // Metadata
    pub tags: Vec<Tag>,
    pub custom_attributes: HashMap<String, CustomAttribute>,
    
    // Merge tracking
    pub merged_into: Option<PersonId>,
    pub merge_status: MergeStatus,
}
```

### Commands → Events Flow

The Person domain processes commands that generate events. Events are immutable facts about what happened.

#### Core Commands

```rust
pub enum PersonCommand {
    // Lifecycle
    CreatePerson(CreatePerson),
    DeactivatePerson(DeactivatePerson),
    ReactivatePerson(ReactivatePerson),
    MergePersons(MergePersons),
    
    // Identity
    UpdateName(UpdateName),
    
    // Contact
    AddEmail(AddEmail),
    RemoveEmail(RemoveEmail),
    VerifyEmail(VerifyEmail),
    AddPhone(AddPhone),
    RemovePhone(RemovePhone),
    AddAddress(AddAddress),
    RemoveAddress(RemoveAddress),
    
    // Professional
    AddEmployment(AddEmployment),
    UpdateEmployment(UpdateEmployment),
    EndEmployment(EndEmployment),
    AddSkill(AddSkill),
    UpdateSkill(UpdateSkill),
    RemoveSkill(RemoveSkill),
    AddCertification(AddCertification),
    AddEducation(AddEducation),
    
    // Relationships
    AddRelationship(AddRelationship),
    UpdateRelationship(UpdateRelationship),
    EndRelationship(EndRelationship),
    
    // Social & Business
    AddSocialProfile(AddSocialProfile),
    UpdateSocialProfile(UpdateSocialProfile),
    RemoveSocialProfile(RemoveSocialProfile),
    SetCustomerSegment(SetCustomerSegment),
    UpdateBehavioralData(UpdateBehavioralData),
    SetCommunicationPreferences(SetCommunicationPreferences),
    SetPrivacyPreferences(SetPrivacyPreferences),
    
    // Metadata
    AddTag(AddTag),
    RemoveTag(RemoveTag),
    SetCustomAttribute(SetCustomAttribute),
}
```

#### Events Generated

Each command produces one or more events that represent the state change:

```rust
pub enum PersonEvent {
    // Lifecycle events
    PersonCreated(PersonCreated),
    PersonDeactivated(PersonDeactivated),
    PersonReactivated(PersonReactivated),
    PersonMergedInto(PersonMergedInto),
    PersonsMerged(PersonsMerged),
    
    // Identity events
    NameUpdated(NameUpdated),
    
    // Contact events
    EmailAdded(EmailAdded),
    EmailRemoved(EmailRemoved),
    EmailVerified(EmailVerified),
    PhoneAdded(PhoneAdded),
    PhoneRemoved(PhoneRemoved),
    AddressAdded(AddressAdded),
    AddressRemoved(AddressRemoved),
    
    // And corresponding events for all other commands...
}
```

### Value Objects

Value objects are immutable and represent concepts without identity:

- **PersonName**: Comprehensive name handling with cultural support
- **EmailAddress**: Email with verification status
- **PhoneNumber**: Phone with country code and capabilities
- **PhysicalAddress**: Structured address information
- **Employment**: Organization affiliation and role
- **Skill**: Skill with proficiency level
- **Relationship**: Connection to another person
- **SocialProfile**: Social media presence
- **CustomerSegment**: Business categorization
- **Tag**: Flexible categorization

## Event-Driven Patterns

### Command Processing

```rust
impl Person {
    pub fn handle_command(&mut self, command: PersonCommand) -> Result<Vec<PersonEvent>, String> {
        match command {
            PersonCommand::AddEmail(cmd) => {
                // Validate business rules
                if self.emails.contains_key(&cmd.email.address) {
                    return Err("Email already exists".to_string());
                }
                
                // Generate event
                let event = PersonEvent::EmailAdded(EmailAdded {
                    person_id: self.id,
                    email: cmd.email,
                    primary: cmd.primary,
                    added_at: Utc::now(),
                });
                
                // Apply event to update state
                self.apply_event(&event);
                
                Ok(vec![event])
            }
            // Other command handlers...
        }
    }
}
```

### Event Application

```rust
impl Person {
    pub fn apply_event(&mut self, event: &PersonEvent) {
        match event {
            PersonEvent::EmailAdded(e) => {
                self.emails.insert(e.email.address.clone(), e.email.clone());
                if e.primary {
                    self.primary_email = Some(e.email.address.clone());
                }
                self.updated_at = e.added_at;
            }
            // Other event handlers...
        }
    }
}
```

### Event Sourcing

All events are persisted to NATS JetStream with CID chains for integrity:

```rust
// Events flow through NATS subjects
person.events.created
person.events.name_updated
person.events.email_added
person.events.relationship_added
// etc.
```

## Usage Examples

### Creating a Person

```rust
// Create command
let command = PersonCommand::CreatePerson(CreatePerson {
    person_id: PersonId::new(),
    name: PersonName::new("John".to_string(), "Doe".to_string()),
    source: "Registration".to_string(),
});

// Process command to generate events
let events = person.handle_command(command)?;

// Events are published to NATS
for event in events {
    event_store.publish("person.events", &event).await?;
}
```

### Adding Contact Information

```rust
// Add email
let add_email = PersonCommand::AddEmail(AddEmail {
    person_id,
    email: EmailAddress::new("john@example.com".to_string()),
    primary: true,
});

// Add phone
let add_phone = PersonCommand::AddPhone(AddPhone {
    person_id,
    phone: PhoneNumber::with_country("555-0123".to_string(), "1".to_string()),
    primary: true,
});

// Process commands
let email_events = person.handle_command(add_email)?;
let phone_events = person.handle_command(add_phone)?;
```

### Managing Relationships

```rust
// Add relationship
let command = PersonCommand::AddRelationship(AddRelationship {
    person_id,
    relationship: Relationship {
        person_id: other_person_id,
        relationship_type: RelationshipType::Colleague,
        status: RelationshipStatus::Active,
        start_date: Utc::now().date_naive(),
        end_date: None,
        notes: Some("Met at conference".to_string()),
    },
});

let events = person.handle_command(command)?;
```

### Person Merging

When duplicate persons are identified, they can be merged:

```rust
let merge_command = PersonCommand::MergePersons(MergePersons {
    source_person_id: duplicate_id,
    target_person_id: primary_id,
    reason: MergeReason::DuplicateIdentity,
});

// This generates two events:
// 1. PersonMergedInto - on the source person
// 2. PersonsMerged - on the target person
```

## Projections

Read models are built from events for efficient querying:

### PersonView
Basic information for display:
```rust
pub struct PersonView {
    pub id: PersonId,
    pub name: PersonName,
    pub primary_email: Option<String>,
    pub primary_phone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ContactView
Complete contact information:
```rust
pub struct ContactView {
    pub person_id: PersonId,
    pub emails: HashMap<String, EmailAddress>,
    pub phones: HashMap<String, PhoneNumber>,
    pub addresses: HashMap<AddressType, PhysicalAddress>,
}
```

### CustomerView
Business-focused view:
```rust
pub struct CustomerView {
    pub person_id: PersonId,
    pub name: String,
    pub segment: Option<SegmentType>,
    pub value_tier: Option<ValueTier>,
    pub lifetime_value: Option<f32>,
    pub engagement_score: Option<f32>,
}
```

## Integration Points

### NATS Subjects
- Commands: `person.commands.*`
- Events: `person.events.*`
- Queries: `person.queries.*`

### Cross-Domain Integration
- **Organization**: Employment relationships
- **Location**: Physical addresses
- **Identity**: Authentication and authorization
- **Workflow**: Task assignments

## Testing

The domain includes comprehensive tests for all command handlers and event applications:

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test person_comprehensive_tests
```

## Design Principles

1. **Event-First**: All changes happen through events
2. **Immutability**: Events and value objects are immutable
3. **No CRUD**: No direct create/read/update/delete operations
4. **Business Language**: Commands and events use domain terminology
5. **Eventual Consistency**: Read models updated asynchronously from events

## License

See the main project LICENSE file. 