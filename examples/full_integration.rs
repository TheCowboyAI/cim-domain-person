//! Full integration example demonstrating the pure event-driven architecture

use cim_domain_person::{
    // Core types
    aggregate::{PersonId, person_onboarding::OnboardingAggregate},
    commands::{PersonCommand, CreatePerson, AddComponent},
    events::{PersonEventV2, EventMetadata, create_event_registry},
    value_objects::PersonName,
    
    // Infrastructure
    infrastructure::{
        InMemoryEventStore, InMemorySnapshotStore, InMemoryComponentStore,
        streaming::StreamingConfig,
    },
    handlers::AsyncCommandProcessor,
    policies::{create_default_policy_engine, PolicyEngine},
    
    // Components
    aggregate::ComponentType,
};
use std::sync::Arc;
use futures::StreamExt;
use tracing::{info, debug};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("=== Pure Event-Driven Architecture Demo ===\n");
    
    // Setup infrastructure
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    
    // Create async command processor
    let processor = Arc::new(AsyncCommandProcessor::new(
        event_store.clone(),
        snapshot_store.clone(),
        component_store.clone(),
    ));
    
    // Create policy engine
    let policy_engine = create_default_policy_engine();
    
    // Create event registry for versioning
    let event_registry = create_event_registry();
    
    // Example 1: Basic person creation with policies
    demo_person_creation(&processor, &policy_engine).await?;
    
    // Example 2: Onboarding workflow with state machine
    demo_onboarding_workflow().await?;
    
    // Example 3: Event versioning and migration
    demo_event_versioning(&event_registry)?;
    
    // Example 4: Concurrent processing
    demo_concurrent_processing(processor.clone()).await?;
    
    // Example 5: Full integration flow
    demo_full_flow(processor, policy_engine, event_store).await?;
    
    Ok(())
}

async fn demo_person_creation(
    processor: &AsyncCommandProcessor,
    policy_engine: &PolicyEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Demo 1: Person Creation with Policies ---");
    
    // Create person
    let person_id = PersonId::new();
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Alice", Some("M".to_string()), "Johnson")?,
        source: "demo".to_string(),
    });
    
    info!("Creating person...");
    let result = processor.process(create_cmd).await?;
    info!("Person created with {} initial events", result.events.len());
    
    // Apply policies to generated events
    for event in &result.events {
        debug!("Processing event: {:?}", event);
        let policy_commands = policy_engine.evaluate(event).await;
        info!("Policies generated {} commands", policy_commands.len());
        
        // Execute policy commands
        for cmd in policy_commands {
            match &cmd {
                PersonCommand::AddComponent(add) => {
                    info!("Policy adding component: {}", add.data["type"]);
                }
                _ => {}
            }
            processor.process(cmd).await?;
        }
    }
    
    Ok(())
}

async fn demo_onboarding_workflow() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_person::aggregate::person_onboarding::OnboardingCommand;
    
    info!("\n--- Demo 2: Onboarding Workflow ---");
    
    let person_id = PersonId::new();
    let mut onboarding = OnboardingAggregate::new(
        person_id,
        PersonName::new("Bob", None, "Smith")?,
    );
    
    info!("Starting onboarding for Bob Smith");
    info!("Initial state: {:?}", onboarding.current_state());
    
    // Progress through onboarding
    let steps = vec![
        ("Add email", OnboardingCommand::AddEmail {
            email: "bob@example.com".to_string(),
        }),
        ("Verify email", OnboardingCommand::VerifyEmail {
            token: "verification-token".to_string(),
        }),
        ("Add skills", OnboardingCommand::AddSkills {
            skills: vec!["Rust".to_string(), "Event Sourcing".to_string()],
        }),
        ("Complete onboarding", OnboardingCommand::CompleteOnboarding),
    ];
    
    for (step_name, command) in steps {
        info!("\nStep: {}", step_name);
        let events = onboarding.handle(command)?;
        info!("Generated {} events", events.len());
        info!("New state: {:?}", onboarding.current_state());
    }
    
    Ok(())
}

fn demo_event_versioning(
    registry: &cim_domain_person::events::EventVersionRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Demo 3: Event Versioning ---");
    
    // Simulate receiving a legacy V1 event
    let v1_event = serde_json::json!({
        "version": "1.0",
        "person_id": "550e8400-e29b-41d4-a716-446655440000",
        "name": {
            "first_name": "Legacy",
            "middle_name": "L",
            "last_name": "User"
        },
        "source": "import",
        "created_at": "2020-01-01T00:00:00Z"
    });
    
    info!("Received legacy V1 event");
    debug!("V1 structure: {:?}", v1_event);
    
    // Migrate to current version
    let migrated = registry.migrate_to_current("PersonCreated", v1_event)?;
    info!("Migrated to current version (V2)");
    debug!("V2 structure: {:?}", migrated);
    
    // Verify metadata was added
    assert!(migrated["metadata"].is_object());
    info!("✓ Metadata added during migration");
    
    Ok(())
}

async fn demo_concurrent_processing(
    processor: Arc<AsyncCommandProcessor>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Demo 4: Concurrent Processing ---");
    
    use futures::future::join_all;
    
    let mut futures = vec![];
    
    // Create 10 persons concurrently
    for i in 0..10 {
        let processor = processor.clone();
        let future = async move {
            let person_id = PersonId::new();
            let cmd = PersonCommand::CreatePerson(CreatePerson {
                person_id,
                name: PersonName::new(&format!("User{}", i), None, "Concurrent")?,
                source: "concurrent-demo".to_string(),
            });
            
            processor.process(cmd).await
        };
        futures.push(future);
    }
    
    info!("Processing 10 commands concurrently...");
    let start = tokio::time::Instant::now();
    let results = join_all(futures).await;
    let elapsed = start.elapsed();
    
    let successful = results.iter().filter(|r| r.is_ok()).count();
    info!("Processed {} commands successfully in {:?}", successful, elapsed);
    
    Ok(())
}

async fn demo_full_flow(
    processor: Arc<AsyncCommandProcessor>,
    policy_engine: PolicyEngine,
    event_store: Arc<InMemoryEventStore>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Demo 5: Full Integration Flow ---");
    
    // 1. Create person
    let person_id = PersonId::new();
    info!("\n1. Creating person with ID: {}", person_id);
    
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Integration", None, "Test")?,
        source: "full-demo".to_string(),
    });
    
    let result = processor.process(create_cmd).await?;
    
    // 2. Apply policies
    info!("\n2. Applying policies to creation event");
    let mut policy_commands = vec![];
    for event in &result.events {
        let commands = policy_engine.evaluate(event).await;
        policy_commands.extend(commands);
    }
    info!("Policies generated {} commands", policy_commands.len());
    
    // 3. Execute policy commands
    for cmd in policy_commands {
        processor.process(cmd).await?;
    }
    
    // 4. Add components
    info!("\n3. Adding components");
    let components = vec![
        ("email", serde_json::json!({
            "email": "integration@test.com",
            "type": "primary",
            "verified": false
        })),
        ("skill", serde_json::json!({
            "skill_name": "Rust",
            "level": "Expert",
            "years_experience": 5
        })),
        ("git_profile", serde_json::json!({
            "username": "integration-test",
            "languages": ["rust", "python"],
            "repositories": 25
        })),
    ];
    
    for (comp_type, data) in components {
        let add_cmd = PersonCommand::AddComponent(AddComponent {
            person_id,
            component_type: ComponentType::CustomAttribute,
            data: data.clone(),
        });
        
        let result = processor.process(add_cmd).await?;
        info!("Added {} component", comp_type);
        
        // Check for streaming events
        if let Some(mut stream) = result.event_stream {
            info!("Component addition triggered streaming events:");
            while let Some(event) = stream.next().await {
                info!("  - Streamed: {:?}", event);
            }
        }
    }
    
    // 5. Query event store
    info!("\n4. Querying event store");
    let events = event_store.events_for_aggregate(&person_id).await?;
    info!("Total events for person: {}", events.len());
    
    // Count event types
    let mut event_counts = std::collections::HashMap::new();
    for event in &events {
        let event_type = match &event.event {
            PersonEventV2::Created { .. } => "Created",
            PersonEventV2::ComponentAdded { .. } => "ComponentAdded",
            PersonEventV2::ComponentUpdated { .. } => "ComponentUpdated",
            _ => "Other",
        };
        *event_counts.entry(event_type).or_insert(0) += 1;
    }
    
    info!("\nEvent type distribution:");
    for (event_type, count) in event_counts {
        info!("  {}: {}", event_type, count);
    }
    
    info!("\n✅ Full integration flow completed successfully!");
    
    Ok(())
}