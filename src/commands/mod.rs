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
    
    /// Archive a person
    ArchivePerson(ArchivePerson),
    
    /// Add a component with data
    AddComponent(AddComponent),
    
    /// Update component data
    UpdateComponent(UpdateComponent),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddComponent {
    pub person_id: PersonId,
    pub component_type: ComponentType,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateComponent {
    pub person_id: PersonId,
    pub component_id: uuid::Uuid,
    pub component_type: ComponentType,
    pub updates: serde_json::Value,
}

// Include new component commands
mod component_commands;
pub use component_commands::*;

impl PersonCommand {
    /// Get the aggregate ID this command applies to
    pub fn aggregate_id(&self) -> PersonId {
        match self {
            PersonCommand::CreatePerson(cmd) => cmd.person_id,
            PersonCommand::UpdateName(cmd) => cmd.person_id,
            PersonCommand::SetBirthDate(cmd) => cmd.person_id,
            PersonCommand::RecordDeath(cmd) => cmd.person_id,
            PersonCommand::RegisterComponent(cmd) => cmd.person_id,
            PersonCommand::UnregisterComponent(cmd) => cmd.person_id,
            PersonCommand::DeactivatePerson(cmd) => cmd.person_id,
            PersonCommand::ReactivatePerson(cmd) => cmd.person_id,
            PersonCommand::MergePersons(cmd) => cmd.source_person_id,
            PersonCommand::ArchivePerson(cmd) => cmd.person_id,
            PersonCommand::AddComponent(cmd) => cmd.person_id,
            PersonCommand::UpdateComponent(cmd) => cmd.person_id,
        }
    }
}


