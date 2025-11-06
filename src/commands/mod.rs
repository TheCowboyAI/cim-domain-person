//! Person domain commands
//!
//! Pure functional domain commands following CQRS:
//! - Core identity management (name, birth/death)
//! - Lifecycle management (active, deactivated, merged)
//!
//! Commands express intent and are validated before generating events.

use cim_domain::{EntityId, formal_domain::DomainCommand as DomainCommandTrait};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

use crate::aggregate::PersonMarker;
use crate::value_objects::{PersonName, PersonAttribute, AttributeType};

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

    /// Deactivate the person
    DeactivatePerson(DeactivatePerson),

    /// Reactivate the person
    ReactivatePerson(ReactivatePerson),

    /// Merge two persons
    MergePersons(MergePersons),

    /// Archive a person
    ArchivePerson(ArchivePerson),

    /// Record an attribute
    RecordAttribute(RecordAttribute),

    /// Update an attribute
    UpdateAttribute(UpdateAttribute),

    /// Invalidate an attribute
    InvalidateAttribute(InvalidateAttribute),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MergeReason {
    DuplicateIdentity,
    DataQualityIssue,
    UserRequested,
    PolicyDetermined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePerson {
    pub person_id: PersonId,
    pub reason: String,
}

// ===== Attribute Commands =====

/// Record a new attribute for a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordAttribute {
    pub person_id: PersonId,
    pub attribute: PersonAttribute,
}

/// Update an existing attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAttribute {
    pub person_id: PersonId,
    pub attribute_type: AttributeType,
    pub new_attribute: PersonAttribute,
}

/// Invalidate an attribute (mark as no longer valid)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateAttribute {
    pub person_id: PersonId,
    pub attribute_type: AttributeType,
    pub reason: Option<String>,
}

impl PersonCommand {
    /// Get the aggregate ID this command applies to
    pub fn aggregate_id(&self) -> PersonId {
        match self {
            PersonCommand::CreatePerson(cmd) => cmd.person_id,
            PersonCommand::UpdateName(cmd) => cmd.person_id,
            PersonCommand::SetBirthDate(cmd) => cmd.person_id,
            PersonCommand::RecordDeath(cmd) => cmd.person_id,
            PersonCommand::DeactivatePerson(cmd) => cmd.person_id,
            PersonCommand::ReactivatePerson(cmd) => cmd.person_id,
            PersonCommand::MergePersons(cmd) => cmd.source_person_id,
            PersonCommand::ArchivePerson(cmd) => cmd.person_id,
            PersonCommand::RecordAttribute(cmd) => cmd.person_id,
            PersonCommand::UpdateAttribute(cmd) => cmd.person_id,
            PersonCommand::InvalidateAttribute(cmd) => cmd.person_id,
        }
    }
}

// Implement DomainCommand trait for formal Category Theory compliance
impl DomainCommandTrait for PersonCommand {
    fn name(&self) -> &str {
        match self {
            PersonCommand::CreatePerson(_) => "CreatePerson",
            PersonCommand::UpdateName(_) => "UpdateName",
            PersonCommand::SetBirthDate(_) => "SetBirthDate",
            PersonCommand::RecordDeath(_) => "RecordDeath",
            PersonCommand::DeactivatePerson(_) => "DeactivatePerson",
            PersonCommand::ReactivatePerson(_) => "ReactivatePerson",
            PersonCommand::MergePersons(_) => "MergePersons",
            PersonCommand::ArchivePerson(_) => "ArchivePerson",
            PersonCommand::RecordAttribute(_) => "RecordAttribute",
            PersonCommand::UpdateAttribute(_) => "UpdateAttribute",
            PersonCommand::InvalidateAttribute(_) => "InvalidateAttribute",
        }
    }
}


