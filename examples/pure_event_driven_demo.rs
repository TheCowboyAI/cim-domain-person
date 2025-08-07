//! Demonstration of the pure event-driven architecture
//! 
//! This example shows how all the components work together in the new architecture:
//! - Async command processing with streaming
//! - State machine aggregates
//! - Event versioning
//! - Policy engine
//! - NATS-style event flow

use cim_domain_person::{
    aggregate::{PersonId, ComponentType, person_onboarding::*},
    commands::{PersonCommand, CreatePerson, AddComponent},
    events::{PersonEventV2, EventMetadata, create_event_registry},
    handlers::AsyncCommandProcessor,
    infrastructure::{
        InMemoryEventStore, InMemorySnapshotStore, InMemoryComponentStore,
        streaming::StreamingEventEnvelope,
    },
    policies::{PolicyEngine, Policy, create_default_policy_engine},
    value_objects::PersonName,
};
use std::sync::Arc;
use futures::StreamExt;
use tracing::{info, debug};
use async_trait::async_trait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("🚀 Pure Event-Driven Architecture Demo");
    info!("=====================================\n");
    
    // Setup infrastructure
    let infrastructure = setup_infrastructure().await?;
    
    // Demo 1: Show the complete event flow
    demo_event_flow(&infrastructure).await?;
    
    // Demo 2: State machine workflows
    demo_state_machine_workflow().await?;
    
    // Demo 3: Event versioning in action
    demo_event_versioning().await?;
    
    // Demo 4: Policy engine with custom policies
    demo_policy_engine(&infrastructure).await?;
    
    // Demo 5: Streaming and concurrency
    demo_streaming_concurrency(&infrastructure).await?;
    
    info!("\n✅ Demo completed successfully!");
    
    Ok(())
}

struct Infrastructure {
    processor: Arc<AsyncCommandProcessor>,
    policy_engine: PolicyEngine,
    event_store: Arc<InMemoryEventStore>,
}

async fn setup_infrastructure() -> Result<Infrastructure, Box<dyn std::error::Error>> {
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    
    let processor = Arc::new(AsyncCommandProcessor::new(
        event_store.clone(),
        snapshot_store,
        component_store,
    ));
    
    let policy_engine = create_default_policy_engine();
    
    Ok(Infrastructure {
        processor,
        policy_engine,
        event_store,
    })
}

async fn demo_event_flow(infra: &Infrastructure) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n📊 Demo 1: Complete Event Flow");
    info!("==============================");
    
    // Create person
    let person_id = PersonId::new();
    info!("Creating person with ID: {}", person_id);
    
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Event", Some("Flow".to_string()), "Demo")?,
        source: "demo".to_string(),
    });
    
    // Process command
    info!("Processing CreatePerson command...");
    let result = infra.processor.process(create_cmd).await?;
    info!("✓ Generated {} events", result.events.len());
    
    // Apply policies
    info!("\nApplying policies to events...");
    for event in &result.events {
        let commands = infra.policy_engine.evaluate(event).await;
        info!("✓ Policy generated {} commands", commands.len());
        
        for cmd in commands {
            match &cmd {
                PersonCommand::AddComponent(add) => {
                    info!("  → Adding component: {}", add.data["type"]);
                }
                _ => {}
            }
            infra.processor.process(cmd).await?;
        }
    }
    
    // Show event store contents
    let events = infra.event_store.events_for_aggregate(&person_id).await?;
    info!("\nEvent store now contains {} events for this person", events.len());
    
    Ok(())
}

async fn demo_state_machine_workflow() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n🔄 Demo 2: State Machine Workflow");
    info!("=================================");
    
    let person_id = PersonId::new();
    let mut onboarding = OnboardingAggregate::new(
        person_id,
        PersonName::new("State", Some("Machine".to_string()), "User")?,
    );
    
    info!("Starting onboarding workflow...");
    info!("Initial state: {:?}", onboarding.current_state());
    
    // Progress through states
    let steps = vec![
        ("Adding email", OnboardingCommand::AddEmail {
            email: "state.machine@example.com".to_string(),
        }),
        ("Verifying email", OnboardingCommand::VerifyEmail {
            token: "demo-token-123".to_string(),
        }),
        ("Adding skills", OnboardingCommand::AddSkills {
            skills: vec![
                "Event Sourcing".to_string(),
                "CQRS".to_string(),
                "Domain Driven Design".to_string(),
            ],
        }),
        ("Completing onboarding", OnboardingCommand::CompleteOnboarding),
    ];
    
    for (description, command) in steps {
        info!("\n{}", description);
        let events = onboarding.handle(command)?;
        info!("✓ Generated {} events", events.len());
        info!("✓ New state: {:?}", onboarding.current_state());
    }
    
    info!("\n✅ Workflow completed!");
    
    Ok(())
}

async fn demo_event_versioning() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n📦 Demo 3: Event Versioning");
    info!("===========================");
    
    let registry = create_event_registry();
    
    // Simulate legacy V1 event
    info!("Simulating legacy V1 event...");
    let v1_event = serde_json::json!({
        "version": "1.0",
        "person_id": "123e4567-e89b-12d3-a456-426614174000",
        "name": {
            "first_name": "Legacy",
            "middle_name": "Version",
            "last_name": "One"
        },
        "source": "legacy_system",
        "created_at": "2020-01-01T00:00:00Z"
    });
    
    debug!("V1 structure: {}", serde_json::to_string_pretty(&v1_event)?);
    
    // Migrate to current version
    info!("\nMigrating to current version (V2)...");
    let migrated = registry.migrate_to_current("PersonCreated", v1_event)?;
    
    info!("✓ Migration successful!");
    debug!("V2 structure: {}", serde_json::to_string_pretty(&migrated)?);
    
    // Show what changed
    info!("\nKey changes:");
    info!("  • Added 'metadata' object");
    info!("  • Moved 'created_at' → 'metadata.timestamp'");
    info!("  • Added correlation_id and other tracking fields");
    
    Ok(())
}

async fn demo_policy_engine(infra: &Infrastructure) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n🎯 Demo 4: Policy Engine");
    info!("========================");
    
    // Create custom demo policy
    struct DemoPolicy;
    
    #[async_trait]
    impl Policy for DemoPolicy {
        async fn evaluate(&self, event: &PersonEventV2) -> cim_domain::DomainResult<Vec<PersonCommand>> {
            match event {
                PersonEventV2::ComponentAdded { person_id, component_data, .. } => {
                    if component_data["type"] == "git_profile" {
                        info!("  🎯 DemoPolicy: Detected Git profile, adding badge!");
                        Ok(vec![
                            PersonCommand::AddComponent(AddComponent {
                                person_id: *person_id,
                                component_type: ComponentType::CustomAttribute,
                                data: serde_json::json!({
                                    "type": "badge",
                                    "name": "Open Source Contributor",
                                    "level": "gold"
                                }),
                            })
                        ])
                    } else {
                        Ok(vec![])
                    }
                }
                _ => Ok(vec![])
            }
        }
        
        fn name(&self) -> &str {
            "DemoPolicy"
        }
    }
    
    // Create engine with custom policy
    let mut custom_engine = PolicyEngine::new();
    custom_engine.register(Arc::new(DemoPolicy));
    
    // Also add default policies
    for policy in infra.policy_engine.policies() {
        custom_engine.register(policy.clone());
    }
    
    // Create person and add Git profile
    let person_id = PersonId::new();
    info!("Creating person and adding Git profile...");
    
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Policy", None, "Demo")?,
        source: "demo".to_string(),
    });
    
    infra.processor.process(create_cmd).await?;
    
    let git_cmd = PersonCommand::AddComponent(AddComponent {
        person_id,
        component_type: ComponentType::CustomAttribute,
        data: serde_json::json!({
            "type": "git_profile",
            "username": "rustacean",
            "languages": ["rust", "go", "python"],
            "repositories": 42
        }),
    });
    
    let result = infra.processor.process(git_cmd).await?;
    
    // Apply custom policies
    info!("\nApplying policies...");
    for event in &result.events {
        let commands = custom_engine.evaluate(event).await;
        for cmd in commands {
            infra.processor.process(cmd).await?;
        }
    }
    
    info!("✓ Custom policy executed successfully!");
    
    Ok(())
}

async fn demo_streaming_concurrency(infra: &Infrastructure) -> Result<(), Box<dyn std::error::Error>> {
    info!("\n⚡ Demo 5: Streaming & Concurrency");
    info!("==================================");
    
    use futures::future::join_all;
    use tokio::time::{Duration, Instant};
    
    // Create multiple persons concurrently
    info!("Creating 50 persons concurrently...");
    let start = Instant::now();
    
    let mut futures = vec![];
    for i in 0..50 {
        let processor = infra.processor.clone();
        let future = async move {
            let cmd = PersonCommand::CreatePerson(CreatePerson {
                person_id: PersonId::new(),
                name: PersonName::new(&format!("User{}", i), None, "Concurrent")?,
                source: "concurrent-demo".to_string(),
            });
            processor.process(cmd).await
        };
        futures.push(future);
    }
    
    let results = join_all(futures).await;
    let elapsed = start.elapsed();
    
    let successful = results.iter().filter(|r| r.is_ok()).count();
    info!("✓ Created {} persons in {:?}", successful, elapsed);
    info!("  Average: {:?} per person", elapsed / successful as u32);
    
    // Demonstrate streaming
    info!("\nDemonstrating event streaming...");
    let person_id = PersonId::new();
    
    // Create person first
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Stream", None, "Demo")?,
        source: "streaming-demo".to_string(),
    });
    infra.processor.process(create_cmd).await?;
    
    // Add bulk data that triggers streaming
    let bulk_cmd = PersonCommand::AddComponent(AddComponent {
        person_id,
        component_type: ComponentType::CustomAttribute,
        data: serde_json::json!({
            "type": "bulk_import",
            "records": (0..20).map(|i| {
                serde_json::json!({
                    "id": i,
                    "data": format!("Record {}", i)
                })
            }).collect::<Vec<_>>()
        }),
    });
    
    let result = infra.processor.process(bulk_cmd).await?;
    
    if let Some(mut stream) = result.event_stream {
        info!("📡 Receiving streamed events:");
        let mut count = 0;
        while let Some(event) = stream.next().await {
            count += 1;
            debug!("  Received event #{}", count);
        }
        info!("✓ Streamed {} additional events", count);
    }
    
    Ok(())
}