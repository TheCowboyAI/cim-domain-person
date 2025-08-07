//! Integration tests for aggregate state machines

use cim_domain_person::{
    aggregate::{
        state_machine::{StateMachine, Transition},
        person_onboarding::{OnboardingState, OnboardingCommand, OnboardingAggregate},
        PersonId,
    },
    value_objects::PersonName,
};
use cim_domain::DomainResult;
use std::collections::HashMap;

#[tokio::test]
async fn test_onboarding_state_machine() -> Result<(), Box<dyn std::error::Error>> {
    // Create onboarding aggregate
    let person_id = PersonId::new();
    let mut aggregate = OnboardingAggregate::new(
        person_id,
        PersonName::new("Test", None, "User")?,
    );
    
    // Test state transitions
    assert_eq!(aggregate.current_state(), &OnboardingState::ProfileCreated);
    
    // Add email
    let events = aggregate.handle(OnboardingCommand::AddEmail {
        email: "test@example.com".to_string(),
    })?;
    assert!(!events.is_empty());
    assert_eq!(aggregate.current_state(), &OnboardingState::EmailAdded);
    
    // Verify email
    let events = aggregate.handle(OnboardingCommand::VerifyEmail {
        token: "test-token".to_string(),
    })?;
    assert!(!events.is_empty());
    assert_eq!(aggregate.current_state(), &OnboardingState::EmailVerified);
    
    // Add skills
    let events = aggregate.handle(OnboardingCommand::AddSkills {
        skills: vec!["Rust".to_string(), "Event Sourcing".to_string()],
    })?;
    assert!(!events.is_empty());
    assert_eq!(aggregate.current_state(), &OnboardingState::SkillsAdded);
    
    // Complete onboarding
    let events = aggregate.handle(OnboardingCommand::CompleteOnboarding)?;
    assert!(!events.is_empty());
    assert_eq!(aggregate.current_state(), &OnboardingState::Completed);
    
    Ok(())
}

#[test]
#[ignore = "StateMachine API needs updating"]
fn test_generic_state_machine() -> DomainResult<()> {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TestState {
        Start,
        Middle,
        End,
    }
    
    #[derive(Debug, Clone)]
    enum TestCommand {
        GoToMiddle,
        GoToEnd,
        Reset,
    }
    
    impl cim_domain_person::aggregate::state_machine::State for TestState {}
    impl cim_domain_person::aggregate::state_machine::Command for TestCommand {}
    
    // Build state machine
    let machine = StateMachine::builder(TestState::Start)
        .add_transition(
            TestState::Start,
            TestState::Middle,
            |_state, _cmd| Ok(()),
        )
        .add_transition(
            TestState::Middle,
            TestState::End,
            |_state, _cmd| Ok(()),
        )
        .add_transition(
            TestState::End,
            TestState::Start,
            |_state, _cmd| Ok(()),
        )
        .on_enter(TestState::End, |_state| {
            println!("Entered End state");
            Ok(())
        })
        .on_exit(TestState::Start, |_state| {
            println!("Exiting Start state");
            Ok(())
        })
        .build();
    
    // Test transitions
    assert_eq!(machine.current_state(), &TestState::Start);
    
    machine.handle_command(&TestCommand::GoToMiddle)?;
    assert_eq!(machine.current_state(), &TestState::Middle);
    
    machine.handle_command(&TestCommand::GoToEnd)?;
    assert_eq!(machine.current_state(), &TestState::End);
    
    machine.handle_command(&TestCommand::Reset)?;
    assert_eq!(machine.current_state(), &TestState::Start);
    
    // Test invalid transition
    let result = machine.handle_command(&TestCommand::GoToEnd);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
#[ignore = "StateMachine API needs updating"]
fn test_state_machine_guards() -> DomainResult<()> {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum GuardedState {
        Locked,
        Unlocked,
    }
    
    #[derive(Debug, Clone)]
    struct UnlockCommand {
        code: String,
    }
    
    impl cim_domain_person::aggregate::state_machine::State for GuardedState {}
    impl cim_domain_person::aggregate::state_machine::Command for UnlockCommand {}
    
    let machine = StateMachine::builder(GuardedState::Locked)
        .add_transition_with_guard(
            GuardedState::Locked,
            GuardedState::Unlocked,
            |state, cmd: &UnlockCommand| {
                // Guard: only unlock with correct code
                if cmd.code == "1234" {
                    Ok(())
                } else {
                    Err(cim_domain::DomainError::ValidationError(
                        "Invalid unlock code".to_string()
                    ))
                }
            },
            |_state, _cmd| Ok(()),
        )
        .build();
    
    // Test with wrong code
    let result = machine.handle_command(&UnlockCommand {
        code: "wrong".to_string(),
    });
    assert!(result.is_err());
    assert_eq!(machine.current_state(), &GuardedState::Locked);
    
    // Test with correct code
    machine.handle_command(&UnlockCommand {
        code: "1234".to_string(),
    })?;
    assert_eq!(machine.current_state(), &GuardedState::Unlocked);
    
    Ok(())
}