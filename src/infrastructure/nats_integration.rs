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
        format!("person.commands.{}", aggregate_id)
    }
    
    /// Event subject for specific aggregate
    pub fn event_for(aggregate_id: PersonId, event_type: &str) -> String {
        format!("person.events.{}.{}", aggregate_id, event_type)
    }
}

/// NATS-based event store implementation
pub struct NatsEventStore {
    client: Client,
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
        
        jetstream.create_stream(stream_config).await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to create stream: {}", e),
            })?;
        
        Ok(Self {
            client,
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
                PersonEvent::ComponentRegistered(_) => "component_registered",
                PersonEvent::ComponentUnregistered(_) => "component_unregistered",
                PersonEvent::PersonDeactivated(_) => "deactivated",
                PersonEvent::PersonReactivated(_) => "reactivated",
                PersonEvent::PersonMergedInto(_) => "merged",
                PersonEvent::ComponentDataUpdated(_) => "component_updated",
            };
            
            let subject = PersonSubjects::event_for(aggregate_id, event_type);
            let sequence = self.get_current_version(aggregate_id).await? + index as u64 + 1;
            
            let envelope = EventEnvelope {
                aggregate_id,
                sequence,
                event,
                timestamp: chrono::Utc::now(),
                correlation_id: uuid::Uuid::new_v4().to_string(),
                causation_id: uuid::Uuid::new_v4().to_string(),
            };
            
            let payload = serde_json::to_vec(&envelope)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?;
            
            self.jetstream.publish(subject, payload.into()).await
                .map_err(|e| DomainError::ExternalServiceError {
                    service: "NATS JetStream".to_string(),
                    message: format!("Failed to publish event: {}", e),
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
        let subject_filter = format!("person.events.{}.>", aggregate_id);
        
        let consumer_config = jetstream::consumer::pull::Config {
            filter_subject: subject_filter,
            ..Default::default()
        };
        
        let consumer = self.jetstream
            .create_consumer_on_stream(consumer_config, self.stream_name.as_str())
            .await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to create consumer: {}", e),
            })?;
        
        let mut events = Vec::new();
        let mut messages = consumer.messages().await
            .map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to get messages: {}", e),
            })?;
        
        while let Some(msg) = messages.next().await {
            let msg = msg.map_err(|e| DomainError::ExternalServiceError {
                service: "NATS JetStream".to_string(),
                message: format!("Failed to get message: {}", e),
            })?;
            
            let envelope: EventEnvelope = serde_json::from_slice(&msg.payload)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?;
            
            if envelope.sequence >= from_version {
                events.push(envelope);
            }
            
            msg.ack().await
                .map_err(|e| DomainError::ExternalServiceError {
                    service: "NATS JetStream".to_string(),
                    message: format!("Failed to ack message: {}", e),
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
                message: format!("Failed to subscribe: {}", e),
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
                                message: format!("Failed to send reply: {}", e),
                            })?;
                    }
                }
                Err(e) => {
                    eprintln!("Error handling command: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle a single command
    async fn handle_command(&self, command: PersonCommand) -> DomainResult<CommandResponse> {
        let aggregate_id = match &command {
            PersonCommand::CreatePerson(cmd) => cmd.person_id,
            PersonCommand::UpdateName(cmd) => cmd.person_id,
            PersonCommand::SetBirthDate(cmd) => cmd.person_id,
            PersonCommand::RecordDeath(cmd) => cmd.person_id,
            PersonCommand::RegisterComponent(cmd) => cmd.person_id,
            PersonCommand::UnregisterComponent(cmd) => cmd.person_id,
            PersonCommand::DeactivatePerson(cmd) => cmd.person_id,
            PersonCommand::ReactivatePerson(cmd) => cmd.person_id,
            PersonCommand::MergePersons(cmd) => cmd.source_person_id,
        };
        
        // Load or create aggregate
        let mut person = match self.repository.load(aggregate_id).await? {
            Some(p) => p,
            None => {
                if matches!(command, PersonCommand::CreatePerson(_)) {
                    Person::empty()
                } else {
                    return Err(DomainError::AggregateNotFound(format!("Person {}", aggregate_id)));
                }
            }
        };
        
        // Handle command
        let events = person.handle_command(command)
            .map_err(|e| DomainError::ValidationError(e))?;
        
        // Save events
        let expected_version = if person.version == 0 { None } else { Some(person.version) };
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
            format!("person.commands.{}", person_id)
        );
        assert_eq!(
            PersonSubjects::event_for(person_id, "created"),
            format!("person.events.{}.created", person_id)
        );
    }
} 