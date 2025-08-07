#!/usr/bin/env rust-script
//! Migration script for upgrading events to the latest version
//!
//! Usage: cargo run --bin migrate_events -- [options]
//!
//! Options:
//!   --dry-run           Show what would be migrated without making changes
//!   --batch-size <N>    Process events in batches of N (default: 1000)
//!   --source <URL>      NATS server URL (default: nats://localhost:4222)
//!   --stream <NAME>     JetStream stream name (default: person-events)

use cim_domain_person::{
    events::{create_event_registry, PersonEventV2, VersionedEventEnvelope},
    infrastructure::NatsEventStore,
};
use async_nats::jetstream;
use clap::{Arg, Command};
use futures::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let matches = Command::new("migrate_events")
        .version("1.0")
        .about("Migrates person events to the latest version")
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show what would be migrated without making changes")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("batch-size")
                .long("batch-size")
                .value_name("N")
                .help("Process events in batches of N")
                .default_value("1000")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("source")
                .long("source")
                .value_name("URL")
                .help("NATS server URL")
                .default_value("nats://localhost:4222")
        )
        .arg(
            Arg::new("stream")
                .long("stream")
                .value_name("NAME")
                .help("JetStream stream name")
                .default_value("person-events")
        )
        .get_matches();
    
    let dry_run = matches.get_flag("dry-run");
    let batch_size = *matches.get_one::<usize>("batch-size").unwrap();
    let nats_url = matches.get_one::<String>("source").unwrap();
    let stream_name = matches.get_one::<String>("stream").unwrap();
    
    info!("Starting event migration");
    info!("NATS URL: {}", nats_url);
    info!("Stream: {}", stream_name);
    info!("Batch size: {}", batch_size);
    info!("Dry run: {}", dry_run);
    
    // Connect to NATS
    let client = async_nats::connect(nats_url).await?;
    let jetstream = jetstream::new(client);
    
    // Create event registry
    let registry = create_event_registry();
    
    // Get stream info
    let stream = jetstream.get_stream(stream_name).await?;
    let info = stream.info().await?;
    info!("Stream has {} messages", info.state.messages);
    
    // Create consumer for reading all messages
    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("migration-consumer".to_string()),
            deliver_policy: jetstream::consumer::DeliverPolicy::All,
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ..Default::default()
        })
        .await?;
    
    // Migration statistics
    let mut stats = MigrationStats::default();
    
    // Process messages in batches
    let mut messages = consumer.messages().await?;
    let mut batch = Vec::new();
    
    while let Some(msg) = messages.next().await {
        let msg = msg?;
        batch.push(msg);
        
        if batch.len() >= batch_size {
            process_batch(&registry, &jetstream, &mut batch, &mut stats, dry_run).await?;
            batch.clear();
        }
    }
    
    // Process remaining messages
    if !batch.is_empty() {
        process_batch(&registry, &jetstream, &mut batch, &mut stats, dry_run).await?;
    }
    
    // Print statistics
    info!("\nMigration complete!");
    info!("Total events processed: {}", stats.total);
    info!("Events migrated: {}", stats.migrated);
    info!("Events already current: {}", stats.already_current);
    info!("Errors: {}", stats.errors);
    
    if !stats.migrations_by_type.is_empty() {
        info!("\nMigrations by event type:");
        for (event_type, count) in &stats.migrations_by_type {
            info!("  {}: {}", event_type, count);
        }
    }
    
    Ok(())
}

#[derive(Default)]
struct MigrationStats {
    total: usize,
    migrated: usize,
    already_current: usize,
    errors: usize,
    migrations_by_type: HashMap<String, usize>,
}

async fn process_batch(
    registry: &cim_domain_person::events::EventVersionRegistry,
    jetstream: &jetstream::Context,
    batch: &mut Vec<jetstream::Message>,
    stats: &mut MigrationStats,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    for msg in batch.drain(..) {
        stats.total += 1;
        
        // Parse event
        let event: Value = match serde_json::from_slice(&msg.payload) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to parse event: {}", e);
                stats.errors += 1;
                msg.ack().await?;
                continue;
            }
        };
        
        // Check version
        let version = event.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0");
        
        let event_type = detect_event_type(&event);
        
        // Check if migration is needed
        let current_version = registry.current_version(&event_type)
            .unwrap_or("2.0");
        
        if version == current_version {
            stats.already_current += 1;
            msg.ack().await?;
            continue;
        }
        
        // Migrate event
        match registry.migrate_to_current(&event_type, event.clone()) {
            Ok(migrated) => {
                stats.migrated += 1;
                *stats.migrations_by_type.entry(event_type.clone()).or_insert(0) += 1;
                
                if !dry_run {
                    // Publish migrated event
                    let subject = format!("person.events.{}", event_type.to_lowercase());
                    let payload = serde_json::to_vec(&migrated)?;
                    jetstream.publish(subject, payload.into()).await?;
                }
                
                info!(
                    "Migrated {} event from v{} to v{}",
                    event_type, version, current_version
                );
            }
            Err(e) => {
                error!("Failed to migrate event: {}", e);
                stats.errors += 1;
            }
        }
        
        msg.ack().await?;
    }
    
    Ok(())
}

fn detect_event_type(event: &Value) -> String {
    // Try to detect event type from structure
    if event.get("person_id").is_some() && event.get("name").is_some() {
        if event.get("created_at").is_some() || event.get("source").is_some() {
            return "PersonCreated".to_string();
        }
        if event.get("updated_at").is_some() {
            return "PersonUpdated".to_string();
        }
    }
    
    if event.get("old_name").is_some() && event.get("new_name").is_some() {
        return "NameUpdated".to_string();
    }
    
    if event.get("birth_date").is_some() {
        return "BirthDateSet".to_string();
    }
    
    if event.get("component_type").is_some() {
        if event.get("registered_at").is_some() {
            return "ComponentRegistered".to_string();
        }
        if event.get("unregistered_at").is_some() {
            return "ComponentUnregistered".to_string();
        }
    }
    
    // Default
    "Unknown".to_string()
}