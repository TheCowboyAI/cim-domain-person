//! Example of setting up enhanced NATS streaming for Person domain

use cim_domain_person::infrastructure::{
    StreamingConfig, StreamingClient, RetryHandler, SubscriptionManager,
    retry::RetryPolicy, subscriptions::ProjectionHandler,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Setting up Person domain with enhanced NATS streaming");
    
    // Create streaming configuration
    let streaming_config = StreamingConfig::default();
    
    // Connect to NATS with streaming
    let streaming_client = StreamingClient::new(
        "nats://localhost:4222",
        streaming_config,
    ).await?;
    
    info!("Connected to NATS with streaming enabled");
    
    // Create retry handler
    let retry_policy = RetryPolicy::default();
    let retry_handler = RetryHandler::new(
        streaming_client.client().clone(),
        streaming_client.jetstream().clone(),
        retry_policy,
        "person.dlq.>".to_string(),
    );
    
    // Create subscription manager
    let mut subscription_manager = SubscriptionManager::new(
        Arc::new(streaming_client),
        Arc::new(retry_handler),
    );
    
    // Register handlers
    subscription_manager.register_handler(
        Box::new(ProjectionHandler::new("summary".to_string()))
    );
    
    subscription_manager.register_handler(
        Box::new(ProjectionHandler::new("search".to_string()))
    );
    
    info!("Starting consumer for person projections");
    
    // Start consuming events
    subscription_manager.start_consumer("person-projection-summary").await?;
    
    Ok(())
}

// Example of publishing events with the new system
#[allow(dead_code)]
async fn publish_example() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_person::events::{PersonEventV2, StreamingEventEnvelope, EventMetadata};
    use cim_domain_person::aggregate::PersonId;
    use cim_domain_person::value_objects::PersonName;
    
    let streaming_config = StreamingConfig::default();
    let streaming_client = StreamingClient::new(
        "nats://localhost:4222",
        streaming_config,
    ).await?;
    
    // Create an event
    let person_id = PersonId::new();
    let event = PersonEventV2::Created {
        person_id,
        name: PersonName::new("Jane", Some("A".to_string()), "Smith")?,
        source: "example".to_string(),
        metadata: EventMetadata::new(),
    };
    
    // Wrap in envelope
    let envelope = StreamingEventEnvelope::new(person_id, 1, event);
    
    // Serialize and publish
    let payload = serde_json::to_vec(&envelope)?;
    let subject = envelope.subject();
    
    streaming_client.jetstream()
        .publish(subject, payload.into())
        .await?;
    
    info!("Published event for person {}", person_id);
    
    Ok(())
}