# API Reference

## Commands

### Person Management

#### CreatePerson
Creates a new person entity.

```rust
use cim_domain_person::commands::{PersonCommand, CreatePerson};
use cim_domain_person::value_objects::PersonName;

let command = PersonCommand::CreatePerson(CreatePerson {
    person_id: PersonId::new(),
    name: PersonName::new("Alice".to_string(), "Smith".to_string()),
    source: "api".to_string(),
});
```

**Fields:**
- `person_id`: Unique identifier for the person
- `name`: PersonName value object
- `source`: Origin system/service

**Events Emitted:**
- `PersonEventV2::Created`

#### UpdatePerson
Updates person information.

```rust
let command = PersonCommand::UpdatePerson(UpdatePerson {
    person_id,
    name: Some(new_name),
    metadata: EventMetadata::new(),
});
```

**Fields:**
- `person_id`: ID of person to update
- `name`: Optional new name
- `metadata`: Event metadata

**Events Emitted:**
- `PersonEventV2::Updated`

### Component Commands

#### AddEmailComponent
Adds an email component to a person.

```rust
use cim_domain_person::commands::AddEmailComponent;

let command = PersonCommand::AddComponent(AddComponent::Email(AddEmailComponent {
    person_id,
    email: "alice@example.com".to_string(),
    is_primary: true,
    verified: false,
}));
```

#### AddPhoneComponent
Adds a phone component to a person.

```rust
let command = PersonCommand::AddComponent(AddComponent::Phone(AddPhoneComponent {
    person_id,
    number: "+1-555-0123".to_string(),
    country_code: "US".to_string(),
    is_primary: true,
    sms_capable: true,
}));
```

#### AddSkillComponent
Adds a skill component to a person.

```rust
let command = PersonCommand::AddComponent(AddComponent::Skill(AddSkillComponent {
    person_id,
    name: "Rust Programming".to_string(),
    category: SkillCategory::Technical,
    proficiency: ProficiencyLevel::Expert,
    years_experience: Some(5),
}));
```

### State Transitions

#### ActivatePerson
Transitions person to active state.

```rust
let command = PersonCommand::Activate(ActivatePerson {
    person_id,
    reason: "Email verified".to_string(),
});
```

#### SuspendPerson
Suspends a person account.

```rust
let command = PersonCommand::Suspend(SuspendPerson {
    person_id,
    reason: "Policy violation".to_string(),
    until: Some(future_date),
});
```

## Queries

### GetPersonById
Retrieves complete person information.

```rust
use cim_domain_person::queries::{PersonQuery, GetPersonById};

let query = PersonQuery::GetById(GetPersonById {
    person_id,
    include_components: true,
});

let result = query_processor.process(query).await?;
```

**Returns:** `Option<PersonView>`

### SearchPersons
Search persons with filters.

```rust
let query = PersonQuery::Search(SearchPersons {
    name_contains: Some("Alice".to_string()),
    status: Some(PersonLifecycle::Active),
    has_email: Some(true),
    limit: 20,
    offset: 0,
});
```

**Returns:** `Vec<PersonSummary>`

### GetPersonTimeline
Get activity timeline for a person.

```rust
let query = PersonQuery::GetTimeline(GetPersonTimeline {
    person_id,
    from_date: Some(start_date),
    to_date: Some(end_date),
    event_types: vec!["Created", "Updated"],
});
```

**Returns:** `Vec<TimelineEntry>`

### GetPersonSkills
Get skill matrix for a person.

```rust
let query = PersonQuery::GetSkills(GetPersonSkills {
    person_id,
    category_filter: Some(SkillCategory::Technical),
});
```

**Returns:** `SkillMatrix`

## Events

### PersonEventV2
Core event enumeration for all person domain events.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonEventV2 {
    Created {
        person_id: PersonId,
        name: PersonName,
        source: String,
        metadata: EventMetadata,
    },
    Updated {
        person_id: PersonId,
        changes: Vec<FieldChange>,
        metadata: EventMetadata,
    },
    ComponentAdded {
        person_id: PersonId,
        component_type: ComponentType,
        component_data: serde_json::Value,
        metadata: EventMetadata,
    },
    ComponentUpdated {
        person_id: PersonId,
        component_id: ComponentId,
        changes: Vec<FieldChange>,
        metadata: EventMetadata,
    },
    ComponentRemoved {
        person_id: PersonId,
        component_id: ComponentId,
        metadata: EventMetadata,
    },
    StateChanged {
        person_id: PersonId,
        from_state: PersonLifecycle,
        to_state: PersonLifecycle,
        reason: String,
        metadata: EventMetadata,
    },
}
```

### EventMetadata
Metadata attached to all events.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub actor: Option<String>,
    pub source_system: String,
    pub version: u32,
}
```

## Value Objects

### PersonId
Unique identifier for persons.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonId(Uuid);

impl PersonId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_string(s: &str) -> Result<Self, ParseError> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}
```

### PersonName
Represents a person's name.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonName {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub display_name: String,
}

impl PersonName {
    pub fn new(first: String, last: String) -> Self {
        let display = format!("{} {}", first, last);
        Self {
            first_name: first,
            last_name: last,
            middle_name: None,
            display_name: display,
        }
    }
}
```

### PersonLifecycle
State enumeration for person lifecycle.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PersonLifecycle {
    Created,
    Active,
    Suspended,
    Archived,
    Deleted,
}
```

### ComponentType
Types of components that can be attached to persons.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentType {
    Email,
    Phone,
    Address,
    Skill,
    Preference,
    Social,
    Professional,
}
```

## Error Handling

### DomainError
Core error type for the domain.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Person not found: {id}")]
    PersonNotFound { id: PersonId },
    
    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition {
        from: PersonLifecycle,
        to: PersonLifecycle,
    },
    
    #[error("Component not found: {component_id}")]
    ComponentNotFound { component_id: ComponentId },
    
    #[error("Validation failed: {field} - {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
}
```

## Basic Usage

### Creating and Managing a Person

```rust
use cim_domain_person::{
    aggregate::PersonId,
    commands::{PersonCommand, CreatePerson},
    handlers::AsyncCommandProcessor,
    value_objects::PersonName,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize processor
    let processor = AsyncCommandProcessor::new(
        event_store,
        snapshot_store,
        component_store,
    );
    
    // Create person
    let person_id = PersonId::new();
    let command = PersonCommand::CreatePerson(CreatePerson {
        person_id: person_id.clone(),
        name: PersonName::new("Alice".to_string(), "Smith".to_string()),
        source: "registration".to_string(),
    });
    
    let result = processor.process(command).await?;
    println!("Person created with {} events", result.events.len());
    
    // Add email component
    let add_email = PersonCommand::AddComponent(AddComponent::Email(AddEmailComponent {
        person_id: person_id.clone(),
        email: "alice@example.com".to_string(),
        is_primary: true,
        verified: false,
    }));
    
    processor.process(add_email).await?;
    
    // Query person
    let query = PersonQuery::GetById(GetPersonById {
        person_id,
        include_components: true,
    });
    
    let person_view = query_processor.process(query).await?;
    println!("Person: {:?}", person_view);
    
    Ok(())
}
```

### Streaming Large Operations

```rust
use futures::StreamExt;

// Process bulk import with streaming
let import_command = PersonCommand::BulkImport(BulkImport {
    source_file: "persons.csv".to_string(),
});

let result = processor.process(import_command).await?;

if let Some(mut stream) = result.event_stream {
    while let Some(event) = stream.next().await {
        match event {
            Ok(PersonEventV2::Created { person_id, .. }) => {
                println!("Imported person: {}", person_id);
            }
            Err(e) => eprintln!("Import error: {}", e),
            _ => {}
        }
    }
}
```

### Working with Policies

```rust
use cim_domain_person::policies::{Policy, PolicyEngine};

// Create policy engine with default policies
let mut policy_engine = PolicyEngine::new();
policy_engine.add_policy(Box::new(WelcomeEmailPolicy));
policy_engine.add_policy(Box::new(DataQualityPolicy));

// Process events through policies
for event in result.events {
    let commands = policy_engine.evaluate(&event).await?;
    for command in commands {
        processor.process(command).await?;
    }
}
```