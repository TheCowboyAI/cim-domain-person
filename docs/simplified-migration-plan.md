# Simplified Migration Plan - Person Domain Pure Event-Driven

## Executive Summary

Based on the insight that sagas are simply aggregates with state machines (our standard practice), this simplified plan reduces the migration to **4-5 weeks** by leveraging existing patterns.

## Key Realization

Since we already build aggregates with state machines as standard practice, we don't need:
- Separate saga framework
- New process management layer
- Complex compensation mechanisms

Instead, we enhance our existing aggregate patterns to handle multi-step workflows.

## Simplified Architecture

```
Command → Aggregate (with State Machine) → Events → NATS JetStream → Projections
                    ↓
            State Transitions
            Business Rules
            Multi-step Workflows
```

## Revised Migration Plan

### Week 1: NATS Streaming Enhancement
**Focus**: Upgrade existing NATS integration to full streaming

#### Tasks
1. **Enhance NATS Configuration**
   ```rust
   // Update existing infrastructure/nats_integration.rs
   pub struct StreamingConfig {
       consumer_groups: HashMap<String, ConsumerConfig>,
       retention_policy: RetentionPolicy::Limits,
       max_age: Duration::days(30),
   }
   ```

2. **Add Event Metadata**
   ```rust
   // Simple extension to existing events
   pub struct EventMetadata {
       version: &'static str,
       correlation_id: Uuid,
       timestamp: DateTime<Utc>,
   }
   ```

3. **Streaming Subscriptions**
   - Convert existing subscribers to streaming
   - Add replay capability
   - Dead letter queue for failures

### Week 2: Async Everything
**Focus**: Remove remaining synchronous code paths

#### Tasks
1. **Convert Remaining Sync Handlers**
   - Identify ~20% sync code
   - Convert to async/await
   - Update tests

2. **Streaming Responses**
   ```rust
   // Return event streams instead of single responses
   pub async fn handle_command(cmd: Command) -> EventStream<PersonEvent>
   ```

### Week 3: Aggregate State Machines
**Focus**: Enhance existing aggregates with explicit state machines

#### Tasks
1. **Formalize State Machines**
   ```rust
   // Add to existing Person aggregate
   impl Person {
       pub fn state_machine() -> StateMachine<PersonState> {
           StateMachine::new()
               .transition(Draft, Create, Active)
               .transition(Active, Suspend, Suspended)
               .transition(Suspended, Reactivate, Active)
               .build()
       }
   }
   ```

2. **Multi-Step Workflows**
   ```rust
   // Example: Person onboarding as aggregate behavior
   pub struct PersonOnboarding {
       person_id: PersonId,
       state: OnboardingState,
       steps_completed: Vec<OnboardingStep>,
   }
   
   impl Aggregate for PersonOnboarding {
       // Standard aggregate with state machine
       fn handle_command(&mut self, cmd: Command) -> Result<Vec<Event>> {
           match self.state.can_transition(&cmd) {
               Ok(new_state) => {
                   let events = self.generate_events(cmd, new_state);
                   self.apply_events(&events);
                   Ok(events)
               }
               Err(e) => Err(e)
           }
       }
   }
   ```

### Week 4: Event Versioning & Policies
**Focus**: Add evolution support and business policies

#### Tasks
1. **Simple Event Versioning**
   ```rust
   // Use const versioning
   pub mod events {
       pub const PERSON_CREATED_V1: &str = "person.created.v1";
       pub const PERSON_CREATED_V2: &str = "person.created.v2";
       
       pub fn upcast_v1_to_v2(v1: PersonCreatedV1) -> PersonCreatedV2 {
           // Simple transformation
       }
   }
   ```

2. **Policy as Event Handlers**
   ```rust
   // Policies are just event handlers that generate commands
   pub async fn auto_archive_policy(event: PersonEvent) -> Option<Command> {
       match event {
           PersonEvent::InactiveFor(person_id, duration) if duration > 365.days() => {
               Some(Command::ArchivePerson { person_id })
           }
           _ => None
       }
   }
   ```

### Week 5: Testing & Migration
**Focus**: Validate and migrate

#### Tasks
1. **Comprehensive Testing**
   - Event flow tests
   - Performance benchmarks
   - Integration tests

2. **Data Migration**
   - Simple event replay
   - Projection rebuild
   - Validation

## What We're NOT Building

1. **Separate Saga Framework** - Aggregates with state machines handle this
2. **Complex Process Manager** - Standard aggregate pattern suffices
3. **New Compensation System** - Event sourcing provides natural rollback
4. **Heavy Policy Engine** - Simple event handlers work fine

## Simplified Timeline

```
Week 1: NATS Streaming (existing infrastructure enhancement)
Week 2: Async Conversion (straightforward refactoring)
Week 3: Aggregate State Machines (formalize existing patterns)
Week 4: Versioning & Policies (lightweight additions)
Week 5: Testing & Migration (validation and deployment)
```

## Key Simplifications

1. **Use Existing Patterns**: Aggregates with state machines ARE our sagas
2. **Minimal New Code**: ~90% enhancement of existing code
3. **No New Frameworks**: Leverage what we already have
4. **Standard Practices**: Follow established aggregate patterns

## Success Metrics

- **Week 1**: Streaming operational
- **Week 2**: 100% async
- **Week 3**: State machines formalized
- **Week 4**: Versioning live
- **Week 5**: Migration complete

## Benefits of Simplified Approach

1. **Faster Delivery**: 5 weeks vs 8 weeks vs original 16 weeks
2. **Less Risk**: Using proven patterns
3. **Team Familiarity**: Everyone knows aggregates with state machines
4. **Maintainable**: No new concepts to learn

## Example: Person Onboarding "Saga"

Instead of a separate saga framework:

```rust
// This IS our saga - just an aggregate with state machine
pub struct PersonOnboardingAggregate {
    id: OnboardingId,
    person_id: PersonId,
    state: OnboardingState,
    completed_steps: HashSet<Step>,
}

impl Aggregate for PersonOnboardingAggregate {
    type Command = OnboardingCommand;
    type Event = OnboardingEvent;
    
    fn handle(&mut self, cmd: Self::Command) -> Result<Vec<Self::Event>> {
        // State machine handles transitions
        // Each step generates events
        // Natural compensation through event replay
    }
}
```

## Conclusion

By recognizing that sagas are just aggregates with state machines (our standard practice), we can:
- Reduce complexity significantly
- Deliver in 5 weeks instead of 8-16
- Use familiar patterns
- Achieve the same pure event-driven goal

The migration becomes an enhancement of existing patterns rather than introducing new concepts.