//! Person domain commands - ECS Architecture
//!
//! In ECS architecture, commands focus on:
//! - Core identity management (name, birth/death)
//! - Lifecycle management (active, deactivated, merged)
//! - Component registration tracking
//!
//! Component-specific operations are handled by their respective systems.

use cim_domain::EntityId;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

use crate::aggregate::{PersonMarker, ComponentType};
use crate::value_objects::PersonName;

/// Person ID type alias
pub type PersonId = EntityId<PersonMarker>;

/// Commands for the Person domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonCommand {
    /// Create a new person
    CreatePerson(CreatePerson),
    
    /// Update person's name
    UpdateName(UpdateName),
    
    /// Set birth date
    SetBirthDate(SetBirthDate),
    
    /// Record death
    RecordDeath(RecordDeath),
    
    /// Register a component
    RegisterComponent(RegisterComponent),
    
    /// Unregister a component
    UnregisterComponent(UnregisterComponent),
    
    /// Deactivate the person
    DeactivatePerson(DeactivatePerson),
    
    /// Reactivate the person
    ReactivatePerson(ReactivatePerson),
    
    /// Merge two persons
    MergePersons(MergePersons),
}

// ===== Core Identity Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerson {
    pub person_id: PersonId,
    pub name: PersonName,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateName {
    pub person_id: PersonId,
    pub name: PersonName,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBirthDate {
    pub person_id: PersonId,
    pub birth_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordDeath {
    pub person_id: PersonId,
    pub date_of_death: NaiveDate,
}

// ===== Component Management Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterComponent {
    pub person_id: PersonId,
    pub component_type: ComponentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnregisterComponent {
    pub person_id: PersonId,
    pub component_type: ComponentType,
}

// ===== Lifecycle Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeactivatePerson {
    pub person_id: PersonId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactivatePerson {
    pub person_id: PersonId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergePersons {
    pub source_person_id: PersonId,
    pub target_person_id: PersonId,
    pub merge_reason: MergeReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeReason {
    DuplicateIdentity,
    DataQualityIssue,
    UserRequested,
    PolicyDetermined,
}

// ===== Legacy Command Stubs (for migration) =====
// These are kept temporarily to avoid breaking existing code
// They should be removed once all systems are updated

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEmail;

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveEmail;

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEmail;

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPhone;

#[deprecated(since = "0.3.0", note = "Use component systems for contact management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovePhone;

#[deprecated(since = "0.3.0", note = "Use location domain for address management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAddress;

#[deprecated(since = "0.3.0", note = "Use location domain for address management")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAddress;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships for employment")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEmployment;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships for employment")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEmployment;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships for employment")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndEmployment;

#[deprecated(since = "0.3.0", note = "Use component systems for skills")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSkill;

#[deprecated(since = "0.3.0", note = "Use component systems for skills")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSkill;

#[deprecated(since = "0.3.0", note = "Use component systems for skills")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSkill;

#[deprecated(since = "0.3.0", note = "Use component systems for certifications")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCertification;

#[deprecated(since = "0.3.0", note = "Use component systems for education")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEducation;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRelationship;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRelationship;

#[deprecated(since = "0.3.0", note = "Use cross-domain relationships")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndRelationship;

#[deprecated(since = "0.3.0", note = "Use component systems for social profiles")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSocialProfile;

#[deprecated(since = "0.3.0", note = "Use component systems for social profiles")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSocialProfile;

#[deprecated(since = "0.3.0", note = "Use component systems for social profiles")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSocialProfile;

#[deprecated(since = "0.3.0", note = "Use component systems for customer data")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCustomerSegment;

#[deprecated(since = "0.3.0", note = "Use component systems for behavioral data")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBehavioralData;

#[deprecated(since = "0.3.0", note = "Use component systems for preferences")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCommunicationPreferences;

#[deprecated(since = "0.3.0", note = "Use component systems for preferences")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPrivacyPreferences;

#[deprecated(since = "0.3.0", note = "Use component systems for tagging")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTag;

#[deprecated(since = "0.3.0", note = "Use component systems for tagging")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTag;

#[deprecated(since = "0.3.0", note = "Use component systems for custom attributes")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCustomAttribute;


