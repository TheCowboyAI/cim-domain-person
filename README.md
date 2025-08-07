# CIM Domain Person

Person domain for the Composable Information Machine (CIM) - a pure event-driven architecture for managing person entities.

## Overview

`cim-domain-person` implements the Person domain using Entity Component System (ECS) architecture with pure event-driven patterns aligned with `cim-graph`. The system features:

- **Pure Event Streaming**: All operations are async and streaming-capable
- **State Machines**: Complex workflows modeled as aggregate state machines
- **Event Versioning**: Schema evolution with automatic migration
- **Policy Engine**: Reactive business rules as simple event handlers
- **NATS JetStream**: Distributed event persistence and streaming

## Architecture

### Core Concepts

1. **Person as Entity**: Minimal core identity (ID + name)
2. **Components**: Composable capabilities (email, skills, preferences)
3. **Events**: All state changes are events
4. **Commands**: Async processing with streaming results
5. **Policies**: Business rules that react to events

### Event Flow

```
Command → AsyncProcessor → Events → NATS JetStream
                              ↓
                         PolicyEngine → Reactive Commands
                              ↓
                         Projections → Read Models
```

## Quick Start

```rust
use cim_domain_person::{
    aggregate::PersonId,
    commands::{PersonCommand, CreatePerson},
    handlers::AsyncCommandProcessor,
    policies::create_default_policy_engine,
    value_objects::PersonName,
};

// Setup
let processor = AsyncCommandProcessor::new(
    event_store,
    snapshot_store,
    component_store,
);
let policy_engine = create_default_policy_engine();

// Create person
let command = PersonCommand::CreatePerson(CreatePerson {
    person_id: PersonId::new(),
    name: PersonName::new("Alice", None, "Smith")?,
    source: "api",
});

// Process command
let result = processor.process(command).await?;

// Apply policies
for event in &result.events {
    let commands = policy_engine.evaluate(event).await;
    for cmd in commands {
        processor.process(cmd).await?;
    }
}
```

## Features

### Streaming Commands

All commands support streaming for handling large operations:

```rust
let result = processor.process(command).await?;

if let Some(mut stream) = result.event_stream {
    while let Some(event) = stream.next().await {
        // Process streamed events
    }
}
```

### State Machine Workflows

Complex workflows use state machines:

```rust
let mut onboarding = OnboardingAggregate::new(person_id, name);

onboarding.handle(OnboardingCommand::AddEmail {
    email: "user@example.com".to_string(),
})?;

onboarding.handle(OnboardingCommand::VerifyEmail {
    token: "verification-token".to_string(),
})?;
```

### Event Versioning

Events can evolve with automatic migration:

```rust
// Legacy V1 event automatically migrated to V2
let registry = create_event_registry();
let current = registry.migrate_to_current("PersonCreated", v1_event)?;
```

### Policy Engine

Business rules as event handlers:

```rust
struct CustomPolicy;

#[async_trait]
impl Policy for CustomPolicy {
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>> {
        // Generate reactive commands
    }
}
```

## Migration

For existing systems, migration tools are provided:

```bash
# Check current state
cargo run --bin verify_migration --features migration

# Migrate events
cargo run --bin migrate_events --features migration

# Verify success
cargo run --bin verify_migration --features migration
```

## Examples

See the `examples/` directory for:
- `basic_usage.rs` - Simple CRUD operations
- `component_system.rs` - Working with components
- `versioning_and_policies.rs` - Event versioning and policies
- `full_integration.rs` - Complete integration example

## Testing

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Run specific integration test
cargo test --test streaming_tests
```

## Documentation

- [Pure Event-Driven Architecture](docs/pure-event-driven-architecture.md)
- [Migration Summary](docs/migration-summary.md)
- [Week-by-week implementation](docs/)

## Dependencies

- `cim-domain` - Core domain abstractions
- `async-nats` - NATS client for event streaming
- `bevy_ecs` - Entity Component System
- `tokio` - Async runtime

## License

MIT