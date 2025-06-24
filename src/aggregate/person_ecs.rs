//! ECS-oriented Person aggregate
//! 
//! In ECS architecture, entities are just IDs with components attached.
//! The Person aggregate only maintains core identity and invariants.

use cim_domain::{AggregateRoot, DomainError, DomainResult, EntityId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chrono::{DateTime, Utc};

use crate::value_objects::PersonName;
use crate::commands::*;
use crate::events::*;

/// Marker type for Person entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonMarker;

/// Person ID type alias
pub type PersonId = EntityId<PersonMarker>;

/// The Person aggregate - minimal core identity only
/// 
/// In ECS architecture, a Person is just:
/// - An ID (entity)
/// - Core identity traits (name, birth date)
/// - Lifecycle state (active, merged)
/// - Component tracking (what components are attached)
/// 
/// Everything else (addresses, employment, skills, etc.) are separate components
/// that can be composed onto the person entity.
#[derive(Debug, Clone)]
pub struct Person {
    /// Unique identifier (the Entity in ECS)
    pub id: PersonId,
    
    /// Core identity - the minimal data that defines a person
    pub core_identity: CoreIdentity,
    
    /// Lifecycle state
    pub lifecycle: PersonLifecycle,
    
    /// Components attached to this person (for tracking/validation)
    pub components: HashSet<ComponentType>,
    
    /// Event sourcing version
    pub version: u64,
}

/// Core identity - the absolute minimum to identify a person
#[derive(Debug, Clone, PartialEq)]
pub struct CoreIdentity {
    /// Legal name (can change but is core identity)
    pub legal_name: PersonName,
    
    /// Birth date (immutable once set)
    pub birth_date: Option<chrono::NaiveDate>,
    
    /// Death date (immutable once set)
    pub death_date: Option<chrono::NaiveDate>,
    
    /// When this person record was created
    pub created_at: DateTime<Utc>,
    
    /// Last update to core identity
    pub updated_at: DateTime<Utc>,
}

/// Person lifecycle state
#[derive(Debug, Clone, PartialEq)]
pub enum PersonLifecycle {
    /// Normal active state
    Active,
    
    /// Temporarily deactivated
    Deactivated { reason: String, since: DateTime<Utc> },
    
    /// Merged into another person
    MergedInto { target_id: PersonId, merged_at: DateTime<Utc> },
    
    /// Deceased
    Deceased { date_of_death: chrono::NaiveDate },
}

/// Types of components that can be attached to a person
/// This is for tracking and validation, not for storing the actual data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    // Contact components
    EmailAddress,
    PhoneNumber,
    
    // Location components (from location domain)
    Address,
    
    // Professional components (from organization domain)
    Employment,
    
    // Skill components
    Skill,
    Certification,
    Education,
    
    // Social components
    SocialProfile,
    
    // Business components
    CustomerSegment,
    BehavioralData,
    
    // Preference components
    CommunicationPreferences,
    PrivacyPreferences,
    
    // Generic components
    Tag,
    CustomAttribute,
    
    // Other domains can register their component types
    External(String),
}

impl Person {
    /// Create a new person with just core identity
    pub fn new(id: PersonId, legal_name: PersonName) -> Self {
        let now = Utc::now();
        Self {
            id,
            core_identity: CoreIdentity {
                legal_name,
                birth_date: None,
                death_date: None,
                created_at: now,
                updated_at: now,
            },
            lifecycle: PersonLifecycle::Active,
            components: HashSet::new(),
            version: 0,
        }
    }

    /// Create an empty person for event replay
    pub fn empty() -> Self {
        Self {
            id: PersonId::new(),
            core_identity: CoreIdentity {
                legal_name: PersonName::new("".to_string(), "".to_string()),
                birth_date: None,
                death_date: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            lifecycle: PersonLifecycle::Active,
            components: HashSet::new(),
            version: 0,
        }
    }

    /// Check if person is active
    pub fn is_active(&self) -> bool {
        matches!(self.lifecycle, PersonLifecycle::Active)
    }

    /// Check if person has a specific component type attached
    pub fn has_component(&self, component_type: &ComponentType) -> bool {
        self.components.contains(component_type)
    }

    /// Register that a component has been attached
    pub fn register_component(&mut self, component_type: ComponentType) -> DomainResult<()> {
        if !self.is_active() {
            return Err(DomainError::ValidationError(
                "Cannot add components to inactive person".to_string()
            ));
        }
        
        self.components.insert(component_type);
        self.core_identity.updated_at = Utc::now();
        Ok(())
    }

    /// Register that a component has been removed
    pub fn unregister_component(&mut self, component_type: &ComponentType) -> DomainResult<()> {
        self.components.remove(component_type);
        self.core_identity.updated_at = Utc::now();
        Ok(())
    }

    /// Handle commands - only core identity commands
    pub fn handle_command(&mut self, command: PersonCommand) -> Result<Vec<PersonEvent>, String> {
        match self.lifecycle {
            PersonLifecycle::MergedInto { .. } => {
                return Err("Cannot modify a merged person".to_string());
            }
            PersonLifecycle::Deceased { .. } => {
                return Err("Cannot modify a deceased person".to_string());
            }
            _ => {}
        }

        match command {
            PersonCommand::CreatePerson(cmd) => self.handle_create_person(cmd),
            PersonCommand::UpdateName(cmd) => self.handle_update_name(cmd),
            PersonCommand::SetBirthDate(cmd) => self.handle_set_birth_date(cmd),
            PersonCommand::RecordDeath(cmd) => self.handle_record_death(cmd),
            PersonCommand::DeactivatePerson(cmd) => self.handle_deactivate(cmd),
            PersonCommand::ReactivatePerson(cmd) => self.handle_reactivate(cmd),
            PersonCommand::MergePersons(cmd) => self.handle_merge(cmd),
            
            // Component registration commands
            PersonCommand::RegisterComponent(cmd) => self.handle_register_component(cmd),
            PersonCommand::UnregisterComponent(cmd) => self.handle_unregister_component(cmd),
            

        }
    }

    // Command handlers for core identity only
    
    fn handle_create_person(&mut self, cmd: CreatePerson) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::PersonCreated(PersonCreated {
            person_id: cmd.person_id,
            name: cmd.name,
            source: cmd.source,
            created_at: Utc::now(),
        })])
    }

    fn handle_update_name(&mut self, cmd: UpdateName) -> Result<Vec<PersonEvent>, String> {
        if !self.is_active() {
            return Err("Cannot update inactive person".to_string());
        }
        
        Ok(vec![PersonEvent::NameUpdated(NameUpdated {
            person_id: self.id,
            old_name: self.core_identity.legal_name.clone(),
            new_name: cmd.name,
            reason: cmd.reason,
            updated_at: Utc::now(),
        })])
    }

    fn handle_set_birth_date(&mut self, cmd: SetBirthDate) -> Result<Vec<PersonEvent>, String> {
        if self.core_identity.birth_date.is_some() {
            return Err("Birth date is immutable once set".to_string());
        }
        
        Ok(vec![PersonEvent::BirthDateSet(BirthDateSet {
            person_id: self.id,
            birth_date: cmd.birth_date,
            set_at: Utc::now(),
        })])
    }

    fn handle_record_death(&mut self, cmd: RecordDeath) -> Result<Vec<PersonEvent>, String> {
        if let PersonLifecycle::Deceased { .. } = self.lifecycle {
            return Err("Person is already recorded as deceased".to_string());
        }
        
        Ok(vec![PersonEvent::DeathRecorded(DeathRecorded {
            person_id: self.id,
            date_of_death: cmd.date_of_death,
            recorded_at: Utc::now(),
        })])
    }

    fn handle_deactivate(&mut self, cmd: DeactivatePerson) -> Result<Vec<PersonEvent>, String> {
        if !self.is_active() {
            return Err("Person is not active".to_string());
        }
        
        Ok(vec![PersonEvent::PersonDeactivated(PersonDeactivated {
            person_id: self.id,
            reason: cmd.reason,
            deactivated_at: Utc::now(),
        })])
    }

    fn handle_reactivate(&mut self, cmd: ReactivatePerson) -> Result<Vec<PersonEvent>, String> {
        match self.lifecycle {
            PersonLifecycle::Deactivated { .. } => {
                Ok(vec![PersonEvent::PersonReactivated(PersonReactivated {
                    person_id: self.id,
                    reason: cmd.reason,
                    reactivated_at: Utc::now(),
                })])
            }
            _ => Err("Can only reactivate deactivated persons".to_string()),
        }
    }

    fn handle_merge(&mut self, cmd: MergePersons) -> Result<Vec<PersonEvent>, String> {
        if !self.is_active() {
            return Err("Cannot merge inactive person".to_string());
        }
        
        Ok(vec![PersonEvent::PersonMergedInto(PersonMergedInto {
            source_person_id: self.id,
            merged_into_id: cmd.target_person_id,
            reason: cmd.merge_reason,
            merged_at: Utc::now(),
        })])
    }

    fn handle_register_component(&mut self, cmd: RegisterComponent) -> Result<Vec<PersonEvent>, String> {
        if self.components.contains(&cmd.component_type) {
            return Err("Component already registered".to_string());
        }
        
        Ok(vec![PersonEvent::ComponentRegistered(ComponentRegistered {
            person_id: self.id,
            component_type: cmd.component_type,
            registered_at: Utc::now(),
        })])
    }

    fn handle_unregister_component(&mut self, cmd: UnregisterComponent) -> Result<Vec<PersonEvent>, String> {
        if !self.components.contains(&cmd.component_type) {
            return Err("Component not registered".to_string());
        }
        
        Ok(vec![PersonEvent::ComponentUnregistered(ComponentUnregistered {
            person_id: self.id,
            component_type: cmd.component_type,
            unregistered_at: Utc::now(),
        })])
    }
}

impl AggregateRoot for Person {
    type Id = PersonId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

// Command and Event structs are now in commands/mod.rs and events/mod.rs 