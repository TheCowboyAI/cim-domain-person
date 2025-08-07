# Week 1: NATS Streaming Enhancement - Completed

## Summary

Week 1 of the migration has been completed, enhancing the NATS integration with full streaming support, event metadata, retry policies, and dead letter queue handling.

## What Was Added

### 1. Enhanced Streaming Configuration (`infrastructure/streaming.rs`)
- **StreamingConfig**: Comprehensive configuration for NATS JetStream
- **Consumer configurations**: Pre-defined consumers for projections and cross-domain sync
- **Dead letter queue support**: Automatic DLQ configuration
- **Stream settings**: Retention policies, deduplication, and limits

### 2. Event Metadata (`infrastructure/streaming.rs`)
- **EventMetadata struct**: Includes version, correlation ID, causation ID, timestamp
- **Correlation tracking**: Link related events across the system
- **Causation tracking**: Link events to their originating commands
- **Context map**: Extensible metadata for additional information

### 3. Retry Policies (`infrastructure/retry.rs`)
- **RetryHandler**: Exponential backoff with jitter
- **Circuit breaker**: Prevent cascading failures
- **Dead letter queue**: Failed events are sent to DLQ after max retries
- **Configurable policies**: Adjust retry behavior per use case

### 4. Enhanced Events (`events/enhanced.rs`)
- **PersonEventV2**: All events now include metadata
- **StreamingEventEnvelope**: Wrapper for events with sequence numbers
- **Subject generation**: Automatic NATS subject routing
- **Event versioning**: Built-in version field for future migrations

### 5. Streaming Subscriptions (`infrastructure/subscriptions.rs`)
- **SubscriptionManager**: Manages multiple event handlers
- **StreamingEventHandler trait**: Interface for event processors
- **Automatic acknowledgment**: Success/failure handling
- **DLQ integration**: Failed messages automatically sent to DLQ

## How to Use

### Setting Up Streaming

```rust
use cim_domain_person::infrastructure::{StreamingConfig, StreamingClient};

// Create configuration
let config = StreamingConfig::default();

// Connect with streaming
let client = StreamingClient::new("nats://localhost:4222", config).await?;
```

### Publishing Events with Metadata

```rust
use cim_domain_person::events::{PersonEventV2, EventMetadata};

let event = PersonEventV2::Created {
    person_id: PersonId::new(),
    name: PersonName::new("John", None, "Doe")?,
    source: "api".to_string(),
    metadata: EventMetadata::new(),
};

// Publish to NATS
let subject = event.subject();
let payload = serde_json::to_vec(&event)?;
client.jetstream().publish(subject, payload.into()).await?;
```

### Creating Event Handlers

```rust
use cim_domain_person::infrastructure::StreamingEventHandler;

struct MyProjectionHandler;

#[async_trait]
impl StreamingEventHandler for MyProjectionHandler {
    async fn handle_event(&self, envelope: StreamingEventEnvelope) -> DomainResult<()> {
        match &envelope.event {
            PersonEventV2::Created { person_id, name, .. } => {
                // Update projection
            }
            _ => {}
        }
        Ok(())
    }
    
    fn name(&self) -> &str {
        "my-projection"
    }
}
```

### Using Retry Handler

```rust
use cim_domain_person::infrastructure::{RetryHandler, RetryPolicy};

let retry_handler = RetryHandler::new(
    client,
    jetstream,
    RetryPolicy::default(),
    "person.dlq.>".to_string(),
);

// Execute with retry
let result = retry_handler.execute_with_retry(
    || Box::pin(async { process_event().await }),
    "process_event"
).await;
```

## Migration Path

### From Old Events to New

The system maintains backward compatibility. Old events can still be processed, but new events should use PersonEventV2:

```rust
// Old style (still works)
let old_event = PersonEvent::PersonCreated(PersonCreated {
    person_id,
    name,
    source,
    created_at,
});

// New style (preferred)
let new_event = PersonEventV2::Created {
    person_id,
    name,
    source,
    metadata: EventMetadata::new(),
};
```

## Testing

Run the example to see streaming in action:

```bash
cargo run --example streaming_setup
```

## Next Steps (Week 2)

- Convert all synchronous command handlers to async
- Implement streaming responses
- Remove blocking calls
- Update projection handlers to use streaming

## Benefits Achieved

1. **Full streaming support**: Events flow through NATS JetStream
2. **Resilience**: Retry policies and circuit breakers prevent failures
3. **Observability**: Event metadata enables tracing and correlation
4. **Scalability**: Consumer groups and streaming enable horizontal scaling
5. **Reliability**: Dead letter queues ensure no event is lost