# Pure Event-Driven Architecture

This document describes the pure event-driven architecture implemented in `cim-domain-person`, aligned with the patterns used in `cim-graph`.

## Architecture Overview

The system follows a pure event-driven model where:
- All state changes are represented as events
- Commands are processed asynchronously
- Events are streamed via NATS JetStream
- Aggregates use state machines for complex workflows
- Policies provide reactive business rules

## Core Components

### 1. Event Streaming (NATS JetStream)

All events flow through NATS JetStream for durability and streaming:

```rust
// Configuration
let config = StreamingConfig::new("person-events")
    .with_consumer("processor", ConsumerType::Durable)
    .with_retention(RetentionPolicy::Limits)
    .with_dead_letter_queue("person-events-dlq", 3);

// Streaming subscription
let mut subscriber = StreamingSubscriber::new(jetstream, config).await?;
let mut stream = subscriber.subscribe("processor").await?;

while let Some(msg) = stream.next().await {
    // Process event
    process_event(msg).await?;
}
```

### 2. Async Command Processing

Commands are processed asynchronously with streaming results:

```rust
let processor = AsyncCommandProcessor::new(
    event_store,
    snapshot_store,
    component_store,
);

let result = processor.process(command).await?;

// Stream additional events
if let Some(mut event_stream) = result.event_stream {
    while let Some(event) = event_stream.next().await {
        // Handle streamed events
    }
}
```

### 3. Aggregate State Machines

Complex workflows are modeled as state machines within aggregates:

```rust
// Define states
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OnboardingState {
    ProfileCreated,
    EmailAdded,
    EmailVerified,
    SkillsAdded,
    Completed,
}

// Build state machine
let machine = StateMachine::builder()
    .initial_state(OnboardingState::ProfileCreated)
    .add_transition(
        OnboardingState::ProfileCreated,
        OnboardingCommand::AddEmail,
        OnboardingState::EmailAdded,
        |state, cmd| {
            // Transition logic
            Ok(())
        },
    )
    .build();
```

### 4. Event Versioning

Events can evolve over time with automatic migration:

```rust
// Register migrations
registry.register_migration(
    "PersonCreated",
    "1.0",
    "2.0",
    FunctionMigration::new(|mut data| {
        // Transform V1 to V2
        let created_at = data["created_at"].take();
        data["metadata"] = json!({
            "timestamp": created_at,
            "version": "1.0"
        });
        Ok(data)
    })
);

// Automatic migration
let current = registry.migrate_to_current("PersonCreated", legacy_event)?;
```

### 5. Policy Engine

Business rules are implemented as simple event handlers:

```rust
#[async_trait]
impl Policy for WelcomeEmailPolicy {
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>> {
        match event {
            PersonEventV2::Created { person_id, name, .. } => {
                Ok(vec![
                    PersonCommand::AddComponent(AddComponent {
                        person_id: *person_id,
                        component_type: ComponentType::CustomAttribute,
                        data: json!({
                            "type": "welcome_email",
                            "template": "welcome_new_user",
                            "to": format!("{} {}", name.first_name, name.last_name)
                        }),
                    })
                ])
            }
            _ => Ok(vec![])
        }
    }
}
```

## Event Flow

1. **Command Reception**: Commands arrive via API or message queue
2. **Async Processing**: AsyncCommandProcessor validates and executes
3. **Event Generation**: State changes produce events
4. **Event Persistence**: Events stored in NATS JetStream
5. **Event Streaming**: Events streamed to subscribers
6. **Policy Evaluation**: Policies generate reactive commands
7. **Projection Updates**: Read models updated from events

## Key Patterns

### Streaming Everything

All operations support streaming for scalability:

```rust
pub struct CommandResult {
    pub aggregate_id: PersonId,
    pub version: u64,
    pub events: Vec<PersonEventV2>,
    pub event_stream: Option<Pin<Box<dyn Stream<Item = PersonEventV2> + Send>>>,
}
```

### Retry and Dead Letter Queues

Automatic retry with exponential backoff:

```rust
let handler = RetryHandler::new(
    client,
    jetstream,
    RetryPolicy {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        multiplier: 2.0,
    },
    "person.events.dlq",
);

handler.retry(|| async {
    // Operation that might fail
    process_event(event).await
}).await?;
```

### Event Metadata

All events carry metadata for tracing and debugging:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
}
```

## Benefits

1. **Scalability**: Stream processing handles any volume
2. **Resilience**: Retry policies and dead letter queues
3. **Evolution**: Event versioning enables change
4. **Simplicity**: No complex frameworks, just patterns
5. **Observability**: Event metadata enables tracing

## Migration Path

For systems migrating to this architecture:

1. **Enable Streaming**: Configure NATS JetStream
2. **Migrate Events**: Use provided migration scripts
3. **Update Handlers**: Convert to async processors
4. **Add Policies**: Implement business rules as policies
5. **Test Thoroughly**: Use comprehensive test suite

## Example Usage

```rust
// Initialize system
let event_store = Arc::new(NatsEventStore::new(jetstream.clone()));
let processor = AsyncCommandProcessor::new(event_store, snapshot_store, component_store);
let policy_engine = create_default_policy_engine();

// Process command
let command = PersonCommand::CreatePerson(CreatePerson {
    person_id: PersonId::new(),
    name: PersonName::new("John", None, "Doe")?,
    source: "api",
});

let result = processor.process(command).await?;

// Apply policies
for event in &result.events {
    let policy_commands = policy_engine.evaluate(event).await;
    for cmd in policy_commands {
        processor.process(cmd).await?;
    }
}

// Stream additional events
if let Some(mut stream) = result.event_stream {
    while let Some(event) = stream.next().await {
        let policy_commands = policy_engine.evaluate(&event).await;
        // Process policy commands...
    }
}
```

## Conclusion

This pure event-driven architecture provides a scalable, resilient, and evolvable foundation for the person domain. By embracing streaming, async processing, and simple patterns like state machines and policies, the system can handle complex requirements while remaining maintainable.