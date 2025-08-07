//! Enhanced NATS streaming configuration for pure event-driven architecture

use async_nats::{jetstream, Client};
use async_nats::jetstream::consumer::{pull::Config as PullConfig, DeliverPolicy, AckPolicy};
use async_nats::jetstream::stream::{Config as StreamConfig, RetentionPolicy, StorageType, DiscardPolicy};
use cim_domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Enhanced streaming configuration for Person domain
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Stream name
    pub stream_name: String,
    /// Consumer configurations by name
    pub consumers: HashMap<String, ConsumerConfig>,
    /// Retention policy
    pub retention_policy: RetentionPolicy,
    /// Maximum age of events
    pub max_age: Duration,
    /// Maximum number of messages
    pub max_messages: i64,
    /// Maximum bytes
    pub max_bytes: i64,
    /// Dead letter queue configuration
    pub dead_letter_config: Option<DeadLetterConfig>,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            stream_name: "PERSON_EVENTS".to_string(),
            consumers: Self::default_consumers(),
            retention_policy: RetentionPolicy::Limits,
            max_age: Duration::from_secs(365 * 24 * 60 * 60), // 1 year
            max_messages: 10_000_000,
            max_bytes: 1024 * 1024 * 1024 * 10, // 10GB
            dead_letter_config: Some(DeadLetterConfig::default()),
        }
    }
}

impl StreamingConfig {
    /// Get default consumer configurations
    fn default_consumers() -> HashMap<String, ConsumerConfig> {
        let mut consumers = HashMap::new();
        
        // Projection consumers
        consumers.insert("person-projection-summary".to_string(), ConsumerConfig {
            durable_name: "person-projection-summary".to_string(),
            filter_subjects: vec!["person.events.>".to_string()],
            deliver_policy: DeliverPolicy::All,
            ack_policy: AckPolicy::Explicit,
            max_deliver: 3,
            ack_wait: Duration::from_secs(30),
            description: Some("Updates person summary projections".to_string()),
        });
        
        consumers.insert("person-projection-search".to_string(), ConsumerConfig {
            durable_name: "person-projection-search".to_string(),
            filter_subjects: vec!["person.events.>".to_string()],
            deliver_policy: DeliverPolicy::All,
            ack_policy: AckPolicy::Explicit,
            max_deliver: 3,
            ack_wait: Duration::from_secs(30),
            description: Some("Updates search index".to_string()),
        });
        
        // Cross-domain consumers
        consumers.insert("person-to-location".to_string(), ConsumerConfig {
            durable_name: "person-to-location".to_string(),
            filter_subjects: vec!["person.events.*.component_registered".to_string()],
            deliver_policy: DeliverPolicy::New,
            ack_policy: AckPolicy::Explicit,
            max_deliver: 5,
            ack_wait: Duration::from_secs(60),
            description: Some("Syncs person-location relationships".to_string()),
        });
        
        // Policy engine consumer
        consumers.insert("person-policy-engine".to_string(), ConsumerConfig {
            durable_name: "person-policy-engine".to_string(),
            filter_subjects: vec!["person.events.>".to_string()],
            deliver_policy: DeliverPolicy::New,
            ack_policy: AckPolicy::Explicit,
            max_deliver: 3,
            ack_wait: Duration::from_secs(30),
            description: Some("Applies business policies to events".to_string()),
        });
        
        consumers
    }
    
    /// Create stream configuration
    pub fn stream_config(&self) -> StreamConfig {
        let mut config = StreamConfig {
            name: self.stream_name.clone(),
            subjects: vec![
                "person.events.>".to_string(),
                "person.commands.>".to_string(),
            ],
            retention: self.retention_policy,
            storage: StorageType::File,
            max_age: self.max_age,
            max_messages: self.max_messages,
            max_bytes: self.max_bytes,
            discard: DiscardPolicy::Old,
            duplicate_window: Duration::from_secs(120), // 2 minute deduplication
            ..Default::default()
        };
        
        // Add mirrors for cross-region replication if needed
        config.description = Some("Person domain event stream with enhanced features".to_string());
        
        config
    }
}

/// Consumer configuration
#[derive(Debug, Clone)]
pub struct ConsumerConfig {
    pub durable_name: String,
    pub filter_subjects: Vec<String>,
    pub deliver_policy: DeliverPolicy,
    pub ack_policy: AckPolicy,
    pub max_deliver: i64,
    pub ack_wait: Duration,
    pub description: Option<String>,
}

/// Dead letter queue configuration
#[derive(Debug, Clone)]
pub struct DeadLetterConfig {
    pub stream_name: String,
    pub max_delivers: i64,
}

impl Default for DeadLetterConfig {
    fn default() -> Self {
        Self {
            stream_name: "PERSON_EVENTS_DLQ".to_string(),
            max_delivers: 3,
        }
    }
}

/// Enhanced NATS streaming client
pub struct StreamingClient {
    client: Client,
    jetstream: jetstream::Context,
    config: StreamingConfig,
}

impl StreamingClient {
    /// Create new streaming client
    pub async fn new(nats_url: &str, config: StreamingConfig) -> DomainResult<Self> {
        let client = async_nats::connect(nats_url).await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS".to_string(),
                message: format!("Failed to connect: {e}"),
            })?;
        
        let jetstream = jetstream::new(client.clone());
        
        // Create or update main stream
        let stream_config = config.stream_config();
        jetstream.create_stream(stream_config).await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to create stream: {e}"),
            })?;
        
        // Create dead letter queue stream if configured
        if let Some(dlq_config) = &config.dead_letter_config {
            let dlq_stream_config = StreamConfig {
                name: dlq_config.stream_name.clone(),
                subjects: vec!["person.dlq.>".to_string()],
                retention: RetentionPolicy::Limits,
                storage: StorageType::File,
                max_age: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
                description: Some("Dead letter queue for failed person events".to_string()),
                ..Default::default()
            };
            
            jetstream.create_stream(dlq_stream_config).await
                .map_err(|e| DomainError::ExternalServiceError {
                    service: "NATS JetStream".to_string(),
                    message: format!("Failed to create DLQ stream: {e}"),
                })?;
        }
        
        Ok(Self {
            client,
            jetstream,
            config,
        })
    }
    
    /// Create a consumer with the given name
    pub async fn create_consumer(&self, consumer_name: &str) -> DomainResult<()> {
        let consumer_config = self.config.consumers.get(consumer_name)
            .ok_or_else(|| DomainError::ValidationError(
                format!("Unknown consumer: {}", consumer_name)
            ))?;
        
        let config = PullConfig {
            durable_name: Some(consumer_config.durable_name.clone()),
            filter_subjects: consumer_config.filter_subjects.clone(),
            deliver_policy: consumer_config.deliver_policy,
            ack_policy: consumer_config.ack_policy,
            max_deliver: consumer_config.max_deliver,
            ack_wait: consumer_config.ack_wait,
            description: consumer_config.description.clone(),
            ..Default::default()
        };
        
        self.jetstream
            .create_consumer_on_stream(config, &self.config.stream_name)
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to create consumer: {e}"),
            })?;
        
        Ok(())
    }
    
    /// Get the underlying client
    pub fn client(&self) -> &Client {
        &self.client
    }
    
    /// Get the JetStream context
    pub fn jetstream(&self) -> &jetstream::Context {
        &self.jetstream
    }
    
    /// Get the configuration
    pub fn config(&self) -> &StreamingConfig {
        &self.config
    }
}

/// Event metadata for enhanced event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Event version
    pub version: String,
    /// Correlation ID for tracking related events
    pub correlation_id: uuid::Uuid,
    /// Causation ID linking to the command that caused this event
    pub causation_id: Option<uuid::Uuid>,
    /// Timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Actor who initiated the action
    pub actor: Option<String>,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl EventMetadata {
    /// Create new metadata with defaults
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            correlation_id: uuid::Uuid::new_v4(),
            causation_id: None,
            timestamp: chrono::Utc::now(),
            actor: None,
            context: HashMap::new(),
        }
    }
    
    /// Create metadata with correlation
    pub fn with_correlation(correlation_id: uuid::Uuid) -> Self {
        Self {
            correlation_id,
            ..Self::new()
        }
    }
    
    /// Create metadata from a command
    pub fn from_command(command_id: uuid::Uuid) -> Self {
        Self {
            causation_id: Some(command_id),
            ..Self::new()
        }
    }
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            multiplier: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_streaming_config() {
        let config = StreamingConfig::default();
        assert_eq!(config.stream_name, "PERSON_EVENTS");
        assert!(!config.consumers.is_empty());
        assert!(config.dead_letter_config.is_some());
    }
    
    #[test]
    fn test_event_metadata() {
        let metadata = EventMetadata::new();
        assert_eq!(metadata.version, "1.0");
        assert!(metadata.causation_id.is_none());
        
        let command_id = uuid::Uuid::new_v4();
        let metadata = EventMetadata::from_command(command_id);
        assert_eq!(metadata.causation_id, Some(command_id));
    }
}