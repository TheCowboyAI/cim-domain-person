//! Example demonstrating aggregate state machines

use cim_domain_person::{
    aggregate::{
        Person, PersonId, PersonOnboarding, PersonState,
        OnboardingCommand
    },
    commands::{PersonCommand, DeactivatePerson, ReactivatePerson},
    value_objects::PersonName,
};
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("=== Person State Machine Demo ===");
    
    // Example 1: Person lifecycle state transitions
    demo_person_lifecycle()?;
    
    // Example 2: Multi-step onboarding workflow
    demo_onboarding_workflow().await?;
    
    Ok(())
}

fn demo_person_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Person Lifecycle Demo ---");
    
    let person_id = PersonId::new();
    let mut person = Person::new(
        person_id,
        PersonName::new("John".to_string(), "Smith".to_string())
    );
    
    info!("Initial state: {:?}", PersonState::from(person.lifecycle.clone()));
    
    // Deactivate person
    let deactivate_cmd = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id,
        reason: "Temporary leave".to_string(),
    });
    
    match person.handle_command(deactivate_cmd) {
        Ok(events) => {
            info!("Person deactivated successfully, {} events generated", events.len());
            info!("New state: {:?}", PersonState::from(person.lifecycle.clone()));
        }
        Err(e) => {
            info!("Failed to deactivate: {}", e);
        }
    }
    
    // Reactivate person
    let reactivate_cmd = PersonCommand::ReactivatePerson(ReactivatePerson {
        person_id,
        reason: "Returned from leave".to_string(),
    });
    
    match person.handle_command(reactivate_cmd) {
        Ok(events) => {
            info!("Person reactivated successfully, {} events generated", events.len());
            info!("New state: {:?}", PersonState::from(person.lifecycle.clone()));
        }
        Err(e) => {
            info!("Failed to reactivate: {}", e);
        }
    }
    
    // Try invalid transition (deactivate already active person)
    let invalid_cmd = PersonCommand::ReactivatePerson(ReactivatePerson {
        person_id,
        reason: "Invalid".to_string(),
    });
    
    match person.handle_command(invalid_cmd) {
        Ok(_) => {
            info!("Unexpected success");
        }
        Err(e) => {
            info!("Expected failure: {}", e);
        }
    }
    
    Ok(())
}

async fn demo_onboarding_workflow() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Onboarding Workflow Demo ---");
    
    let person_id = PersonId::new();
    let mut onboarding = PersonOnboarding::new(person_id);
    
    info!("Starting onboarding for person {}", person_id);
    info!("Initial state: {:?}", onboarding.state);
    
    // Step 1: Start onboarding
    let events = onboarding.handle_command(OnboardingCommand::StartOnboarding)?;
    info!("Step 1 - Started: {:?}, {} events", onboarding.state, events.len());
    
    // Step 2: Verify identity
    let identity_id = Uuid::new_v4();
    let events = onboarding.handle_command(OnboardingCommand::VerifyIdentity { identity_id })?;
    info!("Step 2 - Identity verified: {:?}, {} events", onboarding.state, events.len());
    
    // Step 3: Provide basic info
    let events = onboarding.handle_command(OnboardingCommand::ProvideBasicInfo {
        email: "john.smith@example.com".to_string(),
        phone: "+1-555-0123".to_string(),
    })?;
    info!("Step 3 - Basic info collected: {:?}, {} events", onboarding.state, events.len());
    
    // Step 4: Add components
    let components = vec![
        cim_domain_person::aggregate::person_onboarding::ComponentData {
            component_type: "skill".to_string(),
            data: serde_json::json!({
                "name": "Rust Programming",
                "level": "Expert"
            }),
        },
        cim_domain_person::aggregate::person_onboarding::ComponentData {
            component_type: "preference".to_string(),
            data: serde_json::json!({
                "timezone": "America/New_York",
                "language": "en-US"
            }),
        },
    ];
    
    let events = onboarding.handle_command(OnboardingCommand::AddComponents { components })?;
    info!("Step 4 - Components added: {:?}, {} events", onboarding.state, events.len());
    
    // Step 5: Assign location
    let location_id = Uuid::new_v4();
    let events = onboarding.handle_command(OnboardingCommand::AssignLocation { location_id })?;
    info!("Step 5 - Location assigned: {:?}, {} events", onboarding.state, events.len());
    
    // Step 6: Complete onboarding
    let events = onboarding.handle_command(OnboardingCommand::CompleteOnboarding)?;
    info!("Step 6 - Onboarding completed: {:?}, {} events", onboarding.state, events.len());
    
    // Show final state
    info!("\nOnboarding Summary:");
    info!("- ID: {}", onboarding.id);
    info!("- Person ID: {}", onboarding.person_id);
    info!("- State: {:?}", onboarding.state);
    info!("- Duration: {:?}", onboarding.completed_at.unwrap() - onboarding.started_at);
    info!("- Components added: {}", onboarding.components_added.len());
    
    // Demo failure path
    info!("\n--- Demonstrating Failure Path ---");
    let mut failed_onboarding = PersonOnboarding::new(PersonId::new());
    
    let _ = failed_onboarding.handle_command(OnboardingCommand::StartOnboarding)?;
    let _ = failed_onboarding.handle_command(OnboardingCommand::FailOnboarding {
        reason: "Identity verification failed".to_string(),
    })?;
    
    info!("Failed onboarding state: {:?}", failed_onboarding.state);
    
    // Try to continue after failure (should fail)
    match failed_onboarding.handle_command(OnboardingCommand::VerifyIdentity { 
        identity_id: Uuid::new_v4() 
    }) {
        Ok(_) => info!("Unexpected success"),
        Err(e) => info!("Expected failure: {}", e),
    }
    
    Ok(())
}

// Example of creating custom workflows as aggregates
mod custom_workflow {
    use cim_domain_person::aggregate::{State, Command};
    
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    enum EmploymentState {
        Applied,
        Interviewing,
        OfferExtended,
        Hired,
        Rejected,
    }
    
    impl State for EmploymentState {}
    
    #[derive(Clone, Debug)]
    enum EmploymentCommand {
        ScheduleInterview,
        ExtendOffer,
        AcceptOffer,
        RejectCandidate,
    }
    
    impl Command for EmploymentCommand {}
    
    // This demonstrates how any multi-step process can be modeled
    // as an aggregate with a state machine
}