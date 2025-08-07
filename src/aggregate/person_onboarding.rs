//! Person onboarding workflow - a multi-step aggregate with state machine

use super::state_machine::{State, Command, StateMachine, StateMachineAggregate};
use crate::aggregate::PersonId;
use crate::events::{PersonEventV2, EventMetadata};
use cim_domain::DomainResult;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Onboarding workflow states
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OnboardingState {
    /// Initial state
    Started,
    /// Waiting for identity verification
    AwaitingIdentityVerification,
    /// Identity verified, collecting basic info
    CollectingBasicInfo,
    /// Basic info collected, setting up components
    SettingUpComponents,
    /// Components set up, assigning location
    AssigningLocation,
    /// Location assigned, finalizing
    Finalizing,
    /// Successfully completed
    Completed,
    /// Failed onboarding
    Failed { reason: String },
}

impl State for OnboardingState {}

/// Onboarding commands
#[derive(Clone, Debug)]
pub enum OnboardingCommand {
    StartOnboarding,
    VerifyIdentity { identity_id: Uuid },
    ProvideBasicInfo { email: String, phone: String },
    AddComponents { components: Vec<ComponentData> },
    AssignLocation { location_id: Uuid },
    CompleteOnboarding,
    FailOnboarding { reason: String },
}

impl Command for OnboardingCommand {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentData {
    pub component_type: String,
    pub data: serde_json::Value,
}

/// Person onboarding aggregate - a multi-step workflow
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonOnboarding {
    pub id: Uuid,
    pub person_id: PersonId,
    pub state: OnboardingState,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub identity_id: Option<Uuid>,
    pub basic_info: Option<BasicInfo>,
    pub components_added: Vec<ComponentData>,
    pub location_id: Option<Uuid>,
    pub version: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BasicInfo {
    pub email: String,
    pub phone: String,
}

impl PersonOnboarding {
    /// Create a new onboarding workflow
    pub fn new(person_id: PersonId) -> Self {
        Self {
            id: Uuid::new_v4(),
            person_id,
            state: OnboardingState::Started,
            started_at: Utc::now(),
            completed_at: None,
            identity_id: None,
            basic_info: None,
            components_added: Vec::new(),
            location_id: None,
            version: 0,
        }
    }
    
    /// Handle a command and generate events
    pub fn handle_command(&mut self, command: OnboardingCommand) -> DomainResult<Vec<PersonEventV2>> {
        let new_state = self.handle_command_with_state_machine(command.clone())?;
        let mut events = Vec::new();
        
        // Generate events based on command
        match command {
            OnboardingCommand::VerifyIdentity { identity_id } => {
                self.identity_id = Some(identity_id);
                events.push(PersonEventV2::IdentityLinked {
                    person_id: self.person_id,
                    identity_id,
                    identity_type: "primary".to_string(),
                    metadata: EventMetadata::new(),
                });
            }
            
            OnboardingCommand::ProvideBasicInfo { email, phone } => {
                self.basic_info = Some(BasicInfo { email: email.clone(), phone: phone.clone() });
                
                // Add email component
                events.push(PersonEventV2::ComponentAdded {
                    person_id: self.person_id,
                    component_type: crate::aggregate::ComponentType::EmailAddress,
                    component_data: serde_json::json!({
                        "email": email,
                        "type": "primary",
                        "verified": false
                    }),
                    metadata: EventMetadata::new(),
                });
                
                // Add phone component
                events.push(PersonEventV2::ComponentAdded {
                    person_id: self.person_id,
                    component_type: crate::aggregate::ComponentType::PhoneNumber,
                    component_data: serde_json::json!({
                        "phone": phone,
                        "type": "primary",
                        "verified": false
                    }),
                    metadata: EventMetadata::new(),
                });
            }
            
            OnboardingCommand::AddComponents { components } => {
                for component in components {
                    self.components_added.push(component.clone());
                    events.push(PersonEventV2::ComponentAdded {
                        person_id: self.person_id,
                        component_type: crate::aggregate::ComponentType::CustomAttribute,
                        component_data: component.data,
                        metadata: EventMetadata::new(),
                    });
                }
            }
            
            OnboardingCommand::AssignLocation { location_id } => {
                self.location_id = Some(location_id);
                events.push(PersonEventV2::LocationAssigned {
                    person_id: self.person_id,
                    location_id,
                    location_type: "primary".to_string(),
                    metadata: EventMetadata::new(),
                });
            }
            
            OnboardingCommand::CompleteOnboarding => {
                self.completed_at = Some(Utc::now());
                events.push(PersonEventV2::Activated {
                    person_id: self.person_id,
                    reason: "Onboarding completed".to_string(),
                    metadata: EventMetadata::new(),
                });
            }
            
            OnboardingCommand::FailOnboarding { reason } => {
                events.push(PersonEventV2::Suspended {
                    person_id: self.person_id,
                    reason: format!("Onboarding failed: {}", reason),
                    metadata: EventMetadata::new(),
                });
            }
            
            _ => {}
        }
        
        self.version += 1;
        Ok(events)
    }
}

impl StateMachineAggregate for PersonOnboarding {
    type State = OnboardingState;
    type Command = OnboardingCommand;
    
    fn current_state(&self) -> &Self::State {
        &self.state
    }
    
    fn set_state(&mut self, state: Self::State) {
        self.state = state;
    }
    
    fn state_machine() -> StateMachine<Self::State, Self::Command> {
        StateMachine::builder(OnboardingState::Started)
            // Started -> AwaitingIdentityVerification
            .transition(
                OnboardingState::Started,
                OnboardingCommand::StartOnboarding,
                OnboardingState::AwaitingIdentityVerification,
            )
            
            // AwaitingIdentityVerification -> CollectingBasicInfo
            .transition_with_guard(
                OnboardingState::AwaitingIdentityVerification,
                OnboardingState::CollectingBasicInfo,
                |_state, cmd| {
                    if let OnboardingCommand::VerifyIdentity { identity_id } = cmd {
                        identity_id != &Uuid::nil()
                    } else {
                        false
                    }
                },
            )
            
            // CollectingBasicInfo -> SettingUpComponents
            .transition_with_guard(
                OnboardingState::CollectingBasicInfo,
                OnboardingState::SettingUpComponents,
                |_state, cmd| {
                    if let OnboardingCommand::ProvideBasicInfo { email, phone } = cmd {
                        !email.is_empty() && !phone.is_empty()
                    } else {
                        false
                    }
                },
            )
            
            // SettingUpComponents -> AssigningLocation
            .transition(
                OnboardingState::SettingUpComponents,
                OnboardingCommand::AddComponents { components: Vec::new() },
                OnboardingState::AssigningLocation,
            )
            
            // AssigningLocation -> Finalizing
            .transition(
                OnboardingState::AssigningLocation,
                OnboardingCommand::AssignLocation { location_id: Uuid::nil() },
                OnboardingState::Finalizing,
            )
            
            // Finalizing -> Completed
            .transition(
                OnboardingState::Finalizing,
                OnboardingCommand::CompleteOnboarding,
                OnboardingState::Completed,
            )
            
            // Any non-terminal state -> Failed
            .transition(
                OnboardingState::AwaitingIdentityVerification,
                OnboardingCommand::FailOnboarding { reason: String::new() },
                OnboardingState::Failed { reason: String::new() },
            )
            .transition(
                OnboardingState::CollectingBasicInfo,
                OnboardingCommand::FailOnboarding { reason: String::new() },
                OnboardingState::Failed { reason: String::new() },
            )
            .transition(
                OnboardingState::SettingUpComponents,
                OnboardingCommand::FailOnboarding { reason: String::new() },
                OnboardingState::Failed { reason: String::new() },
            )
            .transition(
                OnboardingState::AssigningLocation,
                OnboardingCommand::FailOnboarding { reason: String::new() },
                OnboardingState::Failed { reason: String::new() },
            )
            
            // Add logging for state transitions
            .on_entry(OnboardingState::Completed, |_state| {
                tracing::info!("Onboarding completed successfully");
                Ok(())
            })
            .on_entry(OnboardingState::Failed { reason: String::new() }, |state| {
                if let OnboardingState::Failed { reason } = state {
                    tracing::error!("Onboarding failed: {}", reason);
                }
                Ok(())
            })
            
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_onboarding_workflow() {
        let mut onboarding = PersonOnboarding::new(PersonId::new());
        
        // Start onboarding
        let result = onboarding.handle_command(OnboardingCommand::StartOnboarding);
        assert!(result.is_ok());
        assert_eq!(onboarding.state, OnboardingState::AwaitingIdentityVerification);
        
        // Verify identity
        let identity_id = Uuid::new_v4();
        let result = onboarding.handle_command(OnboardingCommand::VerifyIdentity { identity_id });
        assert!(result.is_ok());
        assert_eq!(onboarding.state, OnboardingState::CollectingBasicInfo);
        assert_eq!(onboarding.identity_id, Some(identity_id));
        
        // Provide basic info
        let result = onboarding.handle_command(OnboardingCommand::ProvideBasicInfo {
            email: "test@example.com".to_string(),
            phone: "+1234567890".to_string(),
        });
        assert!(result.is_ok());
        assert_eq!(onboarding.state, OnboardingState::SettingUpComponents);
        
        // Continue through the workflow...
    }
}