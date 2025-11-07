//! Person Domain Service
//!
//! A long-running service that handles Person domain commands via NATS.
//! This service:
//! - Connects to NATS and JetStream
//! - Subscribes to person.commands.> subject
//! - Processes Person commands
//! - Publishes events to person.events.>
//! - Maintains event store via JetStream
//!
//! Environment Variables:
//!   NATS_URL         - NATS server URL (default: nats://localhost:4222)
//!   STREAM_NAME      - JetStream stream name (default: PERSON_EVENTS)
//!   LOG_LEVEL        - Logging level (default: info)
//!   SNAPSHOT_FREQ    - Snapshot frequency in events (default: 100)
//!
//! Usage:
//!   cargo run --bin person-service
//!   NATS_URL=nats://10.0.0.41:4222 cargo run --bin person-service

use cim_domain_person::{
    infrastructure::{
        nats_integration::{NatsEventStore, PersonCommandHandler},
        persistence::{PersonRepository, InMemorySnapshotStore},
    },
};
use std::sync::Arc;
use std::env;
use tracing::{info, error};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let _log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("Starting Person Domain Service");

    // Get configuration from environment
    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let stream_name = env::var("STREAM_NAME")
        .unwrap_or_else(|_| "PERSON_EVENTS".to_string());
    let snapshot_frequency: u64 = env::var("SNAPSHOT_FREQ")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100);

    info!("Configuration:");
    info!("  NATS URL: {}", nats_url);
    info!("  Stream Name: {}", stream_name);
    info!("  Snapshot Frequency: {} events", snapshot_frequency);

    // Connect to NATS
    info!("Connecting to NATS...");
    let client = match async_nats::connect(&nats_url).await {
        Ok(c) => {
            info!("✓ Connected to NATS at {}", nats_url);
            c
        }
        Err(e) => {
            error!("✗ Failed to connect to NATS: {}", e);
            return Err(e.into());
        }
    };

    // Create event store with JetStream
    info!("Setting up JetStream event store...");
    let event_store = match NatsEventStore::new(client.clone(), stream_name.clone()).await {
        Ok(es) => {
            info!("✓ JetStream event store ready (stream: {})", stream_name);
            Arc::new(es)
        }
        Err(e) => {
            error!("✗ Failed to create JetStream event store: {}", e);
            error!("  Make sure JetStream is enabled on the NATS server");
            return Err(e.into());
        }
    };

    // Create snapshot store
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    info!("✓ Snapshot store initialized");

    // Create repository
    let repository = Arc::new(PersonRepository::new(
        event_store,
        snapshot_store,
        snapshot_frequency,
    ));
    info!("✓ Person repository ready (snapshot every {} events)", snapshot_frequency);

    // Create command handler
    let handler = PersonCommandHandler::new(repository, client.clone());
    info!("✓ Command handler initialized");

    // Start listening for commands
    info!("Starting command listener on subject: person.commands.>");
    info!("Person Domain Service is ready to handle commands");
    info!("Press Ctrl+C to shutdown gracefully");

    // Spawn command handler
    let handler_task = tokio::spawn(async move {
        match handler.start().await {
            Ok(_) => {
                info!("Command handler stopped normally");
            }
            Err(e) => {
                error!("Command handler error: {}", e);
            }
        }
    });

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, stopping service...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    // Graceful shutdown
    info!("Initiating graceful shutdown...");

    // Cancel handler task
    handler_task.abort();

    // Wait a bit for cleanup
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    info!("Person Domain Service stopped");
    Ok(())
}
