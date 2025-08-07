//! Integration tests for NATS streaming functionality

use cim_domain_person::{
    infrastructure::{
        streaming::{StreamingConfig, ConsumerType, StreamingSubscriber},
        EventMetadata,
    },
    events::{PersonEventV2, PersonCreatedV2},
    aggregate::PersonId,
    value_objects::PersonName,
};
use async_nats::jetstream;
use futures::StreamExt;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
#[ignore = "requires NATS server"]
async fn test_streaming_subscription() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = jetstream::new(client);
    
    // Configure stream
    let config = StreamingConfig::new("test-person-events")
        .with_consumer("test-consumer", ConsumerType::Durable)
        .with_max_age(Duration::from_secs(3600));
    
    // Ensure stream exists
    config.ensure_stream(&jetstream).await?;
    
    // Create subscriber
    let mut subscriber = StreamingSubscriber::new(jetstream.clone(), config.clone()).await?;
    
    // Publish test event
    let event = PersonEventV2::Created {
        person_id: PersonId::new(),
        name: PersonName::new("Test", None, "User")?,
        source: "test".to_string(),
        metadata: EventMetadata::new(),
    };
    
    let subject = "person.events.created";
    let payload = serde_json::to_vec(&event)?;
    jetstream.publish(subject, payload.into()).await?;
    
    // Subscribe and receive
    let mut stream = subscriber.subscribe("test-consumer").await?;
    
    let received = timeout(Duration::from_secs(5), async {
        stream.next().await
    }).await?;
    
    assert!(received.is_some());
    let msg = received.unwrap();
    msg.ack().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_dead_letter_queue() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_person::infrastructure::retry::{RetryHandler, RetryPolicy};
    
    // Connect to NATS
    let client = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = jetstream::new(client.clone());
    
    // Create retry handler
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        multiplier: 2.0,
    };
    
    let handler = RetryHandler::new(
        client,
        jetstream,
        policy,
        "person.events.dlq".to_string(),
    );
    
    // Test retry with failing operation
    let result = handler.retry(|| async {
        Err::<(), _>(cim_domain::DomainError::ValidationError("test error".to_string()))
    }).await;
    
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_event_streaming() -> Result<(), Box<dyn std::error::Error>> {
    use cim_domain_person::infrastructure::streaming::StreamingEventEnvelope;
    
    // Create streaming envelope
    let event = PersonCreatedV2 {
        person_id: PersonId::new(),
        name: PersonName::new("Stream", None, "Test")?,
        source: "test".to_string(),
        metadata: EventMetadata::new(),
    };
    
    let envelope = StreamingEventEnvelope {
        stream_id: event.person_id.to_string(),
        sequence: 1,
        event: PersonEventV2::from(event),
        metadata: EventMetadata::new(),
    };
    
    // Serialize/deserialize
    let json = serde_json::to_string(&envelope)?;
    let decoded: StreamingEventEnvelope = serde_json::from_str(&json)?;
    
    assert_eq!(envelope.stream_id, decoded.stream_id);
    assert_eq!(envelope.sequence, decoded.sequence);
    
    Ok(())
}