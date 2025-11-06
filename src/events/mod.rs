//! Person domain events
//!
//! Pure functional domain events capturing state changes:
//! - Core identity changes (name, birth/death)
//! - Lifecycle changes (active, deactivated, merged)
//!
//! All events are immutable and represent facts that have happened.

use cim_domain::{EntityId, formal_domain::DomainEvent as DomainEventTrait};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};

use crate::aggregate::PersonMarker;
use crate::value_objects::{PersonName, PersonAttribute, AttributeType};
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

    /// Person was deactivated
    PersonDeactivated(PersonDeactivated),

    /// Person was reactivated
    PersonReactivated(PersonReactivated),

    /// Person was merged into another
    PersonMergedInto(PersonMergedInto),

    /// Attribute was recorded
    AttributeRecorded(AttributeRecorded),

    /// Attribute was updated
    AttributeUpdated(AttributeUpdated),

    /// Attribute was invalidated
    AttributeInvalidated(AttributeInvalidated),
}

// Implement DomainEvent trait for formal Category Theory compliance
impl DomainEventTrait for PersonEvent {
    fn name(&self) -> &str {
        match self {
            PersonEvent::PersonCreated(_) => "PersonCreated",
            PersonEvent::PersonUpdated(_) => "PersonUpdated",
            PersonEvent::NameUpdated(_) => "NameUpdated",
            PersonEvent::BirthDateSet(_) => "BirthDateSet",
            PersonEvent::DeathRecorded(_) => "DeathRecorded",
            PersonEvent::PersonDeactivated(_) => "PersonDeactivated",
            PersonEvent::PersonReactivated(_) => "PersonReactivated",
            PersonEvent::PersonMergedInto(_) => "PersonMergedInto",
            PersonEvent::AttributeRecorded(_) => "AttributeRecorded",
            PersonEvent::AttributeUpdated(_) => "AttributeUpdated",
            PersonEvent::AttributeInvalidated(_) => "AttributeInvalidated",
        }
    }
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

// ===== Attribute Events =====

/// An attribute was recorded for a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeRecorded {
    pub person_id: PersonId,
    pub attribute: PersonAttribute,
    pub recorded_at: DateTime<Utc>,
}

/// An existing attribute was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeUpdated {
    pub person_id: PersonId,
    pub attribute_type: AttributeType,
    pub old_attribute: PersonAttribute,
    pub new_attribute: PersonAttribute,
    pub updated_at: DateTime<Utc>,
}

/// An attribute was marked as invalid/expired
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeInvalidated {
    pub person_id: PersonId,
    pub attribute_type: AttributeType,
    pub invalidated_at: DateTime<Utc>,
    pub reason: Option<String>,
}

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
