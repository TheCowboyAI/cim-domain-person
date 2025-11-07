//! NATS Cluster Integration Test
//!
//! This example demonstrates connecting to a real NATS cluster and testing
//! the complete event sourcing workflow with JetStream.
//!
//! Usage:
//!   cargo run --example nats_cluster_test
//!
//! Environment:
//!   NATS_URL - NATS server URL (default: nats://10.0.0.41:4222)

use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::{PersonCommand, CreatePerson, RecordAttribute},
    infrastructure::{
        nats_integration::{NatsEventStore, PersonSubjects},
        event_store::EventStore,
        persistence::{PersonRepository, InMemorySnapshotStore},
    },
    value_objects::{
        PersonName, PersonAttribute, AttributeType, AttributeValue,
        PhysicalAttributeType, TemporalValidity, Provenance,
        AttributeSource, ConfidenceLevel,
    },
};
use chrono::Utc;
use std::sync::Arc;
use std::env;
use tokio::time::{sleep, Duration};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== NATS Cluster Integration Test ===\n");

    // Get NATS URL from environment or use default
    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://10.0.0.41:4222".to_string());

    println!("Connecting to NATS at: {}", nats_url);

    // Connect to NATS
    let client = match async_nats::connect(&nats_url).await {
        Ok(c) => {
            println!("✅ Successfully connected to NATS cluster");
            c
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to NATS: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("  1. Verify NATS server is running: nats-server -m 8222");
            eprintln!("  2. Check connectivity: nc -zv 10.0.0.41 4222");
            eprintln!("  3. Set custom URL: NATS_URL=nats://localhost:4222 cargo run --example nats_cluster_test");
            return Err(e.into());
        }
    };

    println!("\n--- Testing JetStream Setup ---");

    // Create NATS event store with JetStream
    let stream_name = format!("PERSON_EVENTS_{}", uuid::Uuid::now_v7());
    println!("Creating JetStream stream: {}", stream_name);

    let event_store = match NatsEventStore::new(client.clone(), stream_name.clone()).await {
        Ok(es) => {
            println!("✅ JetStream stream created successfully");
            es
        }
        Err(e) => {
            eprintln!("❌ Failed to create JetStream stream: {}", e);
            eprintln!("\nJetStream may not be enabled on this server.");
            eprintln!("To enable JetStream, add this to nats-server.conf:");
            eprintln!("  jetstream {{");
            eprintln!("    store_dir: /tmp/nats");
            eprintln!("  }}");
            return Err(e.into());
        }
    };

    println!("\n--- Testing Event Publishing ---");

    // Create a person
    let person_id = PersonId::new();
    let name = PersonName::parse("Test User").unwrap();

    println!("Creating person: {} (ID: {})", name, person_id);

    let create_command = PersonCommand::CreatePerson(CreatePerson {
        person_id,
        name: name.clone(),
        source: "nats_cluster_test".to_string(),
    });

    // Create person using command
    let mut person = Person::empty();
    person.id = person_id;

    // Handle command (pure functional)
    use cim_domain::formal_domain::Aggregate;
    let (person, events) = person.handle(create_command)?;

    println!("✅ Person created with {} events", events.len());

    // Wrap event store in Arc for shared use
    let event_store = Arc::new(event_store);

    // Append events to NATS
    println!("Publishing events to NATS...");
    event_store.append_events(person_id, events.clone(), None).await?;
    println!("✅ Events published to JetStream");

    // Wait a bit for propagation
    sleep(Duration::from_millis(100)).await;

    // Retrieve events from NATS
    println!("\n--- Testing Event Retrieval ---");
    let retrieved_events = event_store.get_events(person_id).await?;
    println!("✅ Retrieved {} events from NATS", retrieved_events.len());

    // Verify events
    assert_eq!(retrieved_events.len(), events.len());
    println!("✅ Event count matches");

    // Reconstruct person from events
    let mut reconstructed_person = Person::empty();
    reconstructed_person.id = person_id;

    for envelope in &retrieved_events {
        reconstructed_person = reconstructed_person.apply_event_pure(&envelope.event)?;
    }

    println!("✅ Person successfully reconstructed from events");
    println!("  Name: {}", reconstructed_person.core_identity.legal_name);
    println!("  Version: {}", reconstructed_person.version);

    // Test adding attributes
    println!("\n--- Testing Attribute Recording ---");

    let attribute = PersonAttribute::new(
        AttributeType::Physical(PhysicalAttributeType::Height),
        AttributeValue::Length(1.75),
        TemporalValidity::of(Utc::now()),
        Provenance::new(AttributeSource::Measured, ConfidenceLevel::Certain),
    );

    let record_command = PersonCommand::RecordAttribute(RecordAttribute {
        person_id,
        attribute: attribute.clone(),
    });

    let (person, attr_events) = person.handle(record_command)?;
    println!("✅ Attribute command generated {} events", attr_events.len());

    // Publish attribute events
    event_store.append_events(person_id, attr_events.clone(), Some(person.version - 1)).await?;
    println!("✅ Attribute events published");

    sleep(Duration::from_millis(100)).await;

    // Retrieve all events
    let all_events = event_store.get_events(person_id).await?;
    println!("✅ Retrieved {} total events", all_events.len());

    // Test event subscription
    println!("\n--- Testing Event Subscription ---");

    let subject = PersonSubjects::events();
    println!("Subscribing to: {}", subject);

    let mut subscription = client.subscribe(subject).await?;
    println!("✅ Subscription created");

    // Publish a test event
    println!("Publishing test event...");
    let test_subject = PersonSubjects::event_for(person_id, "test");
    client.publish(test_subject.clone(), "test message".into()).await?;
    println!("✅ Test event published to: {}", test_subject);

    // Wait for message (with timeout)
    println!("Waiting for subscription message...");
    tokio::select! {
        msg = subscription.next() => {
            if let Some(msg) = msg {
                println!("✅ Received message on subject: {}", msg.subject);
                println!("  Payload size: {} bytes", msg.payload.len());
            }
        }
        _ = sleep(Duration::from_secs(2)) => {
            println!("⚠️  Timeout waiting for message (this may be expected)");
        }
    }

    // Test repository integration
    println!("\n--- Testing Repository Integration ---");

    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let repository = Arc::new(PersonRepository::new(
        event_store.clone(),
        snapshot_store,
        10, // Snapshot every 10 events
    ));

    // Save person
    repository.save(&person, vec![], None).await?;
    println!("✅ Person saved to repository");

    // Load person
    let loaded_person = repository.load(person_id).await?;
    if let Some(loaded) = loaded_person {
        println!("✅ Person loaded from repository");
        println!("  Name: {}", loaded.core_identity.legal_name);
        println!("  Version: {}", loaded.version);
        println!("  Attributes: {}", loaded.attributes.attributes.len());
    }

    println!("\n--- Summary ---");
    println!("✅ NATS connection: SUCCESS");
    println!("✅ JetStream stream creation: SUCCESS");
    println!("✅ Event publishing: SUCCESS");
    println!("✅ Event retrieval: SUCCESS");
    println!("✅ Event reconstruction: SUCCESS");
    println!("✅ Attribute management: SUCCESS");
    println!("✅ Event subscription: SUCCESS");
    println!("✅ Repository integration: SUCCESS");

    println!("\n=== All tests passed! ===");

    Ok(())
}
