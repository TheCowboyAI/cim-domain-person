# CIM Person Domain Documentation

## Overview

The CIM Person Domain provides comprehensive identity and person management capabilities within the Composable Information Machine ecosystem. It follows NATS-first messaging principles with advanced subject algebra, workflow orchestration, and ECS (Entity Component System) architecture.

## Key Features

- **NATS-First Messaging**: Comprehensive subject algebra for events, commands, and queries
- **Workflow Orchestration**: Pre-built workflows for onboarding, employment, skills certification, and privacy compliance
- **ECS Architecture**: Composable entity-component system for flexible person modeling
- **Cross-Domain Integration**: Seamless integration with document, location, and other CIM domains
- **Privacy Compliance**: Built-in GDPR and privacy compliance workflows
- **Identity Management**: Comprehensive identity verification and management
- **Skills & Certification**: Skills tracking, assessment, and certification workflows
- **Employment Lifecycle**: Complete employment management from hiring to termination
- **Network Analysis**: Social and professional network analysis capabilities

## Architecture

### Subject Algebra

The Person domain uses a sophisticated NATS subject algebra:

```
events.person.{aggregate}.{scope}.{operation}.{entity_id}
commands.person.{aggregate}.{scope}.{operation}.{entity_id}
queries.person.{aggregate}.{scope}.{operation}
```

**Aggregates:**
- `person` - Core person identity
- `identity` - Identity verification and management
- `employment` - Employment relationships
- `skills` - Skills and certifications
- `network` - Professional and social connections
- `contact` - Contact information
- `preferences` - User preferences and settings
- `demographics` - Demographic information

**Scopes:**
- `global` - Global operations
- `user.{user_id}` - User-scoped operations
- `org.{org_id}` - Organization-scoped operations
- `team.{team_id}` - Team-scoped operations
- `region.{region_id}` - Region-scoped operations
- `dept.{dept_id}` - Department-scoped operations

### Workflow System

Pre-defined workflows for common person domain processes:

1. **Person Onboarding Workflow**
   - Identity validation
   - Profile creation
   - Preferences setup
   - Completion notifications

2. **Employment Lifecycle Workflow**
   - Background checks
   - HR approval
   - System provisioning
   - Benefits enrollment

3. **Skills Certification Workflow**
   - Skill assessment
   - Peer review
   - Certification issuance
   - Profile updates

4. **Privacy Compliance Workflow**
   - GDPR data export
   - Data deletion requests
   - Compliance notifications
   - Audit trails

### ECS Components

The domain supports various composable components:

- **Contact Components**: Email, phone, address information
- **Skills Components**: Technical skills, certifications, education
- **Preferences Components**: Communication, privacy preferences
- **Employment Components**: Job history, roles, organizations
- **Network Components**: Professional connections, endorsements

## Quick Start

### Basic Usage

```rust
use cim_domain_person::{
    PersonSubject, PersonAggregate, PersonEventType,
    MessageIdentity, PersonActor,
    create_person_onboarding_workflow, WorkflowManager,
    Person, PersonId
};

// Create person subject for events
let subject = PersonSubject::event(
    PersonAggregate::Person,
    PersonEventType::Created,
    "person123"
);

// Create message identity
let identity = MessageIdentity::for_user("user456");

// Create onboarding workflow
let workflow = create_person_onboarding_workflow();

// Initialize workflow manager
let (manager, _events) = WorkflowManager::new(engine, nats_client);
manager.register_workflow(workflow).await?;
```

### Subject Examples

```rust
// Person creation event
let subject = PersonSubject::event(
    PersonAggregate::Person,
    PersonEventType::Created,
    "person123"
);
// Result: "events.person.person.created.person123"

// User-scoped employment event
let subject = PersonSubject::user_event(
    "user456",
    PersonAggregate::Employment,
    PersonEventType::EmploymentAdded,
    "person123"
);
// Result: "events.person.employment.user.user456.employment_added.person123"

// Organization-scoped skills query
let subject = PersonSubject::query(
    PersonAggregate::Skills,
    PersonQueryType::SearchBySkill
).with_scope(PersonScope::Organization("org789".to_string()));
// Result: "queries.person.skills.org.org789.search_by_skill"
```

### Workflow Usage

```rust
use std::collections::HashMap;

// Start person onboarding workflow
let input_data = HashMap::new();
input_data.insert("person_id".to_string(), serde_json::json!("person123"));
input_data.insert("email".to_string(), serde_json::json!("user@example.com"));

let instance_id = manager.start_workflow(
    &workflow.id,
    input_data,
    PersonActor::user("user456")
).await?;

// Monitor workflow progress
let instance = manager.get_instance(instance_id).await?;
println!("Workflow state: {:?}", instance.state);
```

## Integration Examples

### Cross-Domain Integration

```rust
// Person-Document relationship event
let subject = PersonSubject::person_document_event(
    "person123",
    "QmXx...", // Document CID
    "viewed"
);

// Person-Location relationship event  
let subject = PersonSubject::person_location_event(
    "person123",
    "location456",
    "visited"
);
```

### Message Correlation

```rust
// Create correlated message chain
let initial_identity = MessageIdentity::for_hr_system("workday")
    .with_correlation_id("employment-workflow-789");

let child_identity = initial_identity.create_child();
let related_identity = initial_identity.create_related();

// All messages maintain correlation for distributed tracing
assert!(child_identity.is_correlated_with(&initial_identity));
assert!(related_identity.is_correlated_with(&initial_identity));
```

## API Reference

### Core Types

- `PersonSubject` - NATS subject algebra
- `MessageIdentity` - Message correlation and tracing
- `WorkflowDefinition` - Workflow structure and behavior
- `WorkflowManager` - Workflow orchestration engine
- `Person` - Core person aggregate
- `PersonActor` - Actor types for message attribution

### Subject Patterns

- Events: `events.person.{aggregate}.{event_type}.{entity_id}`
- Commands: `commands.person.{aggregate}.{command_type}.{entity_id}`
- Queries: `queries.person.{aggregate}.{query_type}`
- Scoped: `{root}.person.{aggregate}.{scope}.{operation}.{entity_id}`

### Workflow Types

- `PersonOnboarding` - New person registration and setup
- `IdentityVerification` - Identity validation processes
- `EmploymentLifecycle` - Employment management workflows
- `SkillsCertification` - Skills assessment and certification
- `NetworkConnection` - Professional networking workflows
- `PrivacyCompliance` - GDPR and privacy compliance
- `DataMigration` - Data import/export workflows
- `AccountDeactivation` - Account closure processes

## Contributing

The Person domain follows established CIM patterns:

1. **NATS-First**: All communication via NATS messaging
2. **Event Sourcing**: All state changes via events
3. **Workflow Integration**: Business processes via workflows
4. **Cross-Domain**: Integration with other CIM domains
5. **Privacy by Design**: Built-in privacy and compliance features

## Documentation

- [Integration Guide](integration/) - Detailed integration examples
- [Performance Guide](performance/) - Performance optimization
- [Security Guide](security/) - Security best practices
- [Migration Guide](migration/) - Version migration procedures
- [Troubleshooting Guide](troubleshooting/) - Common issues and solutions

## Version

Current version: 0.5.0

Compatible with:
- CIM Domain Core: ^0.1.0
- NATS: ^0.35.0
- Tokio: ^1.32.0