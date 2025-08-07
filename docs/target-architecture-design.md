# Target Architecture Design - Pure Event-Driven Person Domain

## Overview

Based on the cim-graph patterns, this document defines the target architecture for a pure event-driven person domain that leverages existing strengths while adding missing capabilities.

## Core Principles (from cim-graph)

1. **Events as Single Source of Truth**: All state changes happen through events
2. **NATS JetStream as Event Backbone**: Persistent, ordered event streams
3. **Command-Event-Projection Pattern**: Strict separation of write and read models
4. **Process Management**: Sagas for complex multi-step operations
5. **Event-First Design**: Commands are requests to generate events

## Target Architecture

### Event Flow
```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│   Command   │────▶│Process Manager│────▶│Event Producer│
│  (Request)  │     │(Validation)  │     │   (Write)    │
└─────────────┘     └──────────────┘     └──────┬───────┘
                                                 │
                          ┌──────────────────────┼───────────────────┐
                          │           NATS JetStream                 │
                          │  ┌─────────────────────────────────┐    │
                          │  │ person.events.lifecycle.created │    │
                          │  │ person.events.component.added   │    │
                          │  │ person.events.relationship.*    │    │
                          │  └─────────────────────────────────┘    │
                          └──────────────┬───────────────────────────┘
                                         │
     ┌───────────────┬───────────────────┼──────────────┬────────────────┐
     ▼               ▼                   ▼              ▼                ▼
┌─────────┐    ┌─────────┐      ┌──────────────┐ ┌──────────┐   ┌──────────┐
│Projection│    │Projection│     │Process Manager│ │Cross-Domain│  │Policy   │
│(Summary) │    │(Search)  │     │  (Sagas)     │ │Integration │  │Engine   │
└─────────┘    └─────────┘      └──────────────┘ └──────────┘   └──────────┘
```

### Key Components

#### 1. Event Definitions (Enhanced)
```rust
#[derive(Event, Serialize, Deserialize)]
pub enum PersonEvent {
    // Lifecycle events with metadata
    Created { id: PersonId, name: Name, metadata: EventMetadata },
    Activated { id: PersonId, reason: String, metadata: EventMetadata },
    Suspended { id: PersonId, reason: String, metadata: EventMetadata },
    
    // Component events
    ComponentAdded { 
        person_id: PersonId, 
        component_type: ComponentType,
        component_data: serde_json::Value,
        metadata: EventMetadata 
    },
    
    // Versioned for evolution
    NameUpdatedV2 { 
        id: PersonId, 
        old_name: Name, 
        new_name: Name,
        change_reason: Option<String>,
        metadata: EventMetadata 
    },
}
```

#### 2. Aggregates with State Machines (Standard Pattern)
```rust
// Person aggregate with built-in state machine
pub struct Person {
    id: PersonId,
    name: Name,
    state: PersonState,
}

impl Person {
    pub fn state_machine() -> StateMachine<PersonState> {
        StateMachine::new()
            .transition(Draft, Create, Active)
            .transition(Active, Suspend, Suspended)
            .transition(Suspended, Reactivate, Active)
            .transition(Active, Archive, Archived)
            .build()
    }
    
    pub fn handle_command(&mut self, cmd: PersonCommand) -> Result<Vec<PersonEvent>> {
        // State machine validates transitions
        let new_state = Self::state_machine().validate_transition(&self.state, &cmd)?;
        
        // Generate events based on valid transition
        let events = match cmd {
            PersonCommand::Create { name } => vec![
                PersonEvent::Created { 
                    id: self.id, 
                    name, 
                    metadata: EventMetadata::new() 
                }
            ],
            // ... other commands
        };
        
        Ok(events)
    }
}

// Complex workflows as aggregates (what others call "sagas")
pub struct PersonOnboarding {
    id: OnboardingId,
    person_id: PersonId,
    state: OnboardingState,
    steps: Vec<OnboardingStep>,
}

impl PersonOnboarding {
    pub fn handle_command(&mut self, cmd: OnboardingCommand) -> Result<Vec<Event>> {
        // Multi-step workflow handled as aggregate with state machine
        match (self.state, cmd) {
            (WaitingForIdentityVerification, IdentityVerified) => {
                self.state = WaitingForLocationAssignment;
                Ok(vec![OnboardingStepCompleted { step: IdentityStep }])
            }
            // ... other transitions
        }
    }
}
```

#### 3. Streaming Subscriptions (Enhanced)
```rust
pub struct PersonEventSubscriber {
    jetstream: JetStreamContext,
    handlers: HashMap<String, Box<dyn EventHandler>>,
}

impl PersonEventSubscriber {
    pub async fn start_streaming(&mut self) -> Result<()> {
        let config = ConsumerConfig {
            durable_name: Some("person-projection-consumer".to_string()),
            deliver_policy: DeliverPolicy::All,
            ack_policy: AckPolicy::Explicit,
            max_deliver: 3,
            ..Default::default()
        };
        
        let stream = self.jetstream
            .subscribe_with_config("person.events.>", config)
            .await?;
            
        while let Some(msg) = stream.next().await {
            self.process_message(msg).await?;
        }
        
        Ok(())
    }
}
```

#### 4. Event Versioning Support (New)
```rust
pub trait EventMigration {
    fn from_version(&self) -> &str;
    fn to_version(&self) -> &str;
    fn migrate(&self, old_event: &[u8]) -> Result<Event>;
}

pub struct EventVersionManager {
    migrations: HashMap<(String, String), Box<dyn EventMigration>>,
}
```

#### 5. Policy Implementation (Simplified)
```rust
// Policies as simple event handlers
pub async fn apply_policies(event: &PersonEvent) -> Vec<Command> {
    let mut commands = vec![];
    
    // Auto-archive policy
    if let PersonEvent::InactiveFor { person_id, days } = event {
        if *days > 365 {
            commands.push(Command::ArchivePerson { id: *person_id });
        }
    }
    
    // Skill recommendation policy
    if let PersonEvent::ComponentAdded { person_id, component_type, .. } = event {
        if component_type == &ComponentType::GitProfile {
            commands.push(Command::AnalyzeSkills { person_id: *person_id });
        }
    }
    
    commands
}
```

### Subject Hierarchy (Refined)

```
person.commands.create
person.commands.update
person.commands.lifecycle.*

person.events.lifecycle.created
person.events.lifecycle.activated
person.events.lifecycle.suspended
person.events.identity.updated
person.events.component.added
person.events.component.updated
person.events.relationship.location_assigned
person.events.relationship.employment_added

person.queries.get_by_id
person.queries.search
person.queries.get_projection
```

### Key Enhancements Over Current State

1. **Pure Async Processing**
   - All command handlers are async
   - No synchronous aggregate access
   - Streaming responses for real-time updates

2. **Advanced Event Features**
   - Event versioning with migration support
   - Dead letter queue for failed events
   - Event replay with checkpointing

3. **Process Management**
   - Saga pattern for complex workflows
   - Compensation for failed operations
   - Distributed transaction support

4. **Enhanced Observability**
   - OpenTelemetry integration
   - Event flow tracing
   - Performance metrics per event type

5. **Policy-Driven Behaviors**
   - Automatic event generation
   - Business rule enforcement
   - Dynamic policy updates

## Migration Strategy

### Phase 1: Foundation (1 week)
- Update NATS configuration for streaming
- Add event versioning metadata
- Implement process manager skeleton

### Phase 2: Core Refactoring (2 weeks)
- Convert all command handlers to async
- Implement streaming subscriptions
- Add saga support for complex operations

### Phase 3: Enhanced Features (2 weeks)
- Add event versioning and migrations
- Implement policy engine
- Add dead letter queue handling

### Phase 4: Observability (1 week)
- Add OpenTelemetry integration
- Implement event flow monitoring
- Create operational dashboards

### Phase 5: Testing & Migration (2 weeks)
- Comprehensive testing
- Data migration scripts
- Performance optimization

## Success Criteria

1. **All state changes through events**: No direct mutations
2. **Event replay capability**: Can rebuild state from events
3. **Zero message loss**: All events persisted in JetStream
4. **Saga completion rate**: >99.9% for critical workflows
5. **Event processing latency**: <10ms p95

## Conclusion

The target architecture builds on the existing strong foundation, adding:
- Process management for complex workflows
- Event versioning for evolution
- Enhanced streaming capabilities
- Full observability

Total effort: **6-8 weeks** vs. original 16-week estimate.