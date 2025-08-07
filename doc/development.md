# Development Guide

## Setup

### Prerequisites

- Rust 1.70+ (with cargo)
- NATS Server 2.9+ with JetStream enabled
- PostgreSQL 14+ (for projections)
- Docker & Docker Compose (optional)

### Installation

```bash
# Clone repository
git clone https://github.com/thecowboyai/cim-domain-person.git
cd cim-domain-person

# Install dependencies
cargo build

# Run tests
cargo test

# Start development environment
docker-compose up -d
```

### Development Environment

```yaml
# docker-compose.yml
version: '3.8'
services:
  nats:
    image: nats:2.9-alpine
    ports:
      - "4222:4222"
      - "8222:8222"
    command: "-js -m 8222"
  
  postgres:
    image: postgres:14-alpine
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: person_domain
      POSTGRES_USER: developer
      POSTGRES_PASSWORD: developer
  
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
```

## Project Structure

```
cim-domain-person/
├── src/
│   ├── aggregate/          # Domain aggregates and state machines
│   ├── commands/           # Command definitions and handlers
│   ├── components/         # ECS components
│   ├── cross_domain/       # Cross-domain integrations
│   ├── events/             # Event definitions and versioning
│   ├── handlers/           # Command and query processors
│   ├── infrastructure/     # Technical infrastructure
│   ├── policies/           # Business rule policies
│   ├── projections/        # Read model projections
│   ├── queries/            # Query definitions
│   ├── services/           # Domain services
│   ├── value_objects/      # Value objects
│   └── lib.rs             # Library entry point
├── examples/               # Usage examples
├── tests/                  # Integration tests
├── benches/               # Performance benchmarks
└── doc/                   # Documentation
```

## Development Workflow

### Adding a New Command

1. Define command in `src/commands/mod.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyNewCommand {
    pub person_id: PersonId,
    pub field: String,
    // ... other fields
}
```

2. Add to command enum:
```rust
pub enum PersonCommand {
    // ... existing commands
    MyNew(MyNewCommand),
}
```

3. Implement handler in `src/handlers/command_handlers.rs`:
```rust
pub async fn handle_my_new_command(
    command: MyNewCommand,
    store: &dyn EventStore,
) -> Result<Vec<PersonEventV2>, DomainError> {
    // Validate command
    validate_my_command(&command)?;
    
    // Load aggregate
    let mut person = load_person(command.person_id, store).await?;
    
    // Apply business logic
    person.apply_my_change(command.field)?;
    
    // Generate events
    let event = PersonEventV2::MyEventHappened {
        person_id: command.person_id,
        field: command.field,
        metadata: EventMetadata::new(),
    };
    
    // Store events
    store.append(command.person_id, vec![event.clone()]).await?;
    
    Ok(vec![event])
}
```

4. Add tests:
```rust
#[tokio::test]
async fn test_my_new_command() {
    let store = InMemoryEventStore::new();
    let command = MyNewCommand {
        person_id: PersonId::new(),
        field: "value".to_string(),
    };
    
    let events = handle_my_new_command(command, &store).await.unwrap();
    
    assert_eq!(events.len(), 1);
    matches!(events[0], PersonEventV2::MyEventHappened { .. });
}
```

### Adding a New Component

1. Define component in `src/components/data/`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Component)]
pub struct MyComponent {
    pub id: ComponentId,
    pub person_id: PersonId,
    pub data: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MyComponent {
    pub fn new(person_id: PersonId, data: String) -> Self {
        Self {
            id: ComponentId::new(),
            person_id,
            data,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validation logic
        Ok(())
    }
}
```

2. Add component commands and events
3. Update component store
4. Add component handler

### Adding a New Projection

1. Define projection in `src/projections/`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyProjection {
    pub person_id: PersonId,
    pub computed_field: String,
    pub last_updated: DateTime<Utc>,
}

#[async_trait]
impl Projection for MyProjection {
    async fn handle_event(&mut self, event: &PersonEventV2) -> Result<(), ProjectionError> {
        match event {
            PersonEventV2::Created { person_id, .. } => {
                self.person_id = *person_id;
                self.last_updated = Utc::now();
            }
            // Handle other events
            _ => {}
        }
        Ok(())
    }
}
```

2. Register projection in handler
3. Add query support

## Testing

### Unit Tests

```bash
# Run all unit tests
cargo test --lib

# Run specific test
cargo test test_person_creation

# Run with coverage
cargo tarpaulin --out Html
```

### Integration Tests

```bash
# Run integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test streaming_tests
```

### Performance Tests

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench event_processing

# Profile with flamegraph
cargo flamegraph --bench event_processing
```

### Test Helpers

```rust
use cim_domain_person::test_helpers::*;

#[tokio::test]
async fn test_with_fixtures() {
    // Use test fixtures
    let person = test_fixtures::create_test_person();
    let email = test_fixtures::create_test_email();
    
    // Use test builders
    let person = PersonBuilder::new()
        .with_name("Alice", "Smith")
        .with_email("alice@example.com")
        .with_skill("Rust", ProficiencyLevel::Expert)
        .build();
    
    // Use test environment
    let env = TestEnvironment::new().await;
    let result = env.process_command(command).await;
}
```

## Code Style

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Fix clippy suggestions
cargo clippy --fix
```

### Code Organization

- Keep modules focused and cohesive
- Use descriptive names
- Document public APIs
- Write tests alongside code
- Follow Rust naming conventions

## Documentation

### Inline Documentation

```rust
/// Creates a new person with the given name.
///
/// # Arguments
/// * `name` - The person's legal name
///
/// # Returns
/// * `Ok(Person)` - Successfully created person
/// * `Err(DomainError)` - If validation fails
///
/// # Example
/// ```
/// let person = Person::new(
///     PersonId::new(),
///     PersonName::new("Alice", "Smith")
/// )?;
/// ```
pub fn new(id: PersonId, name: PersonName) -> Result<Self, DomainError> {
    // Implementation
}
```

### Generate Documentation

```bash
# Generate and open docs
cargo doc --open

# Generate with private items
cargo doc --document-private-items
```

## Debugging

### Logging

```rust
use tracing::{debug, error, info, warn};

#[tracing::instrument]
pub async fn process_command(command: PersonCommand) -> Result<(), DomainError> {
    info!("Processing command: {:?}", command);
    
    match validate_command(&command) {
        Ok(_) => debug!("Command validated"),
        Err(e) => {
            error!("Validation failed: {}", e);
            return Err(e);
        }
    }
    
    // Process command
    Ok(())
}
```

### Environment Variables

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable specific module logging
RUST_LOG=cim_domain_person::handlers=debug cargo run

# Pretty print logs
RUST_LOG=info RUST_LOG_STYLE=pretty cargo run
```

### Debugging Tools

```bash
# Use rust-gdb
rust-gdb target/debug/cim-domain-person

# Use rust-lldb
rust-lldb target/debug/cim-domain-person

# Memory profiling with valgrind
valgrind --leak-check=full target/debug/cim-domain-person
```

## Performance Optimization

### Profiling

```bash
# CPU profiling with perf
cargo build --release
perf record -g target/release/cim-domain-person
perf report

# Memory profiling with heaptrack
heaptrack target/release/cim-domain-person
heaptrack_gui heaptrack.cim-domain-person.*.gz
```

### Optimization Tips

1. **Use async/await effectively**
   - Avoid blocking operations
   - Use tokio::spawn for parallel tasks
   - Consider using FuturesUnordered

2. **Optimize data structures**
   - Use appropriate collection types
   - Consider using SmallVec for small collections
   - Use Arc for shared immutable data

3. **Minimize allocations**
   - Reuse buffers
   - Use object pools for frequently created objects
   - Consider using stack-allocated alternatives

4. **Database optimization**
   - Use connection pooling
   - Batch operations when possible
   - Create appropriate indexes

## Deployment

### Building for Production

```bash
# Build release binary
cargo build --release

# Build with optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Strip debug symbols
strip target/release/cim-domain-person
```

### Docker Build

```dockerfile
# Dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/cim-domain-person /usr/local/bin/
CMD ["cim-domain-person"]
```

### Configuration

```toml
# config.toml
[server]
host = "0.0.0.0"
port = 8080

[nats]
url = "nats://localhost:4222"
jetstream = true

[database]
url = "postgresql://user:pass@localhost/person_domain"
max_connections = 20

[logging]
level = "info"
format = "json"
```

## Troubleshooting

### Common Issues

1. **NATS Connection Failed**
   ```
   Error: Failed to connect to NATS
   Solution: Ensure NATS server is running with JetStream enabled
   ```

2. **Database Migration Failed**
   ```
   Error: Migration version mismatch
   Solution: Run: cargo run --bin migrate
   ```

3. **Component Not Found**
   ```
   Error: Component with ID not found
   Solution: Check component store consistency
   ```

### Debug Commands

```bash
# Check system health
cargo run --bin health-check

# Verify event store
cargo run --bin verify-events

# Rebuild projections
cargo run --bin rebuild-projections

# Export domain state
cargo run --bin export-state > state.json
```