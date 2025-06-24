# CIM Domain Person

ECS-oriented Person domain for the Composable Information Machine (CIM) project.

## Overview

The Person domain has been refactored to follow an Entity Component System (ECS) architecture aligned with Bevy. This domain now focuses exclusively on core person identity, with all other data managed as composable components.

## Architecture

### Core Person Aggregate

The Person aggregate now contains only:
- **PersonId**: Unique identifier
- **CoreIdentity**: Legal name, birth/death dates
- **PersonLifecycle**: Active, Deactivated, Deceased, or Merged states
- **Component Registry**: Tracks which components are attached

### Component System

All other person data is managed through components:
- **Contact Components**: Email, Phone
- **Skill Components**: Skills, Certifications, Education
- **Preference Components**: Communication and Privacy preferences
- **Customer Components**: Segments, Behavioral data
- **Social Components**: Social profiles, Tags

### Cross-Domain Relationships

Relationships with other domains are managed through dedicated modules:
- **Person-Location**: Address associations (using Location domain)
- **Person-Organization**: Employment relationships (using Organization domain)

## Usage

```rust
use cim_domain_person::{
    aggregate::{Person, PersonId, ComponentType},
    value_objects::PersonName,
};

// Create a person with core identity only
let person_id = PersonId::new();
let name = PersonName::new("Alice".to_string(), "Johnson".to_string());
let mut person = Person::new(person_id, name);

// Register components as needed
person.register_component(ComponentType::EmailAddress)?;
person.register_component(ComponentType::Skill)?;
```

## Migration from v0.2

The Person domain has undergone significant refactoring:
- 61% code reduction (890 → 346 lines)
- Removed all complex value objects from core aggregate
- Moved to component-based architecture
- Proper domain boundary enforcement

### Breaking Changes

1. **Removed from Person aggregate**:
   - All contact information (emails, phones, addresses)
   - Employment history
   - Skills and certifications
   - Relationships
   - Customer/business attributes

2. **New Component System**:
   - Components are registered but not stored in the aggregate
   - Actual component data is managed by ECS systems
   - Cross-domain relationships use dedicated service interfaces

3. **Simplified Commands/Events**:
   - Only core identity operations remain
   - Component operations are handled by ECS systems
   - Cross-domain operations use domain services

## Examples

See `examples/person_ecs_simple.rs` for a demonstration of the new architecture.

## Testing

Run tests with:
```bash
cargo test
```

The domain includes comprehensive tests for:
- Person lifecycle management
- Component registration
- Command handling
- Event generation

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[Documentation Index](docs/index.md)** - Start here for complete documentation
- **[Domain Overview](docs/domain_overview.md)** - Architecture and vision
- **[User Stories](docs/user_stories.md)** - Business scenarios and requirements
- **[API Reference](docs/api_reference.md)** - Detailed API documentation
- **[Implementation Guide](docs/implementation_guide.md)** - How to use the domain
- **[Network Analysis Guide](docs/network_analysis_guide.md)** - Relationship analysis patterns
- **[Testing Guide](docs/testing_guide.md)** - Testing strategies and examples

## Version History

- **v0.3.0** - Complete ECS refactoring
- **v0.2.0** - Previous monolithic aggregate
- **v0.1.0** - Initial implementation 