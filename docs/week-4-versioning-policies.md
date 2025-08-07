# Week 4: Event Versioning and Policies - Completed

## Summary

Week 4 has been completed, adding event versioning for schema evolution and a simple policy engine for event-driven business rules. The system can now evolve its event schemas while maintaining backward compatibility.

## What Was Added

### 1. Event Versioning Framework (`events/versioning.rs`)
- **VersionedEvent trait**: Interface for versioned events
- **EventVersionRegistry**: Manages versions and migrations
- **EventMigration trait**: Define how to migrate between versions
- **FunctionMigration**: Simple function-based migrations
- **VersionedEventEnvelope**: Wrapper with version metadata

### 2. Versioned Events (`events/versioned_events.rs`)
- **V1 Events**: Original format (legacy)
- **V2 Events**: Current format with metadata
- **V3 Events**: Future format example
- **Migration functions**: V1→V2, V2→V3 migrations
- **Registry setup**: Pre-configured migrations

### 3. Policy Engine (`policies/mod.rs`)
- **Policy trait**: Simple event → commands interface
- **PolicyEngine**: Evaluates events against policies
- **No complex frameworks**: Policies are just event handlers
- **Composable**: Multiple policies can react to same event

### 4. Example Policies
- **AutoArchivePolicy**: Archive inactive persons
- **WelcomeEmailPolicy**: Send welcome messages
- **SkillRecommendationPolicy**: Suggest skills based on activity
- **DataQualityPolicy**: Validate and standardize data

## Key Concepts

### Event Versioning

Events can evolve over time while maintaining compatibility:

```rust
// V1 Event (legacy)
{
  "version": "1.0",
  "person_id": "123",
  "name": {...},
  "created_at": "2023-01-01T00:00:00Z"
}

// V2 Event (current)
{
  "version": "2.0",
  "person_id": "123",
  "name": {...},
  "metadata": {
    "version": "1.0",
    "correlation_id": "...",
    "timestamp": "2023-01-01T00:00:00Z"
  }
}
```

### Migrations

Simple function-based migrations between versions:

```rust
registry.register_migration(
    "PersonCreated",
    "1.0",
    "2.0",
    FunctionMigration::new(|mut data| {
        // Transform v1 to v2
        let created_at = data["created_at"].take();
        data["metadata"] = json!({
            "timestamp": created_at,
            // ... other metadata
        });
        data["version"] = json!("2.0");
        Ok(data)
    })
);
```

### Policies as Event Handlers

Policies are simple functions that generate commands from events:

```rust
#[async_trait]
impl Policy for WelcomeEmailPolicy {
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>> {
        match event {
            PersonEventV2::Created { person_id, name, .. } => {
                // Generate command to send welcome email
                Ok(vec![/* command */])
            }
            _ => Ok(vec![])
        }
    }
}
```

## Usage Examples

### Migrating Legacy Events

```rust
let registry = create_event_registry();

// Receive legacy v1 event
let v1_event = json!({
    "version": "1.0",
    "person_id": "123",
    // ... v1 fields
});

// Automatically migrate to current version
let current = registry.migrate_to_current("PersonCreated", v1_event)?;
```

### Using the Policy Engine

```rust
// Create policy engine with default policies
let engine = create_default_policy_engine();

// Process an event
let event = PersonEventV2::Created { /* ... */ };
let commands = engine.evaluate(&event).await;

// Execute generated commands
for command in commands {
    command_processor.process(command).await?;
}
```

### Creating Custom Policies

```rust
struct CustomPolicy;

#[async_trait]
impl Policy for CustomPolicy {
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>> {
        // Your business logic here
    }
    
    fn name(&self) -> &str {
        "CustomPolicy"
    }
}

// Register with engine
engine.register(Arc::new(CustomPolicy));
```

## Benefits Achieved

1. **Schema Evolution**: Events can change over time
2. **Backward Compatibility**: Old events still work
3. **Simple Policies**: No complex rule engines
4. **Event-Driven Rules**: Policies react to domain events
5. **Extensible**: Easy to add new policies

## Testing

Run the example to see versioning and policies in action:

```bash
cargo run --example versioning_and_policies
```

## Migration Notes

- Existing events continue to work
- New events use V2 format with metadata
- Policies are optional - can be added incrementally
- No breaking changes to existing code

## Next Steps (Week 5)

- Comprehensive testing suite
- Performance benchmarking
- Migration scripts for existing data
- Documentation updates
- Final integration testing

## Key Takeaways

1. **Versioning is Simple**: Just transformation functions
2. **Policies are Event Handlers**: No complex frameworks
3. **Evolution, not Revolution**: Gradual migration path
4. **Composable Rules**: Multiple policies can collaborate
5. **Type Safety**: Strongly typed events and migrations