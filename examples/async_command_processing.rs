//! Example of async command processing with streaming responses

use cim_domain_person::{
    aggregate::PersonId,
    commands::{PersonCommand, CreatePerson},
    handlers::{AsyncCommandProcessor, PersonCommandProcessor},
    infrastructure::{StreamingClient, StreamingConfig, EventStore, EventEnvelope},
    projections::PersonSearchResult,
    value_objects::PersonName,
};
use futures::StreamExt;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Demonstrating async command processing");
    
    // Set up infrastructure
    let streaming_config = StreamingConfig::default();
    let streaming_client = Arc::new(
        StreamingClient::new("nats://localhost:4222", streaming_config).await?
    );
    
    // Create mock event store (in real app, this would be the actual store)
    let event_store = Arc::new(MockEventStore::new()) as Arc<dyn EventStore>;
    
    // Create command processor
    let command_processor = PersonCommandProcessor::new(
        event_store.clone(),
        streaming_client.clone(),
    );
    
    // Example 1: Process a create command with streaming response
    let person_id = PersonId::new();
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Jane".to_string(), "Doe".to_string()),
        source: "async-example".to_string(),
    });
    
    info!("Processing create command...");
    let result = command_processor.process_command(create_cmd).await?;
    
    info!(
        "Command processed. Aggregate: {}, Version: {}, Events: {}",
        result.aggregate_id, result.version, result.events.len()
    );
    
    // Stream events as they're processed
    if let Some(mut event_stream) = result.event_stream {
        info!("Streaming events...");
        while let Some(event) = event_stream.next().await {
            info!("Received event: {}", event.event_type());
        }
    }
    
    // Example 2: Query with streaming results
    info!("Searching for persons...");
    
    // Create search results directly (simplified for demo)
    let search_results = vec![
        PersonSearchResult {
            person_id: PersonId::new(),
            name: "Jane Doe".to_string(),
            email: Some("jane@example.com".to_string()),
            employer: Some("Tech Corp".to_string()),
            role: Some("Software Engineer".to_string()),
            relevance_score: 0.95,
        },
        PersonSearchResult {
            person_id: PersonId::new(),
            name: "John Smith".to_string(),
            email: Some("john@example.com".to_string()),
            employer: Some("Data Inc".to_string()),
            role: Some("Data Scientist".to_string()),
            relevance_score: 0.85,
        },
    ];
    
    info!("Got {} search results", search_results.len());
    for result in &search_results {
        info!("Found: {} - {} at {}", 
            result.name, 
            result.role.as_deref().unwrap_or("Unknown"),
            result.employer.as_deref().unwrap_or("Unknown")
        );
    }
    
    // Example 3: Streaming updates simulation
    info!("Setting up update stream for person {}...", person_id);
    
    // Create a channel for simulating updates
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(10);
    
    // Spawn a task to send updates
    tokio::spawn(async move {
        for i in 0..3 {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            let _ = tx.send(format!("Update {}: Component added", i)).await;
        }
    });
    
    // Process a few updates
    info!("Processing updates...");
    for _ in 0..3 {
        if let Some(update) = rx.recv().await {
            info!("Received: {}", update);
        }
    }
    
    info!("Demo completed!");
    Ok(())
}

// Mock implementations for the example

use async_trait::async_trait;
use cim_domain::DomainResult;

struct MockEventStore;

impl MockEventStore {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventStore for MockEventStore {
    async fn append_events(
        &self,
        _aggregate_id: PersonId,
        _events: Vec<cim_domain_person::events::PersonEvent>,
        _expected_version: Option<u64>,
    ) -> DomainResult<()> {
        Ok(())
    }
    
    async fn get_events(
        &self,
        _aggregate_id: PersonId,
    ) -> DomainResult<Vec<EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn get_events_from_version(
        &self,
        _aggregate_id: PersonId,
        _from_version: u64,
    ) -> DomainResult<Vec<EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn get_current_version(&self, _aggregate_id: PersonId) -> DomainResult<u64> {
        Ok(0)
    }
}