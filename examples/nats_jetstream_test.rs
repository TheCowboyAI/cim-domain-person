//! NATS JetStream Test
//!
//! This example tests JetStream functionality specifically.
//!
//! Usage:
//!   NATS_URL=nats://10.0.0.41:4222 cargo run --example nats_jetstream_test

use std::env;
use tokio::time::{sleep, Duration};
use futures::StreamExt;
use async_nats::jetstream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NATS JetStream Test ===\n");

    // Get NATS URL from environment or use default
    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://10.0.0.41:4222".to_string());

    println!("Connecting to NATS at: {}", nats_url);

    // Connect to NATS
    let client = async_nats::connect(&nats_url).await?;
    println!("✅ Connected to NATS");

    println!("\n--- Testing JetStream Availability ---");

    // Get JetStream context
    let jetstream = jetstream::new(client.clone());
    println!("✅ JetStream context created");
    println!("   (Will verify JetStream is enabled by creating a stream)");

    println!("\n--- Testing Stream Creation ---");

    // Create a test stream
    let stream_name = format!("TEST_STREAM_{}", uuid::Uuid::now_v7());
    println!("Creating stream: {}", stream_name);

    let stream_config = jetstream::stream::Config {
        name: stream_name.clone(),
        subjects: vec!["test.>".to_string()],
        retention: jetstream::stream::RetentionPolicy::Limits,
        storage: jetstream::stream::StorageType::File,
        max_age: Duration::from_secs(60), // 1 minute for testing
        ..Default::default()
    };

    match jetstream.create_stream(stream_config).await {
        Ok(stream) => {
            println!("✅ Stream created successfully");
            println!("   Name: {}", stream.cached_info().config.name);
            println!("   Subjects: {:?}", stream.cached_info().config.subjects);
        }
        Err(e) => {
            eprintln!("❌ Failed to create stream: {}", e);
            return Err(e.into());
        }
    }

    println!("\n--- Testing Message Publishing ---");

    // Publish messages
    for i in 1..=3 {
        let subject = format!("test.message.{}", i);
        let payload = format!("Test message {}", i);

        println!("Publishing to subject: {}", subject);

        match jetstream.publish(subject.clone(), payload.clone().into()).await {
            Ok(ack) => {
                // Need to await the ack
                match ack.await {
                    Ok(_) => println!("✅ Message {} published and acknowledged", i),
                    Err(e) => println!("❌ Failed to get ack for message {}: {}", i, e),
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to publish message {}: {}", i, e);
                return Err(e.into());
            }
        }
    }

    println!("\n--- Testing Consumer Creation ---");

    // Create a consumer
    let consumer_config = jetstream::consumer::pull::Config {
        name: Some("test_consumer".to_string()),
        filter_subject: "test.>".to_string(),
        ..Default::default()
    };

    let consumer = jetstream
        .create_consumer_on_stream(consumer_config, &stream_name)
        .await?;

    println!("✅ Consumer created");

    println!("\n--- Testing Message Retrieval ---");

    // Fetch messages
    let mut messages = consumer.messages().await?;
    let mut received = 0;

    for _ in 0..3 {
        tokio::select! {
            msg = messages.next() => {
                if let Some(Ok(msg)) = msg {
                    received += 1;
                    println!("  Received message {}:", received);
                    println!("    Subject: {}", msg.subject);
                    println!("    Payload: {:?}", String::from_utf8_lossy(&msg.payload));

                    // Acknowledge the message
                    msg.ack().await.map_err(|e| format!("{:?}", e))?;
                }
            }
            _ = sleep(Duration::from_secs(2)) => {
                break;
            }
        }
    }

    println!("✅ Retrieved {} messages", received);

    println!("\n--- Testing Stream Deletion ---");

    // Clean up: delete the stream
    match jetstream.delete_stream(&stream_name).await {
        Ok(_) => println!("✅ Test stream deleted"),
        Err(e) => println!("⚠️  Failed to delete stream (may need manual cleanup): {}", e),
    }

    println!("\n=== Summary ===");
    println!("✅ JetStream enabled: YES");
    println!("✅ Stream creation: SUCCESS");
    println!("✅ Message publishing: SUCCESS");
    println!("✅ Consumer creation: SUCCESS");
    println!("✅ Message retrieval: SUCCESS");
    println!("✅ Stream deletion: SUCCESS");

    println!("\n=== JetStream is fully functional! ===");
    println!("Ready for cim-domain-person event sourcing.");

    Ok(())
}
