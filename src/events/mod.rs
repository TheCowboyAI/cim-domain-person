//! Person domain events - ECS Architecture
//!
//! In ECS architecture, events focus on:
//! - Core identity changes (name, birth/death)
//! - Lifecycle changes (active, deactivated, merged)
//! - Component registration tracking
//!
//! Component-specific events are handled by their respective systems.

use cim_domain::EntityId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};

use crate::aggregate::{PersonMarker, ComponentType};
use crate::value_objects::PersonName;
use crate::commands::MergeReason;

/// Person ID type alias
pub type PersonId = EntityId<PersonMarker>;

/// Events for the Person domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonEvent {
    /// Person was created
    PersonCreated(PersonCreated),
    
    /// Person was updated
    PersonUpdated(PersonUpdated),
    
    /// Person's name was updated
    NameUpdated(NameUpdated),
    
    /// Birth date was set
    BirthDateSet(BirthDateSet),
    
    /// Death was recorded
    DeathRecorded(DeathRecorded),
    
    /// Component was registered
    ComponentRegistered(ComponentRegistered),
    
    /// Component was unregistered
    ComponentUnregistered(ComponentUnregistered),
    
    /// Person was deactivated
    PersonDeactivated(PersonDeactivated),
    
    /// Person was reactivated
    PersonReactivated(PersonReactivated),
    
    /// Person was merged into another
    PersonMergedInto(PersonMergedInto),
    
    /// Component data was updated
    ComponentDataUpdated(ComponentDataUpdated),
}

// ===== Core Identity Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreated {
    pub person_id: PersonId,
    pub name: PersonName,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonUpdated {
    pub person_id: PersonId,
    pub name: PersonName,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameUpdated {
    pub person_id: PersonId,
    pub old_name: PersonName,
    pub new_name: PersonName,
    pub reason: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthDateSet {
    pub person_id: PersonId,
    pub birth_date: NaiveDate,
    pub set_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathRecorded {
    pub person_id: PersonId,
    pub date_of_death: NaiveDate,
    pub recorded_at: DateTime<Utc>,
}

// ===== Component Management Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRegistered {
    pub person_id: PersonId,
    pub component_type: ComponentType,
    pub registered_at: DateTime<Utc>,
    #[serde(default = "default_registered_by")]
    pub registered_by: String,  // Track who/what system registered this
}

fn default_registered_by() -> String {
    "unknown".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUnregistered {
    pub person_id: PersonId,
    pub component_type: ComponentType,
    pub unregistered_at: DateTime<Utc>,
}

// ===== Lifecycle Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonDeactivated {
    pub person_id: PersonId,
    pub reason: String,
    pub deactivated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonReactivated {
    pub person_id: PersonId,
    pub reason: String,
    pub reactivated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonMergedInto {
    pub source_person_id: PersonId,
    pub merged_into_id: PersonId,
    pub merge_reason: MergeReason,
    pub merged_at: DateTime<Utc>,
}

// Include new component events
mod component_events;
pub use component_events::*;

// Enhanced events with metadata
mod enhanced;
pub use enhanced::{PersonEventV2, StreamingEventEnvelope};

// Event versioning support
mod versioning;
mod versioned_events;
pub use versioning::{
    VersionedEvent, EventVersionRegistry, EventMigration, 
    FunctionMigration, VersionedEventEnvelope
};
pub use versioned_events::{
    PersonCreatedV2, PersonNameUpdatedV2, PersonActivatedV2,
    PersonSuspendedV2, PersonArchivedV2, ComponentAddedV2,
    ComponentUpdatedV2, ComponentRemovedV2, create_event_registry
};

// Re-export EventMetadata from infrastructure
pub use crate::infrastructure::EventMetadata;
