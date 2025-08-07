//! Person aggregate - ECS architecture
//! 
//! The Person aggregate represents core identity with minimal data.
//! All other information is managed through components in the ECS system.

use cim_domain::DomainResult;

/// Trait for event sourced aggregates
pub trait EventSourced {
    type Event;
    
    /// Apply an event to update aggregate state
    fn apply_event(&mut self, event: &Self::Event) -> DomainResult<()>;
    
    /// Apply multiple events in sequence
    fn apply_events(&mut self, events: &[Self::Event]) -> DomainResult<()> {
        for event in events {
            self.apply_event(event)?;
        }
        Ok(())
    }
}

// Export the ECS-oriented person aggregate
pub mod person_ecs;
pub use person_ecs::{Person, PersonId, PersonMarker, CoreIdentity, PersonLifecycle, ComponentType};

// State machine framework
pub mod state_machine;
pub mod person_states;
pub mod person_onboarding;

pub use state_machine::{State, Command, StateMachine, StateMachineAggregate};
pub use person_states::{PersonState, PersonStateCommand, create_person_state_machine};
pub use person_onboarding::{PersonOnboarding, OnboardingState, OnboardingCommand};
