//! State machine definitions for Person aggregate

use super::state_machine::{State, Command, StateMachine};
use crate::aggregate::{PersonLifecycle, PersonId};
use crate::commands::{PersonCommand, MergeReason};
use cim_domain::formal_domain::AggregateState;
use serde::{Deserialize, Serialize};

/// States for the Person aggregate
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonState {
    /// Initial state when creating
    Draft,
    /// Normal active state
    Active,
    /// Temporarily suspended
    Suspended { reason: String },
    /// Permanently archived
    Archived { reason: String },
    /// Person has died
    Deceased { date_of_death: chrono::NaiveDate },
    /// Merged into another person
    MergedInto { merged_into_id: PersonId, reason: MergeReason },
}

impl State for PersonState {}

// Implement AggregateState trait for formal Category Theory compliance
impl AggregateState for PersonState {
    fn all_states() -> Vec<Self> {
        // Return representative states (can't enumerate all possible reasons/dates)
        vec![
            PersonState::Draft,
            PersonState::Active,
            PersonState::Suspended { reason: String::from("example") },
            PersonState::Archived { reason: String::from("example") },
            PersonState::Deceased {
                date_of_death: chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
            },
            PersonState::MergedInto {
                merged_into_id: PersonId::new(),
                reason: MergeReason::DuplicateIdentity
            },
        ]
    }

    fn initial() -> Self {
        PersonState::Draft
    }

    fn is_terminal(&self) -> bool {
        matches!(
            self,
            PersonState::Deceased { .. }
            | PersonState::MergedInto { .. }
            | PersonState::Archived { .. }
        )
    }
}

impl From<PersonLifecycle> for PersonState {
    fn from(lifecycle: PersonLifecycle) -> Self {
        match lifecycle {
            PersonLifecycle::Active => PersonState::Active,
            PersonLifecycle::Deactivated { .. } => PersonState::Suspended {
                reason: "Deactivated".to_string(),
            },
            PersonLifecycle::Deceased { date_of_death } => PersonState::Deceased { date_of_death },
            PersonLifecycle::MergedInto { target_id, merged_at: _ } => {
                PersonState::MergedInto { 
                    merged_into_id: target_id, 
                    reason: MergeReason::DuplicateIdentity // Default reason since PersonLifecycle doesn't have it
                }
            }
        }
    }
}

/// Commands for Person state transitions
#[derive(Clone, Debug)]
pub enum PersonStateCommand {
    Create,
    Activate,
    Suspend { reason: String },
    Archive { reason: String },
    RecordDeath { date_of_death: chrono::NaiveDate },
    Merge { target_id: PersonId, reason: MergeReason },
}

impl Command for PersonStateCommand {}

impl PersonStateCommand {
    /// Convert from PersonCommand
    pub fn from_person_command(cmd: &PersonCommand) -> Option<Self> {
        match cmd {
            PersonCommand::CreatePerson(_) => Some(PersonStateCommand::Create),
            PersonCommand::DeactivatePerson(cmd) => Some(PersonStateCommand::Suspend {
                reason: cmd.reason.clone(),
            }),
            PersonCommand::ReactivatePerson(_) => Some(PersonStateCommand::Activate),
            PersonCommand::RecordDeath(cmd) => Some(PersonStateCommand::RecordDeath {
                date_of_death: cmd.date_of_death,
            }),
            PersonCommand::MergePersons(cmd) => Some(PersonStateCommand::Merge {
                target_id: cmd.target_person_id,
                reason: cmd.merge_reason.clone(),
            }),
            _ => None, // Other commands don't affect state
        }
    }
}

/// Create the Person state machine
pub fn create_person_state_machine() -> StateMachine<PersonState, PersonStateCommand> {
    StateMachine::builder(PersonState::Draft)
        // Draft -> Active (on creation)
        .transition(
            PersonState::Draft,
            PersonStateCommand::Create,
            PersonState::Active,
        )
        
        // Active -> Suspended
        .transition_with_action(
            PersonState::Active,
            PersonState::Suspended { reason: String::new() },
            |_state, cmd| {
                if let PersonStateCommand::Suspend { reason } = cmd {
                    if reason.is_empty() {
                        return Err(cim_domain::DomainError::ValidationError(
                            "Suspension reason cannot be empty".to_string()
                        ));
                    }
                }
                Ok(())
            },
        )
        
        // Suspended -> Active
        .transition(
            PersonState::Suspended { reason: String::new() },
            PersonStateCommand::Activate,
            PersonState::Active,
        )
        
        // Active -> Archived
        .transition_with_guard(
            PersonState::Active,
            PersonState::Archived { reason: String::new() },
            |_state, cmd| {
                if let PersonStateCommand::Archive { reason } = cmd {
                    !reason.is_empty()
                } else {
                    false
                }
            },
        )
        
        // Active -> Deceased
        .transition(
            PersonState::Active,
            PersonStateCommand::RecordDeath { date_of_death: chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap() },
            PersonState::Deceased { date_of_death: chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap() },
        )
        
        // Active -> MergedInto
        .add_transition(
            PersonState::Active,
            PersonState::MergedInto {
                merged_into_id: PersonId::new(),
                reason: MergeReason::DuplicateIdentity,
            },
            |_state, _cmd| Ok(()),
        )
        
        // Suspended -> Archived
        .transition(
            PersonState::Suspended { reason: String::new() },
            PersonStateCommand::Archive { reason: String::new() },
            PersonState::Archived { reason: String::new() },
        )
        
        // Add entry/exit actions
        .on_entry(PersonState::Active, |_state| {
            tracing::info!("Person activated");
            Ok(())
        })
        .on_exit(PersonState::Active, |_state| {
            tracing::info!("Person leaving active state");
            Ok(())
        })
        .on_entry(PersonState::Archived { reason: String::new() }, |state| {
            if let PersonState::Archived { reason } = state {
                tracing::info!("Person archived: {}", reason);
            }
            Ok(())
        })
        
        .build()
}