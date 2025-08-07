//! Enhanced events with metadata for pure event-driven architecture

use crate::aggregate::{PersonId, ComponentType};
use crate::value_objects::PersonName;
use crate::commands::MergeReason;
use super::{EventMetadata, PersonEvent, PersonCreated, NameUpdated, BirthDateSet, DeathRecorded};
use super::{ComponentRegistered, ComponentUnregistered, PersonDeactivated, PersonReactivated};
use super::{PersonMergedInto, ComponentDataUpdated};
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
    
    // Component events
    ComponentAdded {
        person_id: PersonId,
        component_type: ComponentType,
        component_data: serde_json::Value,
        metadata: EventMetadata,
    },
    ComponentUpdated {
        person_id: PersonId,
        component_type: ComponentType,
        component_id: uuid::Uuid,
        changes: serde_json::Value,
        metadata: EventMetadata,
    },
    ComponentRemoved {
        person_id: PersonId,
        component_type: ComponentType,
        component_id: uuid::Uuid,
        metadata: EventMetadata,
    },
    
    // Relationship events
    PersonMerged {
        source_person_id: PersonId,
        target_person_id: PersonId,
        merge_reason: MergeReason,
        metadata: EventMetadata,
    },
    
    // Cross-domain events
    LocationAssigned {
        person_id: PersonId,
        location_id: uuid::Uuid,
        location_type: String,
        metadata: EventMetadata,
    },
    EmploymentAdded {
        person_id: PersonId,
        organization_id: uuid::Uuid,
        role: String,
        metadata: EventMetadata,
    },
    IdentityLinked {
        person_id: PersonId,
        identity_id: uuid::Uuid,
        identity_type: String,
        metadata: EventMetadata,
    },
    MetadataUpdated {
        person_id: PersonId,
        metadata_type: String,
        metadata_value: serde_json::Value,
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
            PersonEventV2::DeathRecorded { person_id, .. } |
            PersonEventV2::ComponentAdded { person_id, .. } |
            PersonEventV2::ComponentUpdated { person_id, .. } |
            PersonEventV2::ComponentRemoved { person_id, .. } |
            PersonEventV2::LocationAssigned { person_id, .. } |
            PersonEventV2::EmploymentAdded { person_id, .. } |
            PersonEventV2::IdentityLinked { person_id, .. } |
            PersonEventV2::MetadataUpdated { person_id, .. } => *person_id,
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
            PersonEventV2::ComponentAdded { metadata, .. } |
            PersonEventV2::ComponentUpdated { metadata, .. } |
            PersonEventV2::ComponentRemoved { metadata, .. } |
            PersonEventV2::PersonMerged { metadata, .. } |
            PersonEventV2::LocationAssigned { metadata, .. } |
            PersonEventV2::EmploymentAdded { metadata, .. } |
            PersonEventV2::IdentityLinked { metadata, .. } |
            PersonEventV2::MetadataUpdated { metadata, .. } => metadata,
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
            PersonEventV2::ComponentAdded { .. } => "person.component_added",
            PersonEventV2::ComponentUpdated { .. } => "person.component_updated",
            PersonEventV2::ComponentRemoved { .. } => "person.component_removed",
            PersonEventV2::PersonMerged { .. } => "person.merged",
            PersonEventV2::LocationAssigned { .. } => "person.location_assigned",
            PersonEventV2::EmploymentAdded { .. } => "person.employment_added",
            PersonEventV2::IdentityLinked { .. } => "person.identity_linked",
            PersonEventV2::MetadataUpdated { .. } => "person.metadata_updated",
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
            event_id: uuid::Uuid::new_v4(),
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_subject_generation() {
        let person_id = PersonId::new();
        let event = PersonEventV2::Created {
            person_id,
            name: PersonName::new("John", None, "Doe").unwrap(),
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
            PersonEventV2::ComponentAdded { person_id, component_type, .. } => {
                PersonEvent::ComponentRegistered(ComponentRegistered {
                    person_id,
                    component_type,
                    registered_at: chrono::Utc::now(),
                })
            }
            PersonEventV2::ComponentRemoved { person_id, component_type, .. } => {
                PersonEvent::ComponentUnregistered(ComponentUnregistered {
                    person_id,
                    component_type,
                    unregistered_at: chrono::Utc::now(),
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
            PersonEventV2::ComponentUpdated { person_id,  component_id,  .. } => {
                // Convert changes to ComponentData
                // For now, create a placeholder since the exact mapping depends on component type
                PersonEvent::ComponentDataUpdated(ComponentDataUpdated {
                    person_id,
                    component_id,
                    data: crate::components::data::ComponentData::Contact(
                        crate::components::data::ContactData::Email(
                            crate::components::data::EmailData {
                                email: crate::value_objects::EmailAddress::new("placeholder@example.com".to_string()).unwrap(),
                                email_type: crate::components::data::EmailType::Personal,
                                is_preferred_contact: false,
                                can_receive_notifications: false,
                                can_receive_marketing: false,
                            }
                        )
                    ),
                    updated_at: chrono::Utc::now(),
                })
            }
            // Events that don't have V1 equivalents return a generic update
            PersonEventV2::Archived { person_id, .. } |
            PersonEventV2::Updated { person_id, .. } |
            PersonEventV2::LocationAssigned { person_id, .. } |
            PersonEventV2::EmploymentAdded { person_id, .. } |
            PersonEventV2::IdentityLinked { person_id, .. } |
            PersonEventV2::MetadataUpdated { person_id, .. } => {
                // For events without V1 equivalent, we could either:
                // 1. Add new variants to PersonEvent
                // 2. Map to a generic event
                // 3. Use ComponentDataUpdated as a catch-all
                // For now, let's create a component update
                PersonEvent::ComponentDataUpdated(ComponentDataUpdated {
                    person_id,
                    component_id: uuid::Uuid::new_v4(),
                    data: crate::components::data::ComponentData::Contact(
                        crate::components::data::ContactData::Email(
                            crate::components::data::EmailData {
                                email: crate::value_objects::EmailAddress::new("placeholder@example.com".to_string()).unwrap(),
                                email_type: crate::components::data::EmailType::Personal,
                                is_preferred_contact: false,
                                can_receive_notifications: false,
                                can_receive_marketing: false,
                            }
                        )
                    ),
                    updated_at: chrono::Utc::now(),
                })
            }
        }
    }
}
