//! Async command processor for pure event-driven architecture

use async_trait::async_trait;
use cim_domain::{DomainError, DomainResult};
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, debug};

use crate::aggregate::{Person, PersonId, EventSourced};
use crate::commands::PersonCommand;
use crate::events::{PersonEvent, PersonEventV2, StreamingEventEnvelope, EventMetadata};
use crate::infrastructure::{StreamingClient, EventStore};

/// Result of command processing with streaming events
pub struct CommandResult {
    pub aggregate_id: PersonId,
    pub version: u64,
    pub events: Vec<PersonEventV2>,
    pub event_stream: Option<Pin<Box<dyn Stream<Item = PersonEventV2> + Send>>>,
}

/// Async command processor for Person domain
#[async_trait]
pub trait AsyncCommandProcessor: Send + Sync {
    /// Process a command and return a stream of events
    async fn process_command(&self, command: PersonCommand) -> DomainResult<CommandResult>;
    
    /// Process a command with correlation
    async fn process_command_with_correlation(
        &self,
        command: PersonCommand,
        correlation_id: uuid::Uuid,
    ) -> DomainResult<CommandResult>;
}

/// Implementation of async command processor
pub struct PersonCommandProcessor {
    event_store: Arc<dyn EventStore>,
    streaming_client: Arc<StreamingClient>,
}

impl PersonCommandProcessor {
    /// Create a new command processor
    pub fn new(
        event_store: Arc<dyn EventStore>,
        streaming_client: Arc<StreamingClient>,
    ) -> Self {
        Self {
            event_store,
            streaming_client,
        }
    }
    
    /// Load aggregate from event store
    async fn load_aggregate(&self, aggregate_id: PersonId) -> DomainResult<Option<Person>> {
        let events = self.event_store.get_events(aggregate_id).await?;
        
        if events.is_empty() {
            return Ok(None);
        }
        
        // Replay events to rebuild aggregate (pure functional)
        let person = Person::empty();
        let person = events.into_iter().try_fold(person, |p, envelope| {
            // Convert PersonEventV2 to PersonEvent for backward compatibility
            let event: PersonEvent = envelope.event;
            p.apply_event(&event)
        })?;

        Ok(Some(person))
    }
    
    /// Convert old events to new format with metadata
    fn convert_to_v2_events(
        &self,
        events: Vec<crate::events::PersonEvent>,
        metadata: EventMetadata,
    ) -> Vec<PersonEventV2> {
        events.into_iter().map(|event| {
            match event {
                crate::events::PersonEvent::PersonCreated(e) => PersonEventV2::Created {
                    person_id: e.person_id,
                    name: e.name,
                    source: e.source,
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::PersonUpdated(e) => PersonEventV2::Updated {
                    person_id: e.person_id,
                    updates: serde_json::json!({ "name": e.name }),
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::NameUpdated(e) => PersonEventV2::NameUpdated {
                    person_id: e.person_id,
                    old_name: e.old_name,
                    new_name: e.new_name,
                    change_reason: e.reason,
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::BirthDateSet(e) => PersonEventV2::BirthDateSet {
                    person_id: e.person_id,
                    birth_date: e.birth_date,
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::DeathRecorded(e) => PersonEventV2::DeathRecorded {
                    person_id: e.person_id,
                    date_of_death: e.date_of_death,
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::PersonDeactivated(e) => PersonEventV2::Suspended {
                    person_id: e.person_id,
                    reason: e.reason,
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::PersonReactivated(e) => PersonEventV2::Activated {
                    person_id: e.person_id,
                    reason: e.reason,
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::PersonMergedInto(e) => PersonEventV2::PersonMerged {
                    source_person_id: e.source_person_id,
                    target_person_id: e.merged_into_id,
                    merge_reason: e.merge_reason,
                    metadata: metadata.clone(),
                },
                // Attribute events - convert to Updated for now
                // In the future, PersonEventV2 should have dedicated attribute variants
                crate::events::PersonEvent::AttributeRecorded(e) => PersonEventV2::Updated {
                    person_id: e.person_id,
                    updates: serde_json::json!({ "attribute_recorded": e.attribute }),
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::AttributeUpdated(e) => PersonEventV2::Updated {
                    person_id: e.person_id,
                    updates: serde_json::json!({ "attribute_updated": e.new_attribute }),
                    metadata: metadata.clone(),
                },
                crate::events::PersonEvent::AttributeInvalidated(e) => PersonEventV2::Updated {
                    person_id: e.person_id,
                    updates: serde_json::json!({ "attribute_invalidated": e.attribute_type }),
                    metadata: metadata.clone(),
                },
            }
        }).collect()
    }
    
    /// Publish events to NATS
    async fn publish_events(
        &self,
        aggregate_id: PersonId,
        events: &[PersonEventV2],
        start_sequence: u64,
    ) -> DomainResult<()> {
        for (index, event) in events.iter().enumerate() {
            let sequence = start_sequence + index as u64 + 1;
            let envelope = StreamingEventEnvelope::new(aggregate_id, sequence, event.clone());
            
            let subject = envelope.subject();
            let payload = serde_json::to_vec(&envelope)
                .map_err(|e| DomainError::SerializationError(e.to_string()))?;
            
            self.streaming_client
                .jetstream()
                .publish(subject, payload.into())
                .await
                .map_err(|e| DomainError::ExternalServiceError {
                    service: "NATS JetStream".to_string(),
                    message: format!("Failed to publish event: {e}"),
                })?;
            
            debug!("Published event {} to {}", event.event_type(), envelope.subject());
        }
        
        Ok(())
    }
}

#[async_trait]
impl AsyncCommandProcessor for PersonCommandProcessor {
    async fn process_command(&self, command: PersonCommand) -> DomainResult<CommandResult> {
        let correlation_id = uuid::Uuid::now_v7();
        self.process_command_with_correlation(command, correlation_id).await
    }
    
    async fn process_command_with_correlation(
        &self,
        command: PersonCommand,
        correlation_id: uuid::Uuid,
    ) -> DomainResult<CommandResult> {
        let command_id = uuid::Uuid::now_v7();
        let mut metadata = EventMetadata::from_command(command_id);
        metadata.correlation_id = correlation_id;
        
        info!("Processing command {:?} with correlation {}", command, correlation_id);
        
        // Get aggregate ID from command
        let aggregate_id = command.aggregate_id();
        
        // Load or create aggregate
        let person = match self.load_aggregate(aggregate_id).await? {
            Some(p) => p,
            None => {
                if matches!(command, PersonCommand::CreatePerson(_)) {
                    Person::empty()
                } else {
                    return Err(DomainError::AggregateNotFound(
                        format!("Person {}", aggregate_id)
                    ));
                }
            }
        };
        
        let current_version = person.version;

        // Handle command using formal Aggregate trait (pure functional)
        use cim_domain::formal_domain::Aggregate;
        let (_person, events) = person.handle(command)?;

        // Convert to V2 events with metadata
        let v2_events = self.convert_to_v2_events(events, metadata);
        
        // Store and publish events
        if !v2_events.is_empty() {
            // Store in event store (using old format for compatibility)
            let old_events: Vec<crate::events::PersonEvent> = v2_events.iter()
                .map(|e| self.convert_v2_to_old(e))
                .collect();
            
            self.event_store
                .append_events(aggregate_id, old_events, Some(current_version))
                .await?;
            
            // Publish to NATS
            self.publish_events(aggregate_id, &v2_events, current_version).await?;
        }
        
        // Create event stream for real-time updates
        let (tx, rx) = mpsc::channel(10);
        let mut event_stream = Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx));
        
        // Demonstrate StreamExt usage - peek at first event if any
        if let Some(first_event) = event_stream.next().await {
            debug!("First event in stream: {:?}", first_event);
            // Put it back by creating a new stream with the first event
            let (tx2, rx2) = mpsc::channel(10);
            let event_stream_new = Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx2));
            
            // Send the first event back
            let tx2_clone = tx2.clone();
            tokio::spawn(async move {
                let _ = tx2_clone.send(first_event).await;
            });
            
            // Forward remaining events from original stream
            let mut event_stream_temp = event_stream;
            tokio::spawn(async move {
                while let Some(event) = event_stream_temp.next().await {
                    if tx2.send(event).await.is_err() {
                        break;
                    }
                }
            });
            
            event_stream = event_stream_new;
        }
        
        // Send initial events
        let events_to_stream = v2_events.clone();
        tokio::spawn(async move {
            for event in events_to_stream {
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
        
        Ok(CommandResult {
            aggregate_id,
            version: current_version + v2_events.len() as u64,
            events: v2_events,
            event_stream: Some(event_stream),
        })
    }
}

impl PersonCommandProcessor {
    /// Convert V2 event back to old format for storage compatibility
    fn convert_v2_to_old(&self, event: &PersonEventV2) -> crate::events::PersonEvent {
        match event {
            PersonEventV2::Created { person_id, name, source, .. } => {
                crate::events::PersonEvent::PersonCreated(crate::events::PersonCreated {
                    person_id: *person_id,
                    name: name.clone(),
                    source: source.clone(),
                    created_at: chrono::Utc::now(),
                })
            }
            PersonEventV2::NameUpdated { person_id, old_name, new_name, change_reason, .. } => {
                crate::events::PersonEvent::NameUpdated(crate::events::NameUpdated {
                    person_id: *person_id,
                    old_name: old_name.clone(),
                    new_name: new_name.clone(),
                    reason: change_reason.clone(),
                    updated_at: chrono::Utc::now(),
                })
            }
            // ... other conversions
            _ => panic!("Unhandled event conversion"),
        }
    }
}

/// Command handler trait for components
#[async_trait]
pub trait AsyncComponentCommandHandler: Send + Sync {
    /// Handle a component-specific command
    async fn handle_component_command(
        &self,
        person_id: PersonId,
        command: serde_json::Value,
    ) -> DomainResult<Vec<PersonEventV2>>;
}

#[cfg(test)]
mod tests {
    // Mock implementations would go here
}