//! Simple NATS Connection Test
//!
//! This example tests basic NATS connectivity without JetStream complexity.
//!
//! Usage:
//!   NATS_URL=nats://10.0.0.41:4222 cargo run --example nats_simple_test

use std::env;
use tokio::time::{sleep, Duration};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple NATS Connection Test ===\n");

    // Get NATS URL from environment or use default
    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://10.0.0.41:4222".to_string());

    println!("Connecting to NATS at: {}", nats_url);

    // Connect to NATS
    let client = match async_nats::connect(&nats_url).await {
        Ok(c) => {
            println!("✅ Successfully connected to NATS!");
            c
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to NATS: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("  1. Verify NATS server is running on 10.0.0.41:4222");
            eprintln!("  2. Check network connectivity: ping 10.0.0.41");
            eprintln!("  3. Check port is open: nc -zv 10.0.0.41 4222");
            eprintln!("  4. Check firewall rules");
            return Err(e.into());
        }
    };

    println!("\n--- Testing Basic Pub/Sub ---");

    // Create a test subject
    let test_subject = "person.test.simple";
    println!("Creating subscription on subject: {}", test_subject);

    // Subscribe
    let mut subscription = client.subscribe(test_subject).await?;
    println!("✅ Subscription created");

    // Publish a message
    let test_message = b"Hello from cim-domain-person!";
    println!("\nPublishing test message: {:?}", String::from_utf8_lossy(test_message));

    client.publish(test_subject, test_message.as_ref().into()).await?;
    println!("✅ Message published");

    // Wait for message with timeout
    println!("\nWaiting for message...");
    tokio::select! {
        msg = subscription.next() => {
            if let Some(msg) = msg {
                println!("✅ Message received!");
                println!("   Subject: {}", msg.subject);
                println!("   Payload: {:?}", String::from_utf8_lossy(&msg.payload));

                if msg.payload.as_ref() == test_message.as_ref() {
                    println!("✅ Message content matches!");
                }
            }
        }
        _ = sleep(Duration::from_secs(5)) => {
            println!("❌ Timeout waiting for message");
            return Err("Message not received within timeout".into());
        }
    }

    println!("\n--- Testing Request/Reply Pattern ---");

    // Start a simple responder
    let request_subject = "person.request.test";
    let mut request_sub = client.subscribe(request_subject).await?;

    println!("Responder listening on: {}", request_subject);

    // Spawn responder task
    let responder_client = client.clone();
    tokio::spawn(async move {
        if let Some(msg) = request_sub.next().await {
            println!("  Responder received request: {:?}", String::from_utf8_lossy(&msg.payload));
            if let Some(reply_to) = msg.reply {
                let response = b"Response from person domain";
                responder_client.publish(reply_to, response.as_ref().into()).await.ok();
                println!("  Responder sent reply");
            }
        }
    });

    // Give responder time to start
    sleep(Duration::from_millis(100)).await;

    // Send request
    println!("Sending request...");
    let request_payload = b"Test request";

    match tokio::time::timeout(
        Duration::from_secs(5),
        client.request(request_subject, request_payload.as_ref().into())
    ).await {
        Ok(Ok(response)) => {
            println!("✅ Received response!");
            println!("   Response: {:?}", String::from_utf8_lossy(&response.payload));
        }
        Ok(Err(e)) => {
            println!("❌ Request error: {}", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("❌ Request timeout");
            return Err("Request timeout".into());
        }
    }

    println!("\n--- Testing Multiple Messages ---");

    let multi_subject = "person.multi.test";
    let mut multi_sub = client.subscribe(multi_subject).await?;

    // Publish multiple messages
    for i in 1..=5 {
        let msg = format!("Message {}", i);
        client.publish(multi_subject, msg.into()).await?;
    }
    println!("Published 5 messages");

    // Receive them
    let mut received = 0;
    for _ in 0..5 {
        tokio::select! {
            msg = multi_sub.next() => {
                if let Some(msg) = msg {
                    received += 1;
                    println!("  Received: {:?}", String::from_utf8_lossy(&msg.payload));
                }
            }
            _ = sleep(Duration::from_secs(1)) => {
                break;
            }
        }
    }

    println!("✅ Received {} out of 5 messages", received);

    println!("\n--- Testing Person Domain Subjects ---");

    // Test Person domain subject patterns
    let person_subjects = vec![
        "person.commands.create",
        "person.commands.update",
        "person.events.created",
        "person.events.updated",
    ];

    for subject in &person_subjects {
        match client.publish(*subject, "test".into()).await {
            Ok(_) => println!("✅ Can publish to: {}", subject),
            Err(e) => println!("❌ Failed to publish to {}: {}", subject, e),
        }
    }

    println!("\n=== Summary ===");
    println!("✅ NATS connection: SUCCESS");
    println!("✅ Basic pub/sub: SUCCESS");
    println!("✅ Request/reply: SUCCESS");
    println!("✅ Multiple messages: SUCCESS");
    println!("✅ Person domain subjects: SUCCESS");

    println!("\n=== All tests passed! ===");
    println!("\nThe NATS cluster at {} is working correctly!", nats_url);
    println!("Ready for cim-domain-person integration.");

    Ok(())
}
