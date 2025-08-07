# Architecture Guide

## System Architecture

The CIM Domain Person implements a pure event-driven architecture using Entity Component System (ECS) patterns for maximum flexibility and composability.

## Core Design Principles

### 1. Entity Component System (ECS)
- **Person as Entity**: Minimal core identity (ID + legal name)
- **Components**: Composable capabilities attached to entities
- **Systems**: Processors that operate on components

### 2. Event-Driven Architecture
All state changes flow through events:
```
Command → AsyncProcessor → Events → NATS JetStream
                              ↓
                         PolicyEngine → Reactive Commands
                              ↓
                         Projections → Read Models
```

### 3. CQRS Pattern
- **Commands**: Modify state through event generation
- **Queries**: Read from optimized projections
- **Event Store**: Single source of truth

## Component Architecture

### Core Components

#### Person Entity (`aggregate/person_ecs.rs`)
```rust
pub struct Person {
    pub id: PersonId,
    pub core_identity: CoreIdentity,
    pub lifecycle: PersonLifecycle,
}
```

#### Component Types
- **Contact Components**: Email, Phone, Address
- **Professional Components**: Skills, Experience, Certifications
- **Social Components**: Relationships, Networks
- **Preference Components**: Settings, Notifications

### Event System

#### Event Versioning (`events/versioning.rs`)
- Automatic schema evolution
- Backward compatibility
- Migration support

#### Event Types (`events/enhanced.rs`)
```rust
pub enum PersonEventV2 {
    Created { person_id, name, source, metadata },
    Updated { person_id, changes, metadata },
    ComponentAdded { person_id, component_type, data },
    // ...
}
```

### Command Processing

#### Async Command Processor (`handlers/async_command_processor.rs`)
- Streaming support for large operations
- Transaction boundaries
- Error recovery

#### Command Types (`commands/mod.rs`)
```rust
pub enum PersonCommand {
    CreatePerson(CreatePerson),
    UpdatePerson(UpdatePerson),
    AddComponent(AddComponent),
    // ...
}
```

### State Machines

#### Workflow Management (`aggregate/state_machine.rs`)
- Complex multi-step workflows
- State transitions with guards
- Action execution

#### Example: Onboarding Workflow
```rust
StateMachine::builder()
    .initial_state(OnboardingState::Started)
    .transition(Started, AddEmail, EmailAdded)
    .transition(EmailAdded, VerifyEmail, EmailVerified)
    .transition(EmailVerified, Complete, Completed)
    .build()
```

### Policy Engine

#### Reactive Business Rules (`policies/mod.rs`)
Policies react to events and generate commands:
- Welcome email on creation
- Data quality validation
- Skill recommendations
- Auto-archiving inactive persons

### Projections

#### Read Model Generation (`projections/mod.rs`)
- **Summary Projection**: Basic person information
- **Timeline Projection**: Activity history
- **Search Projection**: Full-text search index
- **Network Projection**: Relationship graphs
- **Skills Projection**: Competency matrix

### Infrastructure

#### Event Store (`infrastructure/event_store.rs`)
- Event persistence
- Snapshot storage
- Event streaming

#### NATS Integration (`infrastructure/nats_integration.rs`)
- JetStream for persistence
- Subject-based routing
- Consumer groups

#### Retry & Circuit Breaker (`infrastructure/retry.rs`)
- Exponential backoff
- Circuit breaker pattern
- Fault tolerance

## Data Flow

### Command Flow
1. Command received via API
2. Validated by command handler
3. Processed by aggregate
4. Events generated
5. Events persisted to store
6. Events published to NATS
7. Policies evaluated
8. Projections updated

### Query Flow
1. Query received via API
2. Routed to appropriate projection
3. Data retrieved from read model
4. Response returned to client

## Integration Points

### Cross-Domain Communication
- **Agent Domain**: Task delegation
- **Location Domain**: Address validation
- **Identity Domain**: Authentication/Authorization
- **Git Domain**: Version control integration

### External Systems
- NATS JetStream for messaging
- Database for projections
- External APIs via adapters

## Scalability Considerations

### Horizontal Scaling
- Stateless command processors
- Partitioned event streams
- Distributed projections

### Performance Optimizations
- Component lazy loading
- Event batching
- Projection caching
- Streaming for large datasets

## Security Architecture

### Authentication & Authorization
- Command-level authorization
- Event encryption for sensitive data
- Audit logging

### Data Protection
- PII handling in components
- GDPR compliance through event design
- Retention policies

## Deployment Architecture

### Container Architecture
- Microservice deployment
- Health checks
- Graceful shutdown

### Configuration Management
- Environment-based configuration
- Feature flags
- Dynamic policy loading