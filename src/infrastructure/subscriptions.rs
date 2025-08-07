//! Streaming subscription handlers for event processing

use async_nats::jetstream;
use async_trait::async_trait;
use cim_domain::{DomainError, DomainResult};
use futures::StreamExt;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::events::{PersonEventV2, StreamingEventEnvelope};
use super::retry::{RetryHandler, FailedEvent};
use super::streaming::StreamingClient;

/// Trait for handling streaming events
#[async_trait]
pub trait StreamingEventHandler: Send + Sync {
    /// Handle a single event
    async fn handle_event(&self, envelope: StreamingEventEnvelope) -> DomainResult<()>;
    
    /// Get the handler name for logging
    fn name(&self) -> &str;
}

/// Streaming subscription manager
pub struct SubscriptionManager {
    streaming_client: Arc<StreamingClient>,
    retry_handler: Arc<RetryHandler>,
    handlers: Vec<Box<dyn StreamingEventHandler>>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new(
        streaming_client: Arc<StreamingClient>,
        retry_handler: Arc<RetryHandler>,
    ) -> Self {
        Self {
            streaming_client,
            retry_handler,
            handlers: Vec::new(),
        }
    }
    
    /// Register an event handler
    pub fn register_handler(&mut self, handler: Box<dyn StreamingEventHandler>) {
        info!("Registered handler: {}", handler.name());
        self.handlers.push(handler);
    }
    
    /// Start processing events for a consumer
    pub async fn start_consumer(&self, consumer_name: &str) -> DomainResult<()> {
        // Create consumer if it doesn't exist
        self.streaming_client.create_consumer(consumer_name).await?;
        
        // Get stream first, then consumer
        let stream = self.streaming_client
            .jetstream()
            .get_stream(&self.streaming_client.config().stream_name)
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to get stream: {e}"),
            })?;
            
        let consumer = stream
            .get_or_create_consumer(consumer_name, jetstream::consumer::pull::Config {
                durable_name: Some(consumer_name.to_string()),
                ..Default::default()
            })
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to get consumer: {e}"),
            })?;
        
        info!("Starting consumer: {}", consumer_name);
        
        // Create message stream
        let mut messages = consumer.messages()
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to create message stream: {e}"),
            })?;
        
        // Process messages
        while let Some(msg) = messages.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    error!("Error receiving message: {}", e);
                    continue;
                }
            };
            
            // Process the message
            match self.process_message(&msg, consumer_name).await {
                Ok(_) => {
                    // Acknowledge successful processing
                    if let Err(e) = msg.ack().await {
                        error!("Failed to acknowledge message: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to process message: {}", e);
                    
                    // Check if should send to DLQ based on max delivery attempts
                    let max_deliver = self.streaming_client.config().consumers
                        .get(consumer_name)
                        .map(|c| c.max_deliver as u64)
                        .unwrap_or(3);
                    
                    // Track delivery attempts using a thread-safe approach
                    use std::sync::LazyLock;
                    use std::sync::Mutex;
                    
                    static DELIVERY_ATTEMPTS: LazyLock<Mutex<std::collections::HashMap<String, u64>>> = 
                        LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));
                    
                    let msg_id = format!("{:?}", msg.subject);
                    
                    let attempts = {
                        let mut attempts_map = DELIVERY_ATTEMPTS.lock().unwrap();
                        let count = attempts_map.entry(msg_id.clone()).or_insert(0);
                        *count += 1;
                        *count
                    };
                    
                    if attempts >= max_deliver {
                        warn!("Message exceeded max delivery attempts ({}), sending to DLQ", max_deliver);
                        if let Err(dlq_err) = self.send_to_dlq(&msg, consumer_name, e).await {
                            error!("Failed to send to DLQ: {}", dlq_err);
                        }
                        // Clear the counter after sending to DLQ
                        {
                            let mut attempts_map = DELIVERY_ATTEMPTS.lock().unwrap();
                            attempts_map.remove(&msg_id);
                        }
                    } else {
                        warn!("Message delivery attempt {} of {}, will retry", attempts, max_deliver);
                        // NAK the message for redelivery
                        return Ok(());
                    }
                    
                    // Acknowledge to prevent redelivery
                    if let Err(ack_err) = msg.ack().await {
                        error!("Failed to acknowledge DLQ message: {}", ack_err);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single message
    async fn process_message(
        &self,
        msg: &jetstream::Message,
        consumer_name: &str,
    ) -> DomainResult<()> {
        // Deserialize envelope
        let envelope: StreamingEventEnvelope = serde_json::from_slice(&msg.payload)
            .map_err(|e| DomainError::SerializationError(format!("Failed to deserialize event: {}", e)))?;
        
        info!(
            "Processing event {} for aggregate {} by consumer {}",
            envelope.event_id, envelope.aggregate_id, consumer_name
        );
        
        // Process with each handler
        let mut processed = false;
        for handler in &self.handlers {
            match handler.handle_event(envelope.clone()).await {
                Ok(_) => {
                    processed = true;
                }
                Err(e) => {
                    warn!("Handler {} failed to process event: {}", handler.name(), e);
                }
            }
        }
        
        if !processed {
            return Err(DomainError::ValidationError(
                "No handler successfully processed the event".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Send failed message to dead letter queue
    async fn send_to_dlq(
        &self,
        msg: &jetstream::Message,
        consumer_name: &str,
        error: DomainError,
    ) -> DomainResult<()> {
        let envelope: StreamingEventEnvelope = serde_json::from_slice(&msg.payload)
            .map_err(|e| DomainError::SerializationError(e.to_string()))?;
        
        let failed_event = FailedEvent {
            event_id: envelope.event_id,
            original_subject: msg.subject.to_string(),
            payload: serde_json::to_value(&envelope)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?,
            failure_reason: error.to_string(),
            failure_count: 1, // TODO: Get actual delivery count from message
            first_failed_at: chrono::Utc::now(),
            last_failed_at: chrono::Utc::now(),
            failed_consumer: consumer_name.to_string(),
            metadata: envelope.event.metadata().clone(),
        };
        
        self.retry_handler.send_to_dlq(failed_event).await
    }
}

/// Example projection handler
pub struct ProjectionHandler {
    name: String,
}

impl ProjectionHandler {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl StreamingEventHandler for ProjectionHandler {
    async fn handle_event(&self, envelope: StreamingEventEnvelope) -> DomainResult<()> {
        match &envelope.event {
            PersonEventV2::Created { person_id, name, .. } => {
                info!("Projection {}: Person {} created with name {}", self.name, person_id, name);
                // Update projection
            }
            PersonEventV2::NameUpdated { person_id, new_name, .. } => {
                info!("Projection {}: Person {} name updated to {}", self.name, person_id, new_name);
                // Update projection
            }
            _ => {
                // Handle other events
            }
        }
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::streaming::EventMetadata;
    use crate::aggregate::PersonId;
    use crate::value_objects::PersonName;
    
    #[tokio::test]
    async fn test_projection_handler() {
        let handler = ProjectionHandler::new("test".to_string());
        
        let event = PersonEventV2::Created {
            person_id: PersonId::new(),
            name: PersonName::new("John".to_string(), "Doe".to_string()),
            source: "test".to_string(),
            metadata: EventMetadata::new(),
        };
        
        let envelope = StreamingEventEnvelope::new(
            PersonId::new(),
            1,
            event,
        );
        
        assert!(handler.handle_event(envelope).await.is_ok());
    }
}