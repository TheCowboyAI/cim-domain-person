//! Example of async command processing with streaming responses

use cim_domain_person::{
    aggregate::PersonId,
    commands::{PersonCommand, CreatePerson},
    handlers::{AsyncCommandProcessor, PersonCommandProcessor},
    infrastructure::{StreamingClient, StreamingConfig},
    queries::{AsyncQueryProcessor, PersonQueryProcessor, QueryResult, consume_query_result},
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
    let event_store = Arc::new(MockEventStore::new());
    
    // Create command processor
    let command_processor = PersonCommandProcessor::new(
        event_store.clone(),
        streaming_client.clone(),
    );
    
    // Example 1: Process a create command with streaming response
    let person_id = PersonId::new();
    let create_cmd = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: PersonName::new("Jane", Some("A".to_string()), "Doe")?,
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
    let query_processor = create_mock_query_processor();
    
    info!("Searching for persons...");
    let search_criteria = crate::queries::SearchCriteria {
        name_pattern: Some("Doe".to_string()),
        email: None,
        phone: None,
        skills: vec![],
        location_id: None,
        organization_id: None,
        active_only: true,
        limit: None, // No limit triggers streaming
        offset: None,
    };
    
    let search_results = query_processor.search_persons(search_criteria).await?;
    
    match search_results {
        QueryResult::Stream(mut stream) => {
            info!("Streaming search results...");
            let mut count = 0;
            while let Some(result) = stream.next().await {
                info!("Found: {} (score: {})", result.name, result.relevance_score);
                count += 1;
                if count >= 10 {
                    break; // Limit for demo
                }
            }
        }
        QueryResult::Multiple(results) => {
            info!("Got {} results", results.len());
            for result in results.iter().take(5) {
                info!("Found: {} (score: {})", result.name, result.relevance_score);
            }
        }
        _ => {}
    }
    
    // Example 3: Subscribe to person updates
    info!("Subscribing to updates for person {}...", person_id);
    let mut update_stream = query_processor.subscribe_to_updates(person_id).await?;
    
    // Spawn a task to process updates
    tokio::spawn(async move {
        while let Some(update) = update_stream.next().await {
            info!(
                "Person {} updated: {:?} at {}",
                update.person_id, update.update_type, update.timestamp
            );
        }
    });
    
    // Simulate some activity
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    info!("Demo completed!");
    Ok(())
}

// Mock implementations for the example

use async_trait::async_trait;
use cim_domain::{DomainResult, DomainError};

struct MockEventStore;

impl MockEventStore {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl crate::infrastructure::EventStore for MockEventStore {
    async fn append_events(
        &self,
        _aggregate_id: PersonId,
        _events: Vec<crate::events::PersonEvent>,
        _expected_version: Option<u64>,
    ) -> DomainResult<()> {
        Ok(())
    }
    
    async fn get_events(
        &self,
        _aggregate_id: PersonId,
    ) -> DomainResult<Vec<crate::infrastructure::EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn get_events_from_version(
        &self,
        _aggregate_id: PersonId,
        _from_version: u64,
    ) -> DomainResult<Vec<crate::infrastructure::EventEnvelope>> {
        Ok(vec![])
    }
    
    async fn get_current_version(&self, _aggregate_id: PersonId) -> DomainResult<u64> {
        Ok(0)
    }
}

fn create_mock_query_processor() -> PersonQueryProcessor {
    use crate::queries::async_query_processor::*;
    use crate::projections::PersonSummary;
    
    struct MockSummaryStore;
    
    #[async_trait]
    impl SummaryProjectionStore for MockSummaryStore {
        async fn get_summary(&self, _person_id: PersonId) -> DomainResult<Option<PersonSummary>> {
            Ok(None)
        }
    }
    
    struct MockSearchStore;
    
    #[async_trait]
    impl SearchProjectionStore for MockSearchStore {
        async fn search(&self, criteria: &SearchCriteria) -> DomainResult<Vec<PersonSearchResult>> {
            Ok(vec![
                PersonSearchResult {
                    person_id: PersonId::new(),
                    name: "Jane Doe".to_string(),
                    relevance_score: 0.95,
                    matched_fields: vec!["name".to_string()],
                }
            ])
        }
        
        async fn search_stream(&self, criteria: &SearchCriteria) 
            -> DomainResult<std::pin::Pin<Box<dyn futures::Stream<Item = PersonSearchResult> + Send>>> {
            let (tx, rx) = tokio::sync::mpsc::channel(10);
            
            tokio::spawn(async move {
                for i in 0..20 {
                    let _ = tx.send(PersonSearchResult {
                        person_id: PersonId::new(),
                        name: format!("Person {}", i),
                        relevance_score: 1.0 - (i as f32 * 0.05),
                        matched_fields: vec!["name".to_string()],
                    }).await;
                }
            });
            
            Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
        }
    }
    
    // Mock other stores...
    struct MockNetworkStore;
    struct MockTimelineStore;
    
    #[async_trait]
    impl NetworkProjectionStore for MockNetworkStore {
        async fn get_network(&self, _person_id: PersonId, _depth: usize) 
            -> DomainResult<Option<crate::projections::PersonNetworkView>> {
            Ok(None)
        }
    }
    
    #[async_trait]
    impl TimelineProjectionStore for MockTimelineStore {
        async fn get_timeline(
            &self,
            _person_id: PersonId,
            _from: Option<chrono::DateTime<chrono::Utc>>,
            _to: Option<chrono::DateTime<chrono::Utc>>,
        ) -> DomainResult<Vec<TimelineEvent>> {
            Ok(vec![])
        }
        
        async fn get_timeline_stream(
            &self,
            _person_id: PersonId,
            _from: Option<chrono::DateTime<chrono::Utc>>,
            _to: Option<chrono::DateTime<chrono::Utc>>,
        ) -> DomainResult<std::pin::Pin<Box<dyn futures::Stream<Item = TimelineEvent> + Send>>> {
            let (tx, rx) = tokio::sync::mpsc::channel(10);
            drop(tx); // Close immediately for demo
            Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
        }
    }
    
    PersonQueryProcessor::new(
        Arc::new(MockSummaryStore),
        Arc::new(MockSearchStore),
        Arc::new(MockNetworkStore),
        Arc::new(MockTimelineStore),
    )
}