# Person Domain Implementation Guide

## Getting Started

This guide helps you implement and use the Person domain in your CIM-based applications.

## Basic Usage

### 1. Adding the Dependency

```toml
[dependencies]
cim-domain-person = { path = "../cim-domain-person" }
cim-domain = { path = "../cim-domain" }
```

### 2. Creating Your First Person

```rust
use cim_domain_person::{
    aggregate::{Person, PersonId},
    value_objects::PersonName,
};

// Create a new person
let person_id = PersonId::new();
let name = PersonName::new("Alice", "Johnson");
let person = Person::new(person_id, name);

println!("Created person: {}", person.id());
```

### 3. Using Components

The Person domain uses an ECS (Entity Component System) pattern. Components are registered on the aggregate but stored separately in your ECS system.

```rust
// Register that a person has an email component
person.register_component(ComponentType::EmailAddress)?;

// In your Bevy ECS system
commands.spawn((
    PersonEntity { person_id },
    EmailComponent {
        email: EmailAddress::new("alice@example.com"),
        is_primary: true,
        context: ContactContext::Work,
        metadata: ComponentMetadata::new(),
    },
));
```

## Common Patterns

### Pattern 1: Employee Management

```rust
use cim_domain_person::services::PersonCompositionService;

// Create an employee with standard components
let employee = PersonCompositionService::create_employee_with_components(
    NameComponent::simple("John", "Smith"),
    EmploymentComponent {
        organization_id: org_id,
        employee_id: "EMP001".to_string(),
        title: "Software Engineer".to_string(),
        department: Some("Engineering".to_string()),
        manager_id: Some(manager_person_id),
        status: "active".to_string(),
        start_date: today(),
        end_date: None,
    },
    ContactComponent {
        emails: vec![EmailAddress {
            email: "john.smith@company.com".to_string(),
            email_type: "work".to_string(),
            is_primary: true,
            is_verified: true,
        }],
        phones: vec![],
        addresses: vec![],
    },
)?;
```

### Pattern 2: Customer Profiles

```rust
// Create a customer with preferences
let customer = PersonCompositionService::create_customer_with_components(
    NameComponent::simple("Jane", "Doe"),
    ContactComponent { /* ... */ },
    PreferencesComponent {
        communication: CommunicationPreferences {
            preferred_channel: ContactChannel::Email,
            preferred_language: "en-US".to_string(),
            frequency_preference: FrequencyPreference::Weekly,
            // ...
        },
        // ...
    },
)?;
```

### Pattern 3: Network Analysis

```rust
use cim_domain_person::network::{NetworkAnalyzer, RelationshipType};

// Find connections between people
let path = network_analyzer
    .find_shortest_path(person_a, person_b, max_hops: 3)
    .await?;

// Calculate influence scores
let influence = network_analyzer
    .calculate_influence_score(person_id)
    .await?;

// Find communities
let communities = network_analyzer
    .detect_communities(seed_people, min_size: 5)
    .await?;
```

## Integration with Other Domains

### Organization Domain Integration

```rust
// Establish employment relationship
let employment = PersonOrganizationRelation {
    person_id,
    organization_id,
    relation_type: OrganizationRelationType::Employee,
    role: Some("Senior Developer".to_string()),
    department: Some("R&D".to_string()),
    start_date: today(),
    end_date: None,
};

cross_domain_service
    .establish_employment(employment)
    .await?;
```

### Location Domain Integration

```rust
// Associate person with location
let location_relation = PersonLocationRelation {
    person_id,
    location_id,
    relation_type: LocationRelationType::WorkLocation,
    is_primary: true,
    valid_from: Some(today()),
    valid_until: None,
};

cross_domain_service
    .associate_location(location_relation)
    .await?;
```

## Query Examples

### Finding People

```rust
use cim_domain_person::queries::PersonQuery;

// Find by email
let results = query_handler
    .handle_query(PersonQuery::FindPeopleByEmail {
        email: "alice@example.com".to_string(),
    })
    .await?;

// Find by skill
let experts = query_handler
    .handle_query(PersonQuery::FindPeopleBySkill {
        skill_name: "Rust".to_string(),
        min_proficiency: Some(ProficiencyLevel::Expert),
    })
    .await?;

// Complex search
let results = query_handler
    .handle_query(PersonQuery::SearchPeople {
        name: Some("John".to_string()),
        skills: Some(vec!["Rust".to_string(), "Python".to_string()]),
        organization: Some(org_id),
        limit: 50,
        offset: 0,
    })
    .await?;
```

## Event Handling

### Subscribing to Person Events

```rust
// Subscribe to person events via NATS
let subscription = nats_client
    .subscribe("person.events.>")
    .await?;

while let Some(msg) = subscription.next().await {
    match parse_event(&msg)? {
        PersonEvent::PersonCreated { person_id, name, .. } => {
            println!("New person created: {} - {}", person_id, name);
        }
        PersonEvent::ComponentRegistered { person_id, component_type, .. } => {
            println!("Component {} added to person {}", component_type, person_id);
        }
        // Handle other events...
    }
}
```

## Best Practices

### 1. Component Design

- Keep components focused and single-purpose
- Use value objects for component fields
- Include metadata for audit trails
- Version components for evolution

### 2. Performance Optimization

- Index frequently queried fields
- Use batch operations for bulk updates
- Cache network analysis results
- Implement pagination for large result sets

### 3. Privacy and Security

- Check privacy preferences before exposing data
- Implement field-level access control
- Audit all data access
- Handle PII with care

### 4. Testing

- Test aggregate behavior separately from components
- Use builders for test data
- Mock cross-domain interactions
- Verify event generation

## Troubleshooting

### Common Issues

**Issue**: "Cannot modify inactive person"
```rust
// Check lifecycle before operations
if person.is_active() {
    person.update_name(new_name, reason)?;
} else {
    return Err(DomainError::PersonInactive);
}
```

**Issue**: "Component already registered"
```rust
// Check before registering
if !person.has_component(&ComponentType::EmailAddress) {
    person.register_component(ComponentType::EmailAddress)?;
}
```

**Issue**: "Network query timeout"
```rust
// Use appropriate limits
let results = network_analyzer
    .find_connections(person_id)
    .with_max_depth(3)
    .with_timeout(Duration::from_secs(5))
    .execute()
    .await?;
```

## Advanced Topics

### Custom Components

```rust
// Define your own component
#[derive(Component, Serialize, Deserialize)]
pub struct CustomComponent {
    pub custom_field: String,
    pub metadata: ComponentMetadata,
}

// Register with the system
impl From<CustomComponent> for ComponentType {
    fn from(_: CustomComponent) -> Self {
        ComponentType::Custom("MyCustomComponent".to_string())
    }
}
```

### Event Projections

```rust
// Build custom projections
pub struct TeamProjection {
    teams: HashMap<TeamId, Vec<PersonId>>,
}

impl EventHandler for TeamProjection {
    fn handle_event(&mut self, event: PersonEvent) {
        match event {
            PersonEvent::ComponentRegistered { 
                person_id, 
                component_type: ComponentType::Team(team_id),
                ..
            } => {
                self.teams
                    .entry(team_id)
                    .or_default()
                    .push(person_id);
            }
            _ => {}
        }
    }
}
``` 