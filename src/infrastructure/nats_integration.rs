//! NATS integration for Person domain

use async_nats::{Client, jetstream};
use async_trait::async_trait;
use cim_domain::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::StreamExt;

use crate::aggregate::{Person, PersonId};
use crate::events::PersonEvent;
use crate::commands::PersonCommand;
use super::event_store::{EventStore, EventEnvelope};

/// NATS subject patterns for Person domain
pub struct PersonSubjects;

impl PersonSubjects {
    /// Commands subject pattern
    pub fn commands() -> &'static str {
        "person.commands.>"
    }
    
    /// Events subject pattern
    pub fn events() -> &'static str {
        "person.events.>"
    }
    
    /// Command subject for specific aggregate
    pub fn command_for(aggregate_id: PersonId) -> String {
        format!("person.commands.{aggregate_id}")
    }
    
    /// Event subject for specific aggregate
    pub fn event_for(aggregate_id: PersonId, event_type: &str) -> String {
        format!("person.events.{aggregate_id}.{event_type}")
    }
}

/// NATS-based event store implementation
pub struct NatsEventStore {
    _client: Client,
    jetstream: jetstream::Context,
    stream_name: String,
}

impl NatsEventStore {
    /// Create a new NATS event store
    pub async fn new(client: Client, stream_name: String) -> DomainResult<Self> {
        let jetstream = jetstream::new(client.clone());

        // Create or update the stream
        let stream_config = jetstream::stream::Config {
            name: stream_name.clone(),
            subjects: vec![PersonSubjects::events().to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            storage: jetstream::stream::StorageType::File,
            max_age: std::time::Duration::from_secs(365 * 24 * 60 * 60),
            ..Default::default()
        };

        // Try to get existing stream first, create if it doesn't exist
        match jetstream.get_stream(&stream_name).await {
            Ok(_stream) => {
                // Stream already exists, use it
            }
            Err(_) => {
                // Stream doesn't exist, try to create it
                // If creation fails due to overlapping subjects, try to get it anyway
                match jetstream.create_stream(stream_config).await {
                    Ok(_) => {},
                    Err(e) => {
                        // Check if error is due to overlapping subjects
                        let error_str = format!("{:?}", e);
                        if error_str.contains("10065") || error_str.contains("overlap") {
                            // Stream exists with same subjects, try to get it
                            jetstream.get_stream(&stream_name).await
                                .map_err(|e2| DomainError::ExternalServiceError {
                                    service: "NATS JetStream".to_string(),
                                    message: format!("Stream with overlapping subjects exists but cannot access it: {e2}"),
                                })?;
                        } else {
                            return Err(DomainError::ExternalServiceError {
                                service: "NATS JetStream".to_string(),
                                message: format!("Failed to create stream: {e}"),
                            });
                        }
                    }
                }
            }
        }

        Ok(Self {
            _client: client,
            jetstream,
            stream_name,
        })
    }
}

#[async_trait]
impl EventStore for NatsEventStore {
    async fn append_events(
        &self,
        aggregate_id: PersonId,
        events: Vec<PersonEvent>,
        expected_version: Option<u64>,
    ) -> DomainResult<()> {
        // Check expected version if provided
        if let Some(expected) = expected_version {
            let current = self.get_current_version(aggregate_id).await?;
            if current != expected {
                return Err(DomainError::ConcurrencyConflict {
                    expected,
                    actual: current,
                });
            }
        }
        
        // Publish each event
        for (index, event) in events.into_iter().enumerate() {
            let event_type = match &event {
                PersonEvent::PersonCreated(_) => "created",
                PersonEvent::PersonUpdated(_) => "updated",
                PersonEvent::NameUpdated(_) => "name_updated",
                PersonEvent::BirthDateSet(_) => "birth_date_set",
                PersonEvent::DeathRecorded(_) => "death_recorded",
                PersonEvent::PersonDeactivated(_) => "deactivated",
                PersonEvent::PersonReactivated(_) => "reactivated",
                PersonEvent::PersonMergedInto(_) => "merged",
                PersonEvent::AttributeRecorded(_) => "attribute_recorded",
                PersonEvent::AttributeUpdated(_) => "attribute_updated",
                PersonEvent::AttributeInvalidated(_) => "attribute_invalidated",
            };
            
            let subject = PersonSubjects::event_for(aggregate_id, event_type);
            let sequence = self.get_current_version(aggregate_id).await? + index as u64 + 1;
            
            let envelope = EventEnvelope {
                aggregate_id,
                sequence,
                event,
                timestamp: chrono::Utc::now(),
                correlation_id: uuid::Uuid::now_v7().to_string(),
                causation_id: uuid::Uuid::now_v7().to_string(),
            };
            
            let payload = serde_json::to_vec(&envelope)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?;
            
            self.jetstream.publish(subject, payload.into()).await
                .map_err(|e| DomainError::ExternalServiceError {
                    service: "NATS JetStream".to_string(),
                    message: format!("Failed to publish event: {e}"),
                })?;
        }
        
        Ok(())
    }
    
    async fn get_events(&self, aggregate_id: PersonId) -> DomainResult<Vec<EventEnvelope>> {
        self.get_events_from_version(aggregate_id, 0).await
    }
    
    async fn get_events_from_version(
        &self,
        aggregate_id: PersonId,
        from_version: u64,
    ) -> DomainResult<Vec<EventEnvelope>> {
        let subject_filter = format!("person.events.{aggregate_id}.>");
        
        let consumer_config = jetstream::consumer::pull::Config {
            filter_subject: subject_filter,
            ..Default::default()
        };
        
        let consumer = self.jetstream
            .create_consumer_on_stream(consumer_config, self.stream_name.as_str())
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to create consumer: {e}"),
            })?;
        
        let mut events = Vec::new();
        let mut messages = consumer.messages().await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to get messages: {e}"),
            })?;
        
        while let Some(msg) = messages.next().await {
            let msg = msg.map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to get message: {e}"),
            })?;
            
            let envelope: EventEnvelope = serde_json::from_slice(&msg.payload)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?;
            
            if envelope.sequence >= from_version {
                events.push(envelope);
            }
            
            msg.ack().await
                .map_err(|e| DomainError::ExternalServiceError {
                    service: "NATS JetStream".to_string(),
                    message: format!("Failed to ack message: {e}"),
                })?;
        }
        
        events.sort_by_key(|e| e.sequence);
        Ok(events)
    }
    
    async fn get_current_version(&self, aggregate_id: PersonId) -> DomainResult<u64> {
        let events = self.get_events(aggregate_id).await?;
        Ok(events.len() as u64)
    }
}

/// Command handler service
pub struct PersonCommandHandler {
    repository: Arc<super::persistence::PersonRepository>,
    client: Client,
}

impl PersonCommandHandler {
    pub fn new(repository: Arc<super::persistence::PersonRepository>, client: Client) -> Self {
        Self { repository, client }
    }
    
    /// Start listening for commands
    pub async fn start(&self) -> DomainResult<()> {
        let mut subscription = self.client
            .subscribe(PersonSubjects::commands())
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS".to_string(),
                message: format!("Failed to subscribe: {e}"),
            })?;
        
        while let Some(msg) = subscription.next().await {
            let command: PersonCommand = serde_json::from_slice(&msg.payload)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?;
            
            // Handle command
            match self.handle_command(command).await {
                Ok(response) => {
                    if let Some(reply) = msg.reply {
                        let payload = serde_json::to_vec(&response)
                            .map_err(|e| DomainError::SerializationError(e.to_string()))?;
                        
                        self.client.publish(reply, payload.into()).await
                            .map_err(|e| DomainError::ExternalServiceError {
                                service: "NATS".to_string(),
                                message: format!("Failed to send reply: {e}"),
                            })?;
                    }
                }
                Err(e) => {
                    eprintln!("Error handling command: {e}");
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle a single command
    async fn handle_command(&self, command: PersonCommand) -> DomainResult<CommandResponse> {
        let aggregate_id = command.aggregate_id();
        
        // Load or create aggregate
        let person = match self.repository.load(aggregate_id).await? {
            Some(p) => p,
            None => {
                if matches!(command, PersonCommand::CreatePerson(_)) {
                    Person::empty()
                } else {
                    return Err(DomainError::AggregateNotFound(format!("Person {aggregate_id}")));
                }
            }
        };

        // Handle command using formal Aggregate trait (pure functional)
        use cim_domain::formal_domain::Aggregate;
        let expected_version = if person.version == 0 { None } else { Some(person.version) };
        let (person, events) = person.handle(command)?;

        // Save events
        self.repository.save(&person, events.clone(), expected_version).await?;

        Ok(CommandResponse {
            aggregate_id,
            version: person.version,
            events_generated: events.len(),
        })
    }
}

/// Response to a command
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
    pub aggregate_id: PersonId,
    pub version: u64,
    pub events_generated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subject_patterns() {
        let person_id = PersonId::new();
        
        assert_eq!(PersonSubjects::commands(), "person.commands.>");
        assert_eq!(PersonSubjects::events(), "person.events.>");
        assert_eq!(
            PersonSubjects::command_for(person_id),
            format!("person.commands.{person_id}")
        );
        assert_eq!(
            PersonSubjects::event_for(person_id, "created"),
            format!("person.events.{person_id}.created")
        );
    }
} 