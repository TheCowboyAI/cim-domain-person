# Week 3: Aggregate State Machines - Completed

## Summary

Week 3 has been completed, formalizing state machines in aggregates. Following your standard practice where "sagas are simply aggregates with state machines," we've created a reusable framework and demonstrated both simple and complex workflows.

## What Was Added

### 1. Generic State Machine Framework (`aggregate/state_machine.rs`)
- **State trait**: Define states with terminal state support
- **Command trait**: Define commands that trigger transitions
- **Transition definitions**: Include guards and actions
- **State machine builder**: Fluent API for defining transitions
- **StateMachineAggregate trait**: Standard interface for aggregates

### 2. Person State Machine (`aggregate/person_states.rs`)
- **PersonState enum**: Formalized states (Draft, Active, Suspended, Archived, Deceased, MergedInto)
- **PersonStateCommand**: Commands that affect state
- **State transition rules**: Guards ensure valid transitions
- **Entry/exit actions**: Logging and side effects

### 3. Complex Workflow Example (`aggregate/person_onboarding.rs`)
- **PersonOnboarding aggregate**: Multi-step onboarding process
- **OnboardingState**: 8 states from Started to Completed/Failed
- **Command handling**: Each step generates appropriate events
- **Failure handling**: Can fail at any step with proper state

### 4. Integration with Person Aggregate
- State machine validation before command processing
- Automatic state transition checking
- Clear error messages for invalid transitions

## Key Concepts

### State Machine as Standard Practice

Instead of separate "saga" frameworks, we use aggregates with state machines:

```rust
pub struct PersonOnboarding {
    id: Uuid,
    person_id: PersonId,
    state: OnboardingState,
    // ... other fields
}

impl StateMachineAggregate for PersonOnboarding {
    type State = OnboardingState;
    type Command = OnboardingCommand;
    
    fn state_machine() -> StateMachine<Self::State, Self::Command> {
        // Define transitions
    }
}
```

### Building State Machines

Using the fluent builder API:

```rust
StateMachine::builder(InitialState)
    .transition(FromState, Command, ToState)
    .transition_with_guard(FromState, Command, ToState, |state, cmd| {
        // Validation logic
    })
    .transition_with_action(FromState, Command, ToState, |state, cmd| {
        // Side effects
    })
    .on_entry(State, |state| {
        // Entry actions
    })
    .on_exit(State, |state| {
        // Exit actions
    })
    .build()
```

### Multi-Step Workflows

Complex processes are just aggregates:

```rust
// What others might call a "saga"
pub struct PersonOnboarding {
    // This IS our saga - just an aggregate with state machine
}

// Handle each step
onboarding.handle_command(StartOnboarding)?;
onboarding.handle_command(VerifyIdentity { identity_id })?;
onboarding.handle_command(ProvideBasicInfo { email, phone })?;
// ... continue through workflow
```

## Usage Examples

### Simple State Transitions

```rust
// Person lifecycle transitions
let mut person = Person::new(person_id, name);

// State machine validates this transition
person.handle_command(DeactivatePerson { reason })?;

// Invalid transitions are caught
person.handle_command(ReactivatePerson { reason })?; // OK
person.handle_command(ReactivatePerson { reason })?; // Error: already active
```

### Complex Workflows

```rust
// Multi-step onboarding
let mut onboarding = PersonOnboarding::new(person_id);

// Each step moves through the state machine
onboarding.handle_command(StartOnboarding)?;
onboarding.handle_command(VerifyIdentity { identity_id })?;
onboarding.handle_command(ProvideBasicInfo { email, phone })?;
onboarding.handle_command(AddComponents { components })?;
onboarding.handle_command(AssignLocation { location_id })?;
onboarding.handle_command(CompleteOnboarding)?;

// State machine ensures correct order
```

## Creating Your Own Workflows

Any multi-step process can be an aggregate:

```rust
// 1. Define states
enum ApprovalState {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
}

// 2. Define commands
enum ApprovalCommand {
    Submit,
    StartReview,
    Approve,
    Reject,
}

// 3. Create aggregate
struct ApprovalWorkflow {
    id: Uuid,
    state: ApprovalState,
    // ... other fields
}

// 4. Implement state machine
impl StateMachineAggregate for ApprovalWorkflow {
    // ... implementation
}
```

## Benefits Achieved

1. **Formalized state management**: All state transitions are explicit
2. **Invalid transitions prevented**: State machine validates all commands
3. **Complex workflows simplified**: Multi-step processes are just aggregates
4. **No separate saga framework**: Standard pattern for all workflows
5. **Reusable framework**: Easy to create new state machines

## Testing State Machines

Run the example to see state machines in action:

```bash
cargo run --example state_machine_demo
```

## Migration Notes

- Existing Person aggregate now validates state transitions
- No breaking changes to external APIs
- State machine is internal implementation detail
- Can gradually add state machines to other aggregates

## Next Steps (Week 4)

- Add event versioning support
- Implement schema evolution
- Create migration strategies
- Add backward compatibility

## Key Takeaways

1. **Sagas = Aggregates with State Machines**: No need for separate frameworks
2. **Explicit is better**: All transitions are clearly defined
3. **Guards ensure validity**: Business rules enforced at transition time
4. **Actions enable side effects**: Entry/exit actions for cross-cutting concerns
5. **Reusable pattern**: Same approach works for any workflow