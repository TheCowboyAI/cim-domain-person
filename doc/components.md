# Component System Guide

## Overview

The Person domain uses an Entity Component System (ECS) architecture where persons are entities with attached components providing specific capabilities.

## Component Architecture

### Core Concepts

- **Entity**: Person with minimal core identity
- **Component**: Data container for specific capability
- **System**: Logic that operates on components

### Component Lifecycle

```
Add Component → Validate → Store → Emit Event → Update Projections
```

## Available Components

### Contact Components

#### EmailComponent
Manages email addresses for a person.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailComponent {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub email: Email,
    pub is_primary: bool,
    pub context: ContactContext,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub address: String,
    pub verified: bool,
    pub verification_token: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
}
```

**Usage:**
```rust
let email_component = EmailComponent::new(
    person_id,
    "alice@example.com",
    true, // is_primary
    ContactContext::Work,
);
```

#### PhoneComponent
Manages phone numbers.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneComponent {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub phone: Phone,
    pub is_primary: bool,
    pub context: ContactContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phone {
    pub number: String,
    pub country_code: String,
    pub extension: Option<String>,
    pub sms_capable: bool,
    pub verified: bool,
}
```

#### AddressComponent
Manages physical addresses.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressComponent {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub address: Address,
    pub is_primary: bool,
    pub context: ContactContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street_1: String,
    pub street_2: Option<String>,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country: String,
    pub coordinates: Option<Coordinates>,
}
```

### Professional Components

#### SkillComponent
Tracks skills and competencies.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillComponent {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub name: String,
    pub category: SkillCategory,
    pub proficiency: ProficiencyLevel,
    pub years_experience: Option<u8>,
    pub last_used: Option<DateTime<Utc>>,
    pub certifications: Vec<Certification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillCategory {
    Technical,
    Language,
    Management,
    Creative,
    Interpersonal,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}
```

#### ExperienceComponent
Work experience and employment history.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceComponent {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub organization: String,
    pub role: String,
    pub description: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_current: bool,
    pub skills_used: Vec<String>,
}
```

### Social Components

#### RelationshipComponent
Manages relationships between persons.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipComponent {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub related_person_id: PersonId,
    pub relationship_type: RelationshipType,
    pub strength: RelationshipStrength,
    pub context: RelationshipContext,
    pub established_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Family,
    Friend,
    Colleague,
    Manager,
    DirectReport,
    Mentor,
    Mentee,
    Other(String),
}
```

#### SocialMediaComponent
Social media profiles and handles.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMediaComponent {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub platform: SocialPlatform,
    pub handle: String,
    pub url: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialPlatform {
    LinkedIn,
    Twitter,
    GitHub,
    Facebook,
    Instagram,
    Other(String),
}
```

### Preference Components

#### NotificationPreferences
Notification settings and preferences.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub frequency: NotificationFrequency,
    pub quiet_hours: Option<QuietHours>,
    pub categories: HashMap<String, bool>,
}
```

#### PrivacySettings
Privacy and data sharing preferences.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub profile_visibility: Visibility,
    pub contact_sharing: ContactSharing,
    pub data_retention: DataRetentionPreference,
    pub third_party_sharing: bool,
}
```

## Component Operations

### Adding Components

```rust
use cim_domain_person::components::ComponentStore;

// Add email component
let email_component = EmailComponent::new(
    person_id,
    "alice@example.com",
    true,
    ContactContext::Personal,
);

component_store.add_component(email_component).await?;

// Emit event
let event = PersonEventV2::ComponentAdded {
    person_id,
    component_type: ComponentType::Email,
    component_data: serde_json::to_value(&email_component)?,
    metadata: EventMetadata::new(),
};
```

### Updating Components

```rust
// Update component
let updated = component_store
    .update_component(component_id, |mut component| {
        if let Component::Email(ref mut email) = component {
            email.email.verified = true;
            email.email.verified_at = Some(Utc::now());
        }
        component
    })
    .await?;

// Emit event
let event = PersonEventV2::ComponentUpdated {
    person_id,
    component_id,
    changes: vec![
        FieldChange::new("verified", "false", "true"),
    ],
    metadata: EventMetadata::new(),
};
```

### Querying Components

```rust
// Get all components for a person
let components = component_store
    .get_components_for_person(person_id)
    .await?;

// Get specific component type
let emails: Vec<EmailComponent> = component_store
    .get_components_by_type(person_id, ComponentType::Email)
    .await?;

// Find primary email
let primary_email = emails
    .iter()
    .find(|e| e.is_primary)
    .ok_or(DomainError::NoPrimaryEmail)?;
```

### Removing Components

```rust
// Remove component
component_store.remove_component(component_id).await?;

// Emit event
let event = PersonEventV2::ComponentRemoved {
    person_id,
    component_id,
    metadata: EventMetadata::new(),
};
```

## Component Validation

### Validation Rules

Each component type has specific validation rules:

```rust
impl EmailComponent {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Email format validation
        if !self.email.address.contains('@') {
            return Err(ValidationError::InvalidEmail);
        }
        
        // Domain validation
        if self.context == ContactContext::Work {
            validate_corporate_domain(&self.email.address)?;
        }
        
        Ok(())
    }
}
```

### Cross-Component Validation

```rust
pub async fn validate_components(
    person_id: PersonId,
    store: &ComponentStore,
) -> Result<(), ValidationError> {
    let components = store.get_components_for_person(person_id).await?;
    
    // Ensure at least one primary email
    let has_primary_email = components
        .iter()
        .any(|c| matches!(c, Component::Email(e) if e.is_primary));
        
    if !has_primary_email {
        return Err(ValidationError::NoPrimaryEmail);
    }
    
    // Check for duplicate primary components
    validate_single_primary(&components)?;
    
    Ok(())
}
```

## Component Composition

### Building Person Views

```rust
pub struct PersonView {
    pub id: PersonId,
    pub name: PersonName,
    pub lifecycle: PersonLifecycle,
    pub emails: Vec<EmailComponent>,
    pub phones: Vec<PhoneComponent>,
    pub addresses: Vec<AddressComponent>,
    pub skills: Vec<SkillComponent>,
    pub relationships: Vec<RelationshipComponent>,
}

impl PersonView {
    pub async fn build(
        person: Person,
        store: &ComponentStore,
    ) -> Result<Self, DomainError> {
        let components = store
            .get_components_for_person(person.id)
            .await?;
            
        Ok(Self {
            id: person.id,
            name: person.core_identity.legal_name,
            lifecycle: person.lifecycle,
            emails: extract_components(components, ComponentType::Email),
            phones: extract_components(components, ComponentType::Phone),
            addresses: extract_components(components, ComponentType::Address),
            skills: extract_components(components, ComponentType::Skill),
            relationships: extract_components(components, ComponentType::Relationship),
        })
    }
}
```

### Component Aggregation

```rust
pub struct SkillMatrix {
    pub person_id: PersonId,
    pub technical_skills: Vec<SkillComponent>,
    pub soft_skills: Vec<SkillComponent>,
    pub languages: Vec<SkillComponent>,
    pub average_proficiency: f32,
    pub total_years_experience: u32,
}

impl SkillMatrix {
    pub fn from_components(
        person_id: PersonId,
        skills: Vec<SkillComponent>,
    ) -> Self {
        let technical = skills
            .iter()
            .filter(|s| s.category == SkillCategory::Technical)
            .cloned()
            .collect();
            
        // ... aggregate other categories
        
        Self {
            person_id,
            technical_skills: technical,
            // ...
        }
    }
}
```

## Best Practices

### Component Design
1. Keep components focused on single responsibility
2. Use value objects for complex fields
3. Include audit fields (created_at, updated_at)
4. Make components serializable

### Performance
1. Lazy load components when possible
2. Use projections for complex queries
3. Cache frequently accessed components
4. Batch component operations

### Data Integrity
1. Validate on creation and update
2. Enforce business rules through events
3. Use transactions for multi-component operations
4. Maintain referential integrity

### Testing
1. Unit test component validation
2. Integration test component storage
3. Test component composition
4. Verify event generation