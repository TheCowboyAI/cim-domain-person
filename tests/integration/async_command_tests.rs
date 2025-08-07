//! Integration tests for async command processing

use cim_domain_person::{
    handlers::AsyncCommandProcessor,
    commands::{PersonCommand, CreatePerson},
    aggregate::PersonId,
    value_objects::PersonName,
    infrastructure::{InMemoryEventStore, InMemorySnapshotStore, InMemoryComponentStore},
};
use futures::StreamExt;
use std::sync::Arc;

#[tokio::test]
async fn test_async_command_processing() -> Result<(), Box<dyn std::error::Error>> {
    // Setup infrastructure
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    
    // Create processor
    let processor = AsyncCommandProcessor::new(
        event_store.clone(),
        snapshot_store,
        component_store,
    );
    
    // Create command
    let person_id = PersonId::new();
    let command = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Async", None, "Test")?,
        source: "test".to_string(),
    });
    
    // Process command
    let result = processor.process(command).await?;
    
    assert_eq!(result.aggregate_id, person_id);
    assert_eq!(result.events.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_streaming_command_results() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_person::commands::{AddComponent, UpdateComponent};
    use cim_domain_person::aggregate::ComponentType;
    
    // Setup infrastructure
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    
    let processor = AsyncCommandProcessor::new(
        event_store.clone(),
        snapshot_store,
        component_store,
    );
    
    // Create person first
    let person_id = PersonId::new();
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Stream", None, "Test")?,
        source: "test".to_string(),
    });
    
    processor.process(create_cmd).await?;
    
    // Add multiple components
    for i in 0..5 {
        let add_cmd = PersonCommand::AddComponent(AddComponent {
            person_id,
            component_type: ComponentType::CustomAttribute,
            data: serde_json::json!({
                "type": "test",
                "index": i,
                "value": format!("test-{}", i)
            }),
        });
        
        let result = processor.process(add_cmd).await?;
        
        // Check if we have streaming results
        if let Some(mut stream) = result.event_stream {
            while let Some(event) = stream.next().await {
                println!("Streamed event: {:?}", event);
            }
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_command_processing() -> Result<(), Box<dyn std::error::Error>> {
    use futures::future::join_all;
    
    // Setup infrastructure
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    
    let processor = Arc::new(AsyncCommandProcessor::new(
        event_store.clone(),
        snapshot_store,
        component_store,
    ));
    
    // Create multiple commands for different persons
    let mut futures = vec![];
    
    for i in 0..10 {
        let processor = processor.clone();
        let future = async move {
            let person_id = PersonId::new();
            let command = PersonCommand::CreatePerson(CreatePerson {
                person_id,
                name: PersonName::new(&format!("Person{}", i), None, "Test")?,
                source: "test".to_string(),
            });
            
            processor.process(command).await
        };
        
        futures.push(future);
    }
    
    // Process all commands concurrently
    let results = join_all(futures).await;
    
    // Verify all succeeded
    for result in results {
        assert!(result.is_ok());
    }
    
    // Verify event store has all events
    let all_events = event_store.all_events().await?;
    assert_eq!(all_events.len(), 10);
    
    Ok(())
}