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
    pub reason: MergeReason,
    pub merged_at: DateTime<Utc>,
}

// ===== Legacy Event Stubs (for migration) =====
// These are kept temporarily to avoid breaking existing code
// They should be removed once all systems are updated

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAdded;

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRemoved;

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerified;

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneAdded;

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneRemoved;

#[deprecated(since = "0.3.0", note = "Use location domain for address management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressAdded;

#[deprecated(since = "0.3.0", note = "Use location domain for address management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressRemoved;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships for employment")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentAdded;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships for employment")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentUpdated;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships for employment")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentEnded;

#[deprecated(since = "0.3.0", note = "Use component systems for skills")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAdded;

#[deprecated(since = "0.3.0", note = "Use component systems for skills")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdated;

#[deprecated(since = "0.3.0", note = "Use component systems for skills")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRemoved;

#[deprecated(since = "0.3.0", note = "Use component systems for certifications")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationAdded;

#[deprecated(since = "0.3.0", note = "Use component systems for education")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationAdded;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipAdded;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipUpdated;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEnded;

#[deprecated(since = "0.3.0", note = "Use component systems for social profiles")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfileAdded;

#[deprecated(since = "0.3.0", note = "Use component systems for social profiles")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfileUpdated;

#[deprecated(since = "0.3.0", note = "Use component systems for social profiles")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfileRemoved;

#[deprecated(since = "0.3.0", note = "Use component systems for customer data")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSegmentSet;

#[deprecated(since = "0.3.0", note = "Use component systems for behavioral data")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralDataUpdated;

#[deprecated(since = "0.3.0", note = "Use component systems for preferences")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPreferencesSet;

#[deprecated(since = "0.3.0", note = "Use component systems for preferences")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPreferencesSet;

#[deprecated(since = "0.3.0", note = "Use component systems for tagging")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagAdded;

#[deprecated(since = "0.3.0", note = "Use component systems for tagging")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRemoved;

#[deprecated(since = "0.3.0", note = "Use component systems for custom attributes")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributeSet;
