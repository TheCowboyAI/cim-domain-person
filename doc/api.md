# Person API Documentation

## Overview

The Person domain API provides commands, queries, and events for {domain purpose}.

## Commands

### CreatePerson

Creates a new person in the system.

```rust
use cim_domain_person::commands::CreatePerson;

let command = CreatePerson {
    id: PersonId::new(),
    // ... fields
};
```

**Fields:**
- `id`: Unique identifier for the person
- `field1`: Description
- `field2`: Description

**Validation:**
- Field1 must be non-empty
- Field2 must be valid

**Events Emitted:**
- `PersonCreated`

### UpdatePerson

Updates an existing person.

```rust
use cim_domain_person::commands::UpdatePerson;

let command = UpdatePerson {
    id: entity_id,
    // ... fields to update
};
```

**Fields:**
- `id`: Identifier of the person to update
- `field1`: New value (optional)

**Events Emitted:**
- `PersonUpdated`

## Queries

### GetPersonById

Retrieves a person by its identifier.

```rust
use cim_domain_person::queries::GetPersonById;

let query = GetPersonById {
    id: entity_id,
};
```

**Returns:** `Option<PersonView>`

### List{Entities}

Lists all {entities} with optional filtering.

```rust
use cim_domain_person::queries::List{Entities};

let query = List{Entities} {
    filter: Some(Filter {
        // ... filter criteria
    }),
    pagination: Some(Pagination {
        page: 1,
        per_page: 20,
    }),
};
```

**Returns:** `Vec<PersonView>`

## Events

### PersonCreated

Emitted when a new person is created.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreated {
    pub id: PersonId,
    pub timestamp: SystemTime,
    // ... other fields
}
```

### PersonUpdated

Emitted when a person is updated.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonUpdated {
    pub id: PersonId,
    pub changes: Vec<FieldChange>,
    pub timestamp: SystemTime,
}
```

## Value Objects

### PersonId

Unique identifier for {entities}.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonId(Uuid);

impl PersonId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### {ValueObject}

Represents {description}.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {ValueObject} {
    pub field1: String,
    pub field2: i32,
}
```

## Error Handling

The domain uses the following error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum PersonError {
    #[error("person not found: {id}")]
    NotFound { id: PersonId },
    
    #[error("Invalid {field}: {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Operation not allowed: {reason}")]
    Forbidden { reason: String },
}
```

## Usage Examples

### Creating a New Person

```rust
use cim_domain_person::{
    commands::CreatePerson,
    handlers::handle_create_person,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = CreatePerson {
        id: PersonId::new(),
        name: "Example".to_string(),
        // ... other fields
    };
    
    let events = handle_create_person(command).await?;
    
    for event in events {
        println!("Event emitted: {:?}", event);
    }
    
    Ok(())
}
```

### Querying {Entities}

```rust
use cim_domain_person::{
    queries::{List{Entities}, execute_query},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = List{Entities} {
        filter: None,
        pagination: Some(Pagination {
            page: 1,
            per_page: 10,
        }),
    };
    
    let results = execute_query(query).await?;
    
    for item in results {
        println!("{:?}", item);
    }
    
    Ok(())
}
```

## Integration with Other Domains

This domain integrates with:

- **{Other Domain}**: Description of integration
- **{Other Domain}**: Description of integration

## Performance Considerations

- Commands are processed asynchronously
- Queries use indexed projections for fast retrieval
- Events are published to NATS for distribution

## Security Considerations

- All commands require authentication
- Authorization is enforced at the aggregate level
- Sensitive data is encrypted in events 