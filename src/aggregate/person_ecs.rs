//! ECS-oriented Person aggregate
//! 
//! In ECS architecture, entities are just IDs with components attached.
//! The Person aggregate only maintains core identity and invariants.

use cim_domain::{AggregateRoot, DomainError, DomainResult, EntityId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chrono::{DateTime, Utc};
use std::fmt;

use crate::value_objects::PersonName;
use crate::commands::*;
use crate::events::*;
use super::person_states::{PersonState, PersonStateCommand, create_person_state_machine};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    // Contact components
    EmailAddress,
    PhoneNumber,
    MessagingApp,
    
    // Location components (from location domain)
    Address,
    
    // Professional components (from organization domain)
    Employment,
    ProfessionalAffiliation,
    Project,
    
    // Skill components
    Skill,
    Certification,
    Education,
    
    // Social components
    SocialProfile,
    Website,
    ProfessionalNetwork,
    Relationship,
    
    // Business components
    CustomerSegment,
    BehavioralData,
    
    // Preference components
    CommunicationPreferences,
    PrivacyPreferences,
    GeneralPreferences,
    
    // Generic components
    Tag,
    CustomAttribute,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentType::EmailAddress => write!(f, "EmailAddress"),
            ComponentType::PhoneNumber => write!(f, "PhoneNumber"),
            ComponentType::MessagingApp => write!(f, "MessagingApp"),
            ComponentType::Address => write!(f, "Address"),
            ComponentType::Employment => write!(f, "Employment"),
            ComponentType::ProfessionalAffiliation => write!(f, "ProfessionalAffiliation"),
            ComponentType::Project => write!(f, "Project"),
            ComponentType::Skill => write!(f, "Skill"),
            ComponentType::Certification => write!(f, "Certification"),
            ComponentType::Education => write!(f, "Education"),
            ComponentType::SocialProfile => write!(f, "SocialProfile"),
            ComponentType::Website => write!(f, "Website"),
            ComponentType::ProfessionalNetwork => write!(f, "ProfessionalNetwork"),
            ComponentType::Relationship => write!(f, "Relationship"),
            ComponentType::CustomerSegment => write!(f, "CustomerSegment"),
            ComponentType::BehavioralData => write!(f, "BehavioralData"),
            ComponentType::CommunicationPreferences => write!(f, "CommunicationPreferences"),
            ComponentType::PrivacyPreferences => write!(f, "PrivacyPreferences"),
            ComponentType::GeneralPreferences => write!(f, "GeneralPreferences"),
            ComponentType::Tag => write!(f, "Tag"),
            ComponentType::CustomAttribute => write!(f, "CustomAttribute"),
        }
    }
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
        // First check state machine if this command affects state
        if let Some(state_command) = PersonStateCommand::from_person_command(&command) {
            let current_state = PersonState::from(self.lifecycle.clone());
            let state_machine = create_person_state_machine();
            
            // Validate state transition
            match state_machine.validate_transition(&current_state, &state_command) {
                Ok(new_state) => {
                    // State transition is valid, proceed
                    tracing::debug!("State transition: {:?} -> {:?}", current_state, new_state);
                }
                Err(e) => {
                    return Err(format!("State transition failed: {}", e));
                }
            }
        }

        let events = match command {
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
            
            // New commands that don't have handlers yet
            PersonCommand::ArchivePerson(_) => {
                // TODO: Implement archive handler
                Ok(vec![])
            },
            PersonCommand::AddComponent(_) => {
                // Component addition is handled by component store, not aggregate
                Ok(vec![])
            },
            PersonCommand::UpdateComponent(_) => {
                // Component updates are handled by component store, not aggregate
                Ok(vec![])
            },
        }?;
        
        // Apply the events to update state
        use super::EventSourced;
        for event in &events {
            self.apply_event(event).map_err(|e| e.to_string())?;
        }
        
        Ok(events)
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
        
        let merge_event = PersonEvent::PersonMergedInto(PersonMergedInto {
            source_person_id: self.id,
            merged_into_id: cmd.target_person_id,
            merge_reason: cmd.merge_reason,
            merged_at: Utc::now(),
        });
        
        Ok(vec![merge_event])
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

// EventSourced implementation
impl super::EventSourced for Person {
    type Event = PersonEvent;
    
    fn apply_event(&mut self, event: &PersonEvent) -> DomainResult<()> {
        match event {
            PersonEvent::PersonCreated(e) => self.apply_person_created(e),
            PersonEvent::PersonUpdated(e) => self.apply_person_updated(e),
            PersonEvent::NameUpdated(e) => self.apply_name_updated(e),
            PersonEvent::BirthDateSet(e) => self.apply_birth_date_set(e),
            PersonEvent::DeathRecorded(e) => self.apply_death_recorded(e),
            PersonEvent::ComponentRegistered(e) => self.apply_component_registered(e),
            PersonEvent::ComponentUnregistered(e) => self.apply_component_unregistered(e),
            PersonEvent::PersonDeactivated(e) => self.apply_person_deactivated(e),
            PersonEvent::PersonReactivated(e) => self.apply_person_reactivated(e),
            PersonEvent::PersonMergedInto(e) => self.apply_person_merged_into(e),
            PersonEvent::ComponentDataUpdated(e) => self.apply_component_data_updated(e),
        }
    }
}

// Event application methods
impl Person {
    fn apply_person_created(&mut self, event: &PersonCreated) -> DomainResult<()> {
        self.id = event.person_id;
        self.core_identity.legal_name = event.name.clone();
        self.core_identity.created_at = event.created_at;
        self.core_identity.updated_at = event.created_at;
        self.lifecycle = PersonLifecycle::Active;
        self.increment_version();
        Ok(())
    }
    
    fn apply_name_updated(&mut self, event: &NameUpdated) -> DomainResult<()> {
        self.core_identity.legal_name = event.new_name.clone();
        self.core_identity.updated_at = event.updated_at;
        self.increment_version();
        Ok(())
    }
    
    fn apply_birth_date_set(&mut self, event: &BirthDateSet) -> DomainResult<()> {
        self.core_identity.birth_date = Some(event.birth_date);
        self.core_identity.updated_at = event.set_at;
        self.increment_version();
        Ok(())
    }
    
    fn apply_death_recorded(&mut self, event: &DeathRecorded) -> DomainResult<()> {
        self.core_identity.death_date = Some(event.date_of_death);
        self.lifecycle = PersonLifecycle::Deceased { date_of_death: event.date_of_death };
        self.core_identity.updated_at = event.recorded_at;
        self.increment_version();
        Ok(())
    }
    
    fn apply_component_registered(&mut self, event: &ComponentRegistered) -> DomainResult<()> {
        self.components.insert(event.component_type);
        self.core_identity.updated_at = event.registered_at;
        self.increment_version();
        Ok(())
    }
    
    fn apply_component_unregistered(&mut self, event: &ComponentUnregistered) -> DomainResult<()> {
        self.components.remove(&event.component_type);
        self.core_identity.updated_at = event.unregistered_at;
        self.increment_version();
        Ok(())
    }
    
    fn apply_person_deactivated(&mut self, event: &PersonDeactivated) -> DomainResult<()> {
        self.lifecycle = PersonLifecycle::Deactivated {
            reason: event.reason.clone(),
            since: event.deactivated_at,
        };
        self.core_identity.updated_at = event.deactivated_at;
        self.increment_version();
        Ok(())
    }
    
    fn apply_person_reactivated(&mut self, event: &PersonReactivated) -> DomainResult<()> {
        self.lifecycle = PersonLifecycle::Active;
        self.core_identity.updated_at = event.reactivated_at;
        self.increment_version();
        Ok(())
    }
    
    fn apply_person_merged_into(&mut self, event: &PersonMergedInto) -> DomainResult<()> {
        self.lifecycle = PersonLifecycle::MergedInto {
            target_id: event.merged_into_id,
            merged_at: event.merged_at,
        };
        self.core_identity.updated_at = event.merged_at;
        self.increment_version();
        Ok(())
    }

    fn apply_component_data_updated(&mut self, _event: &ComponentDataUpdated) -> DomainResult<()> {
        // Component data is managed by the component store, not the aggregate
        // This event is for audit/notification purposes only
        Ok(())
    }

    fn apply_person_updated(&mut self, event: &PersonUpdated) -> DomainResult<()> {
        self.core_identity.legal_name = event.name.clone();
        self.core_identity.updated_at = event.updated_at;
        self.increment_version();
        Ok(())
    }
}

// Command and Event structs are now in commands/mod.rs and events/mod.rs 