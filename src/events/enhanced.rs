//! Enhanced events with metadata for pure event-driven architecture

use crate::aggregate::PersonId;
use crate::value_objects::PersonName;
use crate::commands::MergeReason;
use super::{EventMetadata, PersonEvent, PersonCreated, NameUpdated, BirthDateSet, DeathRecorded};
use super::{PersonDeactivated, PersonReactivated, PersonMergedInto};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// Enhanced person events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum PersonEventV2 {
    // Lifecycle events
    Created {
        person_id: PersonId,
        name: PersonName,
        source: String,
        metadata: EventMetadata,
    },
    Activated {
        person_id: PersonId,
        reason: String,
        metadata: EventMetadata,
    },
    Suspended {
        person_id: PersonId,
        reason: String,
        metadata: EventMetadata,
    },
    Archived {
        person_id: PersonId,
        reason: String,
        metadata: EventMetadata,
    },
    Updated {
        person_id: PersonId,
        updates: serde_json::Value,
        metadata: EventMetadata,
    },
    
    // Identity events
    NameUpdated {
        person_id: PersonId,
        old_name: PersonName,
        new_name: PersonName,
        change_reason: Option<String>,
        metadata: EventMetadata,
    },
    BirthDateSet {
        person_id: PersonId,
        birth_date: NaiveDate,
        metadata: EventMetadata,
    },
    DeathRecorded {
        person_id: PersonId,
        date_of_death: NaiveDate,
        metadata: EventMetadata,
    },

    // Relationship events
    PersonMerged {
        source_person_id: PersonId,
        target_person_id: PersonId,
        merge_reason: MergeReason,
        metadata: EventMetadata,
    },
}

impl PersonEventV2 {
    /// Get the aggregate ID this event applies to
    pub fn aggregate_id(&self) -> PersonId {
        match self {
            PersonEventV2::Created { person_id, .. } |
            PersonEventV2::Activated { person_id, .. } |
            PersonEventV2::Suspended { person_id, .. } |
            PersonEventV2::Archived { person_id, .. } |
            PersonEventV2::Updated { person_id, .. } |
            PersonEventV2::NameUpdated { person_id, .. } |
            PersonEventV2::BirthDateSet { person_id, .. } |
            PersonEventV2::DeathRecorded { person_id, .. } => *person_id,
            PersonEventV2::PersonMerged { source_person_id, .. } => *source_person_id,
        }
    }
    
    /// Get the metadata for this event
    pub fn metadata(&self) -> &EventMetadata {
        match self {
            PersonEventV2::Created { metadata, .. } |
            PersonEventV2::Activated { metadata, .. } |
            PersonEventV2::Suspended { metadata, .. } |
            PersonEventV2::Archived { metadata, .. } |
            PersonEventV2::Updated { metadata, .. } |
            PersonEventV2::NameUpdated { metadata, .. } |
            PersonEventV2::BirthDateSet { metadata, .. } |
            PersonEventV2::DeathRecorded { metadata, .. } |
            PersonEventV2::PersonMerged { metadata, .. } => metadata,
        }
    }
    
    /// Get the event type as a string for routing
    pub fn event_type(&self) -> &'static str {
        match self {
            PersonEventV2::Created { .. } => "person.created",
            PersonEventV2::Activated { .. } => "person.activated",
            PersonEventV2::Suspended { .. } => "person.suspended",
            PersonEventV2::Archived { .. } => "person.archived",
            PersonEventV2::Updated { .. } => "person.updated",
            PersonEventV2::NameUpdated { .. } => "person.name_updated",
            PersonEventV2::BirthDateSet { .. } => "person.birth_date_set",
            PersonEventV2::DeathRecorded { .. } => "person.death_recorded",
            PersonEventV2::PersonMerged { .. } => "person.merged",
        }
    }
    
    /// Get the NATS subject for this event
    pub fn subject(&self) -> String {
        let aggregate_id = self.aggregate_id();
        let event_type = self.event_type();
        format!("person.events.{}.{}", aggregate_id, event_type)
    }
}

/// Event envelope for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingEventEnvelope {
    /// Unique event ID
    pub event_id: uuid::Uuid,
    /// Aggregate ID
    pub aggregate_id: PersonId,
    /// Sequence number for this aggregate
    pub sequence: u64,
    /// The actual event
    pub event: PersonEventV2,
    /// Stream position (assigned by NATS)
    pub stream_position: Option<u64>,
}

impl StreamingEventEnvelope {
    /// Create a new envelope
    pub fn new(aggregate_id: PersonId, sequence: u64, event: PersonEventV2) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            aggregate_id,
            sequence,
            event,
            stream_position: None,
        }
    }
    
    /// Get the NATS subject for this envelope
    pub fn subject(&self) -> String {
        self.event.subject()
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &EventMetadata {
        self.event.metadata()
    }
}

// Conversion from V2 to V1 events for backward compatibility
impl From<PersonEventV2> for PersonEvent {
    fn from(event: PersonEventV2) -> Self {
        match event {
            PersonEventV2::Created { person_id, name, source, metadata } => {
                PersonEvent::PersonCreated(PersonCreated {
                    person_id,
                    name,
                    source,
                    created_at: metadata.timestamp,
                })
            }
            PersonEventV2::NameUpdated { person_id, old_name, new_name, change_reason, metadata } => {
                PersonEvent::NameUpdated(NameUpdated {
                    person_id,
                    old_name,
                    new_name,
                    reason: change_reason,
                    updated_at: metadata.timestamp,
                })
            }
            PersonEventV2::BirthDateSet { person_id, birth_date, metadata } => {
                PersonEvent::BirthDateSet(BirthDateSet {
                    person_id,
                    birth_date,
                    set_at: metadata.timestamp,
                })
            }
            PersonEventV2::DeathRecorded { person_id, date_of_death, metadata } => {
                PersonEvent::DeathRecorded(DeathRecorded {
                    person_id,
                    date_of_death,
                    recorded_at: metadata.timestamp,
                })
            }
            PersonEventV2::Suspended { person_id, reason, metadata } => {
                PersonEvent::PersonDeactivated(PersonDeactivated {
                    person_id,
                    reason,
                    deactivated_at: metadata.timestamp,
                })
            }
            PersonEventV2::Activated { person_id, reason, metadata } => {
                PersonEvent::PersonReactivated(PersonReactivated {
                    person_id,
                    reason,
                    reactivated_at: metadata.timestamp,
                })
            }
            PersonEventV2::PersonMerged { source_person_id, target_person_id, merge_reason, metadata } => {
                PersonEvent::PersonMergedInto(PersonMergedInto {
                    source_person_id,
                    merged_into_id: target_person_id,
                    merge_reason,
                    merged_at: metadata.timestamp,
                })
            }
            // Events that don't have V1 equivalents - map to generic update
            PersonEventV2::Archived { .. } |
            PersonEventV2::Updated { .. } => {
                // These don't have direct V1 equivalents - they would need specific handling
                // For now, we'll panic as these conversions shouldn't happen in normal flow
                panic!("Cannot convert V2 event to V1: event type not supported in V1 schema")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_subject_generation() {
        let person_id = PersonId::new();
        let event = PersonEventV2::Created {
            person_id,
            name: PersonName::new("John".to_string(), "Doe".to_string()),
            source: "test".to_string(),
            metadata: EventMetadata::new(),
        };
        
        let subject = event.subject();
        assert!(subject.starts_with("person.events."));
        assert!(subject.ends_with(".person.created"));
    }
    
    #[test]
    fn test_event_metadata_access() {
        let metadata = EventMetadata::new();
        let correlation_id = metadata.correlation_id;
        
        let event = PersonEventV2::Activated {
            person_id: PersonId::new(),
            reason: "Test activation".to_string(),
            metadata,
        };
        
        assert_eq!(event.metadata().correlation_id, correlation_id);
        assert_eq!(event.event_type(), "person.activated");
    }
}
