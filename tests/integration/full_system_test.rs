//! Full system integration test for the pure event-driven architecture

use cim_domain_person::{
    aggregate::{PersonId, ComponentType},
    commands::{PersonCommand, CreatePerson, AddComponent, UpdateComponent},
    events::{PersonEventV2, EventMetadata, create_event_registry},
    handlers::AsyncCommandProcessor,
    infrastructure::{
        InMemoryEventStore, InMemorySnapshotStore, InMemoryComponentStore,
        streaming::{StreamingConfig, ConsumerType},
    },
    policies::{create_default_policy_engine, PolicyEngine},
    value_objects::PersonName,
};
use std::sync::Arc;
use futures::StreamExt;
use std::collections::HashMap;

#[tokio::test]
async fn test_full_system_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize infrastructure
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    
    // Create processors and engines
    let processor = Arc::new(AsyncCommandProcessor::new(
        event_store.clone(),
        snapshot_store.clone(),
        component_store.clone(),
    ));
    let policy_engine = create_default_policy_engine();
    let event_registry = create_event_registry();
    
    // Track all generated events
    let mut all_events = Vec::new();
    let mut policy_commands = Vec::new();
    
    // Step 1: Create a person
    let person_id = PersonId::new();
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Integration", Some("Test".to_string()), "User")?,
        source: "test-suite".to_string(),
    });
    
    let result = processor.process(create_cmd).await?;
    assert_eq!(result.aggregate_id, person_id);
    assert!(!result.events.is_empty());
    all_events.extend(result.events.clone());
    
    // Apply policies to creation event
    for event in &result.events {
        let commands = policy_engine.evaluate(event).await;
        policy_commands.extend(commands);
    }
    
    // Step 2: Execute policy-generated commands
    for cmd in &policy_commands {
        let result = processor.process(cmd.clone()).await?;
        all_events.extend(result.events);
    }
    
    // Step 3: Add various components
    let components_to_add = vec![
        ("email", serde_json::json!({
            "email": "Integration.User@EXAMPLE.com  ", // Intentionally malformed
            "type": "primary",
            "verified": false
        })),
        ("phone", serde_json::json!({
            "number": "+1-555-123-4567",
            "type": "mobile",
            "verified": true
        })),
        ("skill", serde_json::json!({
            "skill_name": "Rust",
            "level": "Expert",
            "years_experience": 5
        })),
        ("git_profile", serde_json::json!({
            "username": "integration-test",
            "languages": ["rust", "python", "go"],
            "repositories": 42,
            "contributions": 1337
        })),
    ];
    
    for (comp_type, data) in components_to_add {
        let add_cmd = PersonCommand::AddComponent(AddComponent {
            person_id,
            component_type: ComponentType::CustomAttribute,
            data: data.clone(),
        });
        
        let result = processor.process(add_cmd).await?;
        all_events.extend(result.events.clone());
        
        // Apply policies to component events
        for event in &result.events {
            let commands = policy_engine.evaluate(event).await;
            for cmd in commands {
                let result = processor.process(cmd).await?;
                all_events.extend(result.events);
            }
        }
    }
    
    // Step 4: Verify data quality policy normalized email
    let email_updates = all_events.iter().filter_map(|e| {
        match e {
            PersonEventV2::ComponentUpdated { updates, .. } => {
                updates.get("email").and_then(|v| v.as_str())
            },
            _ => None
        }
    }).collect::<Vec<_>>();
    
    assert!(email_updates.iter().any(|&email| email == "integration.user@example.com"));
    
    // Step 5: Verify skill recommendations were generated
    let skill_additions = all_events.iter().filter(|e| {
        matches!(e, PersonEventV2::ComponentAdded { component_data, .. } 
            if component_data["type"] == "skill")
    }).count();
    
    assert!(skill_additions > 1); // Original skill + recommendations
    
    // Step 6: Test event versioning
    let v1_event = serde_json::json!({
        "version": "1.0",
        "person_id": person_id.to_string(),
        "name": {
            "first_name": "Legacy",
            "middle_name": "V1",
            "last_name": "Event"
        },
        "source": "legacy_system",
        "created_at": "2020-01-01T00:00:00Z"
    });
    
    let migrated = event_registry.migrate_to_current("PersonCreated", v1_event)?;
    assert_eq!(migrated["version"], "2.0");
    assert!(migrated["metadata"].is_object());
    
    // Step 7: Verify event store integrity
    let stored_events = event_store.events_for_aggregate(&person_id).await?;
    assert_eq!(stored_events.len(), all_events.len());
    
    // Count event types
    let mut event_type_counts = HashMap::new();
    for event in &stored_events {
        let event_type = match &event.event {
            PersonEventV2::Created { .. } => "Created",
            PersonEventV2::ComponentAdded { .. } => "ComponentAdded",
            PersonEventV2::ComponentUpdated { .. } => "ComponentUpdated",
            _ => "Other",
        };
        *event_type_counts.entry(event_type).or_insert(0) += 1;
    }
    
    // Verify we have the expected event types
    assert!(event_type_counts.contains_key("Created"));
    assert!(event_type_counts.contains_key("ComponentAdded"));
    assert!(event_type_counts.contains_key("ComponentUpdated")); // From data quality policy
    
    // Step 8: Test streaming results
    let large_update_cmd = PersonCommand::AddComponent(AddComponent {
        person_id,
        component_type: ComponentType::CustomAttribute,
        data: serde_json::json!({
            "type": "bulk_data",
            "items": (0..100).map(|i| format!("item-{}", i)).collect::<Vec<_>>()
        }),
    });
    
    let result = processor.process(large_update_cmd).await?;
    if let Some(mut stream) = result.event_stream {
        let mut streamed_count = 0;
        while let Some(_event) = stream.next().await {
            streamed_count += 1;
        }
        assert!(streamed_count > 0);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_person_operations() -> Result<(), Box<dyn std::error::Error>> {
    use futures::future::join_all;
    
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    
    let processor = Arc::new(AsyncCommandProcessor::new(
        event_store.clone(),
        snapshot_store,
        component_store,
    ));
    
    // Create multiple persons concurrently
    let mut create_futures = vec![];
    let mut person_ids = vec![];
    
    for i in 0..20 {
        let person_id = PersonId::new();
        person_ids.push(person_id);
        
        let processor = processor.clone();
        let future = async move {
            let cmd = PersonCommand::CreatePerson(CreatePerson {
                person_id,
                name: PersonName::new(&format!("Concurrent{}", i), None, "Test")?,
                source: "concurrent-test".to_string(),
            });
            processor.process(cmd).await
        };
        create_futures.push(future);
    }
    
    // Execute all creates concurrently
    let results = join_all(create_futures).await;
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Now add components to all persons concurrently
    let mut component_futures = vec![];
    
    for (i, person_id) in person_ids.iter().enumerate() {
        let processor = processor.clone();
        let person_id = *person_id;
        
        let future = async move {
            let cmd = PersonCommand::AddComponent(AddComponent {
                person_id,
                component_type: ComponentType::Email,
                data: serde_json::json!({
                    "email": format!("user{}@test.com", i),
                    "type": "primary"
                }),
            });
            processor.process(cmd).await
        };
        component_futures.push(future);
    }
    
    let results = join_all(component_futures).await;
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Verify all persons have their events
    for person_id in &person_ids {
        let events = event_store.events_for_aggregate(person_id).await?;
        assert!(events.len() >= 2); // At least Created + ComponentAdded
    }
    
    Ok(())
}

#[tokio::test]
async fn test_state_machine_workflow_integration() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_person::aggregate::person_onboarding::{
        OnboardingAggregate, OnboardingCommand, OnboardingState
    };
    
    let person_id = PersonId::new();
    let mut onboarding = OnboardingAggregate::new(
        person_id,
        PersonName::new("Workflow", None, "Test")?,
    );
    
    // Run through complete workflow
    let workflow_steps = vec![
        OnboardingCommand::AddEmail {
            email: "workflow@test.com".to_string(),
        },
        OnboardingCommand::VerifyEmail {
            token: "test-verification-token".to_string(),
        },
        OnboardingCommand::AddSkills {
            skills: vec!["Rust".to_string(), "CQRS".to_string(), "Event Sourcing".to_string()],
        },
        OnboardingCommand::CompleteOnboarding,
    ];
    
    let mut all_events = Vec::new();
    
    for command in workflow_steps {
        let events = onboarding.handle(command)?;
        all_events.extend(events);
    }
    
    // Verify final state
    assert_eq!(onboarding.current_state(), &OnboardingState::Completed);
    
    // Verify events were generated for each step
    assert!(all_events.len() >= 4);
    
    // Verify events contain expected data
    let has_email_event = all_events.iter().any(|e| {
        matches!(e, PersonEventV2::ComponentAdded { component_data, .. } 
            if component_data.get("email").is_some())
    });
    assert!(has_email_event);
    
    let skill_events: Vec<_> = all_events.iter().filter(|e| {
        matches!(e, PersonEventV2::ComponentAdded { component_data, .. } 
            if component_data.get("skill_name").is_some())
    }).collect();
    assert_eq!(skill_events.len(), 3); // Three skills added
    
    Ok(())
}