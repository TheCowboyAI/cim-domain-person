#!/usr/bin/env rust-script
//! Verification script for event migration
//!
//! Usage: cargo run --bin verify_migration -- [options]
//!
//! Options:
//!   --source <URL>      NATS server URL (default: nats://localhost:4222)
//!   --stream <NAME>     JetStream stream name (default: person-events)
//!   --sample-size <N>   Number of events to sample (default: 100)

use cim_domain_person::events::{PersonEventV2, EventMetadata};
use async_nats::jetstream;
use clap::{Arg, Command};
use futures::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let matches = Command::new("verify_migration")
        .version("1.0")
        .about("Verifies event migration was successful")
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
        .arg(
            Arg::new("sample-size")
                .long("sample-size")
                .value_name("N")
                .help("Number of events to sample")
                .default_value("100")
                .value_parser(clap::value_parser!(usize))
        )
        .get_matches();
    
    let nats_url = matches.get_one::<String>("source").unwrap();
    let stream_name = matches.get_one::<String>("stream").unwrap();
    let sample_size = *matches.get_one::<usize>("sample-size").unwrap();
    
    info!("Verifying event migration");
    info!("NATS URL: {}", nats_url);
    info!("Stream: {}", stream_name);
    info!("Sample size: {}", sample_size);
    
    // Connect to NATS
    let client = async_nats::connect(nats_url).await?;
    let jetstream = jetstream::new(client);
    
    // Get stream info
    let stream = jetstream.get_stream(stream_name).await?;
    let info = stream.info().await?;
    info!("Stream has {} messages", info.state.messages);
    
    // Create consumer for sampling
    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("verify-consumer".to_string()),
            deliver_policy: jetstream::consumer::DeliverPolicy::Last,
            ack_policy: jetstream::consumer::AckPolicy::None,
            ..Default::default()
        })
        .await?;
    
    // Verification statistics
    let mut version_counts: HashMap<String, usize> = HashMap::new();
    let mut event_type_counts: HashMap<String, usize> = HashMap::new();
    let mut validation_errors = 0;
    let mut parse_errors = 0;
    
    // Sample events
    let mut messages = consumer.messages().await?;
    let mut count = 0;
    
    while let Some(msg) = messages.next().await {
        if count >= sample_size {
            break;
        }
        
        let msg = msg?;
        count += 1;
        
        // Parse event
        let event: Value = match serde_json::from_slice(&msg.payload) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to parse event: {}", e);
                parse_errors += 1;
                continue;
            }
        };
        
        // Check version
        let version = event.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        *version_counts.entry(version.to_string()).or_insert(0) += 1;
        
        // Try to parse as V2 event
        match serde_json::from_value::<PersonEventV2>(event.clone()) {
            Ok(_) => {
                // Valid V2 event
                if let Some(event_type) = detect_v2_event_type(&event) {
                    *event_type_counts.entry(event_type).or_insert(0) += 1;
                }
            }
            Err(e) => {
                warn!("Event failed V2 validation: {}", e);
                validation_errors += 1;
            }
        }
    }
    
    // Print results
    info!("\nVerification Results:");
    info!("Total events sampled: {}", count);
    info!("Parse errors: {}", parse_errors);
    info!("Validation errors: {}", validation_errors);
    
    info!("\nVersion distribution:");
    for (version, count) in &version_counts {
        let percentage = (*count as f64 / count as f64) * 100.0;
        info!("  v{}: {} ({:.1}%)", version, count, percentage);
    }
    
    info!("\nEvent type distribution:");
    for (event_type, type_count) in &event_type_counts {
        let percentage = (*type_count as f64 / count as f64) * 100.0;
        info!("  {}: {} ({:.1}%)", event_type, type_count, percentage);
    }
    
    // Check if migration is complete
    let v2_count = version_counts.get("2.0").unwrap_or(&0);
    let migration_complete = *v2_count == count && validation_errors == 0;
    
    if migration_complete {
        info!("\n✅ Migration verification PASSED");
        info!("All sampled events are at version 2.0 and valid");
    } else {
        warn!("\n❌ Migration verification FAILED");
        if let Some(v1_count) = version_counts.get("1.0") {
            warn!("Found {} v1.0 events that need migration", v1_count);
        }
        if validation_errors > 0 {
            warn!("Found {} events that fail V2 validation", validation_errors);
        }
    }
    
    Ok(())
}

fn detect_v2_event_type(event: &Value) -> Option<String> {
    // Check for metadata field (V2 indicator)
    if event.get("metadata").is_none() {
        return None;
    }
    
    // Detect based on fields
    if event.get("name").is_some() && event.get("source").is_some() {
        return Some("Created".to_string());
    }
    
    if event.get("old_name").is_some() && event.get("new_name").is_some() {
        return Some("NameUpdated".to_string());
    }
    
    if event.get("birth_date").is_some() {
        return Some("BirthDateSet".to_string());
    }
    
    if event.get("component_type").is_some() {
        if event.get("component_data").is_some() {
            return Some("ComponentAdded".to_string());
        }
        if event.get("updates").is_some() {
            return Some("ComponentUpdated".to_string());
        }
    }
    
    if event.get("status").is_some() {
        if event["status"] == "suspended" {
            return Some("Suspended".to_string());
        }
        if event["status"] == "archived" {
            return Some("Archived".to_string());
        }
    }
    
    None
}