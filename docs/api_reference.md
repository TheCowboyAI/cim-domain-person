# Person Domain API Reference

## Core Types

### PersonId
```rust
pub type PersonId = EntityId<PersonMarker>;
```
Unique identifier for a person entity. Uses CIM's EntityId type system.

### Person (Aggregate)
```rust
pub struct Person {
    id: PersonId,
    core_identity: CoreIdentity,
    lifecycle: PersonLifecycle,
    components: HashSet<ComponentType>,
}
```

The minimal Person aggregate containing only core identity information.

### PersonLifecycle
```rust
pub enum PersonLifecycle {
    Active,
    Deactivated { reason: String, since: DateTime<Utc> },
    Deceased { date_of_death: NaiveDate },
    MergedInto { target_id: PersonId, merge_date: DateTime<Utc> },
}
```

Represents the lifecycle state of a person record.

## Commands

### CreatePerson
```rust
pub struct CreatePerson {
    pub person_id: PersonId,
    pub name: PersonName,
}
```
Creates a new person with minimal identity information.

### UpdatePersonName
```rust
pub struct UpdatePersonName {
    pub person_id: PersonId,
    pub new_name: PersonName,
    pub reason: String,
}
```
Updates a person's legal name with audit reason.

### RegisterComponent
```rust
pub struct RegisterComponent {
    pub person_id: PersonId,
    pub component_type: ComponentType,
}
```
Registers that a component type has been attached to this person.

### DeactivatePerson
```rust
pub struct DeactivatePerson {
    pub person_id: PersonId,
    pub reason: String,
}
```
Deactivates a person record, preventing further modifications.

### RecordDeath
```rust
pub struct RecordDeath {
    pub person_id: PersonId,
    pub date_of_death: NaiveDate,
    pub certificate_reference: Option<String>,
}
```
Records that a person has died.

### MergePersons
```rust
pub struct MergePersons {
    pub source_id: PersonId,
    pub target_id: PersonId,
    pub reason: MergeReason,
}
```
Merges one person record into another.

## Events

### PersonCreated
```rust
pub struct PersonCreated {
    pub person_id: PersonId,
    pub name: PersonName,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}
```

### PersonNameUpdated
```rust
pub struct PersonNameUpdated {
    pub person_id: PersonId,
    pub old_name: PersonName,
    pub new_name: PersonName,
    pub reason: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
}
```

### ComponentRegistered
```rust
pub struct ComponentRegistered {
    pub person_id: PersonId,
    pub component_type: ComponentType,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
}
```

### PersonDeactivated
```rust
pub struct PersonDeactivated {
    pub person_id: PersonId,
    pub reason: String,
    pub deactivated_at: DateTime<Utc>,
    pub deactivated_by: String,
}
```

### PersonDeathRecorded
```rust
pub struct PersonDeathRecorded {
    pub person_id: PersonId,
    pub date_of_death: NaiveDate,
    pub certificate_reference: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub recorded_by: String,
}
```

### PersonsMerged
```rust
pub struct PersonsMerged {
    pub source_id: PersonId,
    pub target_id: PersonId,
    pub components_to_migrate: Vec<ComponentType>,
    pub reason: MergeReason,
    pub merged_at: DateTime<Utc>,
    pub merged_by: String,
}
```

## Components

### EmailComponent
```rust
pub struct EmailComponent {
    pub email: EmailAddress,
    pub is_primary: bool,
    pub context: ContactContext,
    pub metadata: ComponentMetadata,
}
```

### PhoneComponent
```rust
pub struct PhoneComponent {
    pub phone: PhoneNumber,
    pub is_primary: bool,
    pub context: ContactContext,
    pub metadata: ComponentMetadata,
}
```

### SkillComponent
```rust
pub struct SkillComponent {
    pub skill_id: SkillId,
    pub name: String,
    pub category: SkillCategory,
    pub proficiency: ProficiencyLevel,
    pub years_experience: Option<f32>,
    pub last_used: Option<NaiveDate>,
    pub metadata: ComponentMetadata,
}
```

### PreferencesComponent
```rust
pub struct PreferencesComponent {
    pub communication: CommunicationPreferences,
    pub privacy: PrivacyPreferences,
    pub content: ContentPreferences,
    pub metadata: ComponentMetadata,
}
```

## Queries

### GetPersonById
```rust
pub struct GetPersonById {
    pub person_id: PersonId,
}
```

### FindPeopleByEmail
```rust
pub struct FindPeopleByEmail {
    pub email: String,
}
```

### FindPeopleBySkill
```rust
pub struct FindPeopleBySkill {
    pub skill_name: String,
    pub min_proficiency: Option<ProficiencyLevel>,
}
```

### FindPeopleByOrganization
```rust
pub struct FindPeopleByOrganization {
    pub organization_id: OrganizationId,
    pub include_inactive: bool,
}
```

### SearchPeople
```rust
pub struct SearchPeople {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub skills: Option<Vec<String>>,
    pub limit: usize,
    pub offset: usize,
}
```

## Cross-Domain Relationships

### PersonLocationRelation
```rust
pub struct PersonLocationRelation {
    pub person_id: PersonId,
    pub location_id: LocationId,
    pub relation_type: LocationRelationType,
    pub is_primary: bool,
    pub valid_from: Option<NaiveDate>,
    pub valid_until: Option<NaiveDate>,
}
```

### PersonOrganizationRelation
```rust
pub struct PersonOrganizationRelation {
    pub person_id: PersonId,
    pub organization_id: OrganizationId,
    pub relation_type: OrganizationRelationType,
    pub role: Option<String>,
    pub department: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}
```

## Error Types

### PersonDomainError
```rust
pub enum PersonDomainError {
    PersonNotFound(PersonId),
    InvalidLifecycleTransition { from: PersonLifecycle, to: PersonLifecycle },
    ComponentAlreadyRegistered(ComponentType),
    CannotModifyInactivePerson,
    CannotModifyDeceasedPerson,
    CannotModifyMergedPerson,
    ValidationError(String),
}
```

## Usage Examples

### Creating a Person
```rust
use cim_domain_person::{
    PersonId, PersonCommand, PersonName,
    aggregate::Person,
};

// Create command
let command = PersonCommand::CreatePerson(CreatePerson {
    person_id: PersonId::new(),
    name: PersonName::new("Alice", "Johnson"),
});

// Handle command
let mut person = Person::new(person_id, name);
let events = person.handle_command(command)?;
```

### Adding Components (ECS Pattern)
```rust
// Register component on aggregate
person.register_component(ComponentType::EmailAddress)?;

// In Bevy ECS system
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

### Querying People
```rust
use cim_domain_person::queries::{PersonQuery, FindPeopleBySkill};

let query = PersonQuery::FindPeopleBySkill {
    skill_name: "Rust".to_string(),
    min_proficiency: Some(ProficiencyLevel::Expert),
};

let results = query_handler.handle_query(query).await?;
```

### Cross-Domain Integration
```rust
// Link person to organization
let employment = PersonOrganizationRelation {
    person_id,
    organization_id,
    relation_type: OrganizationRelationType::Employee,
    role: Some("Software Engineer".to_string()),
    department: Some("Engineering".to_string()),
    start_date: today,
    end_date: None,
};

// This would trigger cross-domain events
cross_domain_handler.establish_employment(employment).await?;
``` 