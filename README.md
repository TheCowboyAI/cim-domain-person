# Person Domain

[![Version](https://img.shields.io/badge/version-0.3.0-blue.svg)](https://github.com/cim/person-domain)

## Overview

The Person domain implements an Entity Component System (ECS) architecture aligned with Bevy. This domain focuses exclusively on core person identity, with all other data managed as composable components.

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