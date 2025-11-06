//! Person aggregate - Pure functional DDD
//!
//! The Person aggregate represents core identity with minimal data.
//! This is a pure domain model following functional programming principles.

use cim_domain::DomainResult;

/// Trait for event sourced aggregates - Pure Functional
///
/// This trait follows FRP principles: event application consumes the aggregate
/// and returns a new one, maintaining immutability.
pub trait EventSourced: Sized {
    type Event;

    /// Apply an event to update aggregate state (pure functional)
    ///
    /// Consumes self and returns a new instance with the event applied.
    /// This is the Category Theory compliant version.
    fn apply_event(self, event: &Self::Event) -> DomainResult<Self>;

    /// Apply multiple events in sequence (pure functional)
    ///
    /// Uses fold to chain event applications functionally.
    fn apply_events(self, events: &[Self::Event]) -> DomainResult<Self> {
        events.iter().try_fold(self, |aggregate, event| {
            aggregate.apply_event(event)
        })
    }
}

// Export the person aggregate
pub mod person_ecs;
pub use person_ecs::{Person, PersonId, PersonMarker, CoreIdentity, PersonLifecycle};

// State machine framework
pub mod state_machine;
pub mod person_states;
pub mod person_onboarding;

pub use state_machine::{State, Command, StateMachine, StateMachineAggregate};
pub use person_states::{PersonState, PersonStateCommand, create_person_state_machine};
pub use person_onboarding::{PersonOnboarding, OnboardingState, OnboardingCommand};
