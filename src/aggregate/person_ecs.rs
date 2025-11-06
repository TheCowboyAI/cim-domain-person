//! Person aggregate - Pure functional domain model
//!
//! The Person aggregate maintains core identity (name, birth/death dates)
//! and lifecycle state. This is a pure functional implementation following
//! Category Theory and FRP principles with no framework dependencies.

use cim_domain::{AggregateRoot, DomainError, DomainResult, EntityId};
use cim_domain::formal_domain::{
    MealyStateMachine, Aggregate, FormalDomainEntity,
    DomainConcept
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::value_objects::{PersonName, PersonAttributeSet, PersonAttribute, AttributeType};
use crate::commands::*;
use crate::events::*;
use super::person_states::{PersonState, PersonStateCommand, create_person_state_machine};

/// Marker type for Person entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonMarker;

// Implement marker traits for formal domain types
impl DomainConcept for PersonMarker {}

/// Person ID type alias
///
/// Uses EntityId<PersonMarker> which now implements FormalEntityId
/// thanks to the blanket implementation in cim-domain.
pub type PersonId = EntityId<PersonMarker>;

/// The Person aggregate - PURE domain model
///
/// **Person domain contains ONLY intrinsic person properties:**
/// - Unique ID (PersonId)
/// - Name (legal name)
/// - Birth/Death dates (lifecycle events)
/// - Physical attributes (eye color, height, weight, etc.)
/// - Lifecycle state (active, deceased, deactivated, merged)
///
/// **NOT in Person domain** (belong in other domains):
/// - Email/Phone → Contact domain (separate entities that reference PersonId)
/// - Address → Location domain (separate entity)
/// - Employment → Organization domain (relationship)
/// - Skills → Skills domain (separate aggregates)
/// - Relationships → Relationship domain (graph edges, not node properties)
///
/// This follows pure FP/DDD principles: Person contains only what IS a person,
/// not what a person HAS or what a person DOES.
///
/// # Category Theory Compliance
///
/// Person is a Coalgebra: Person → F(Person)
/// The `unfold` method expands Person into its attribute structure (PersonAttributeSet)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    /// Unique identifier
    pub id: PersonId,

    /// Core identity - name and vital dates
    pub core_identity: CoreIdentity,

    /// Extensible attribute collection (EAV pattern)
    /// Includes: birth details, physical attributes, demographics, healthcare, etc.
    pub attributes: PersonAttributeSet,

    /// Lifecycle state
    pub lifecycle: PersonLifecycle,

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

// Physical attributes removed - they belong in specialized domains:
// - Medical domain: blood type, height, weight
// - Security domain: physical descriptions for identification
// - Any physical data should reference PersonId, not be stored here

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

impl Person {
    /// Create a new person with just core identity
    ///
    /// Person domain is minimal: just ID, name, and lifecycle.
    /// Attributes can be added via commands after creation.
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
            attributes: PersonAttributeSet::empty(),
            lifecycle: PersonLifecycle::Active,
            version: 0,
        }
    }

    /// Pure functional event application - consumes self, returns new self
    ///
    /// This is the FRP version of apply_event that follows Category Theory principles.
    /// It consumes the current Person and returns a new Person with the event applied.
    pub fn apply_event_pure(self, event: &PersonEvent) -> DomainResult<Self> {
        match event {
            PersonEvent::PersonCreated(e) => self.apply_person_created_pure(e),
            PersonEvent::PersonUpdated(e) => self.apply_person_updated_pure(e),
            PersonEvent::NameUpdated(e) => self.apply_name_updated_pure(e),
            PersonEvent::BirthDateSet(e) => self.apply_birth_date_set_pure(e),
            PersonEvent::DeathRecorded(e) => self.apply_death_recorded_pure(e),
            PersonEvent::PersonDeactivated(e) => self.apply_person_deactivated_pure(e),
            PersonEvent::PersonReactivated(e) => self.apply_person_reactivated_pure(e),
            PersonEvent::PersonMergedInto(e) => self.apply_person_merged_into_pure(e),
            PersonEvent::AttributeRecorded(e) => self.apply_attribute_recorded_pure(e),
            PersonEvent::AttributeUpdated(e) => self.apply_attribute_updated_pure(e),
            PersonEvent::AttributeInvalidated(e) => self.apply_attribute_invalidated_pure(e),
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
            attributes: PersonAttributeSet::empty(),
            lifecycle: PersonLifecycle::Active,
            version: 0,
        }
    }

    /// Check if person is active
    pub fn is_active(&self) -> bool {
        matches!(self.lifecycle, PersonLifecycle::Active)
    }
    
    /// Get core identity
    pub fn core_identity(&self) -> &CoreIdentity {
        &self.core_identity
    }
    
    /// Get lifecycle state
    pub fn lifecycle(&self) -> &PersonLifecycle {
        &self.lifecycle
    }

    /// Check if person can be modified (not deceased or merged)
    pub fn can_be_modified(&self) -> Result<(), String> {
        match &self.lifecycle {
            PersonLifecycle::Deceased { .. } => {
                Err("Cannot modify a deceased person".to_string())
            }
            PersonLifecycle::MergedInto { .. } => {
                Err("Cannot modify a merged person".to_string())
            }
            _ => Ok(())
        }
    }

    // ========================================================================
    // ATTRIBUTE METHODS - Category Theory Compliance
    // ========================================================================

    /// Coalgebra unfold: Person → F(Person)
    ///
    /// Expands Person into its attribute structure (PersonAttributeSet).
    /// This is the fundamental coalgebra operation that makes Person
    /// a Category Theory compliant structure.
    pub fn unfold(&self) -> PersonAttributeSet {
        self.attributes.clone()
    }

    /// Get a specific attribute by type
    pub fn get_attribute(&self, attr_type: &AttributeType) -> Option<&PersonAttribute> {
        self.attributes.find_by_type(attr_type)
    }

    /// Get all current attributes
    pub fn get_all_attributes(&self) -> &PersonAttributeSet {
        &self.attributes
    }

    /// Temporal query: Get attributes valid on a specific date
    ///
    /// This allows querying: "What attributes were valid on 2020-01-01?"
    pub fn observe_at(&self, date: chrono::NaiveDate) -> PersonAttributeSet {
        self.attributes.valid_on(date)
    }

    /// Get currently valid attributes
    pub fn observe_now(&self) -> PersonAttributeSet {
        self.attributes.currently_valid()
    }

    /// Functor map over attributes
    ///
    /// Transforms all attributes while preserving structure.
    /// This is structure-preserving: the temporal relationships,
    /// provenance, and attribute types remain intact.
    pub fn map_attributes<F>(mut self, f: F) -> Self
    where
        F: Fn(PersonAttribute) -> PersonAttribute,
    {
        self.attributes = self.attributes.map(f);
        self
    }

    /// Get identifying attributes for disambiguation
    pub fn identifying_attributes(&self) -> PersonAttributeSet {
        self.attributes.identifying_attributes()
    }

    /// Get healthcare-relevant attributes
    pub fn healthcare_attributes(&self) -> PersonAttributeSet {
        self.attributes.healthcare_attributes()
    }

    /// Add an attribute (internal method, typically called via events)
    pub(crate) fn add_attribute(&mut self, attribute: PersonAttribute) {
        self.attributes.attributes.push(attribute);
    }
}

// ============================================================================
// FORMAL CATEGORY THEORY IMPLEMENTATIONS
// ============================================================================

// Marker trait implementation for Person
impl DomainConcept for Person {}

/// MealyStateMachine implementation - Output depends on both State and Input
///
/// This is the fundamental model for aggregates in CIM. The same command in the same
/// state can produce different events based on the command's parameters.
impl MealyStateMachine for Person {
    type State = PersonState;
    type Input = PersonCommand;
    type Output = Vec<PersonEvent>;

    fn transition(&self, current_state: PersonState, input: PersonCommand) -> PersonState {
        // Compute next state based on current state and command
        // This is a pure function - NO mutation of self

        match (&current_state, &input) {
            // Draft -> Active on creation
            (PersonState::Draft, PersonCommand::CreatePerson(_)) => PersonState::Active,

            // Active -> Suspended on deactivation
            (PersonState::Active, PersonCommand::DeactivatePerson(cmd)) => {
                PersonState::Suspended { reason: cmd.reason.clone() }
            }

            // Suspended -> Active on reactivation
            (PersonState::Suspended { .. }, PersonCommand::ReactivatePerson(_)) => {
                PersonState::Active
            }

            // Active -> Deceased on death recording
            (PersonState::Active, PersonCommand::RecordDeath(cmd)) => {
                PersonState::Deceased { date_of_death: cmd.date_of_death }
            }

            // Active -> MergedInto on merge
            (PersonState::Active, PersonCommand::MergePersons(cmd)) => {
                PersonState::MergedInto {
                    merged_into_id: cmd.target_person_id,
                    reason: cmd.merge_reason.clone(),
                }
            }

            // For commands that don't change state, return current state
            _ => current_state,
        }
    }

    fn output(&self, current_state: PersonState, input: PersonCommand) -> Vec<PersonEvent> {
        // Compute events based on current state and command
        // This is a pure function - NO mutation of self

        // Delegate to the command handlers which are now pure
        match input {
            PersonCommand::CreatePerson(cmd) => {
                vec![PersonEvent::PersonCreated(PersonCreated {
                    person_id: cmd.person_id,
                    name: cmd.name,
                    source: cmd.source,
                    created_at: Utc::now(),
                })]
            }

            PersonCommand::UpdateName(cmd) => {
                if !matches!(current_state, PersonState::Active) {
                    return vec![]; // No event if not active
                }
                vec![PersonEvent::NameUpdated(NameUpdated {
                    person_id: self.id,
                    old_name: self.core_identity.legal_name.clone(),
                    new_name: cmd.name,
                    reason: cmd.reason,
                    updated_at: Utc::now(),
                })]
            }

            PersonCommand::SetBirthDate(cmd) => {
                if self.core_identity.birth_date.is_some() {
                    return vec![]; // Birth date is immutable
                }
                vec![PersonEvent::BirthDateSet(BirthDateSet {
                    person_id: self.id,
                    birth_date: cmd.birth_date,
                    set_at: Utc::now(),
                })]
            }

            PersonCommand::RecordDeath(cmd) => {
                if matches!(self.lifecycle, PersonLifecycle::Deceased { .. }) {
                    return vec![]; // Already deceased
                }
                vec![PersonEvent::DeathRecorded(DeathRecorded {
                    person_id: self.id,
                    date_of_death: cmd.date_of_death,
                    recorded_at: Utc::now(),
                })]
            }

            PersonCommand::DeactivatePerson(cmd) => {
                if !self.is_active() {
                    return vec![]; // Not active
                }
                vec![PersonEvent::PersonDeactivated(PersonDeactivated {
                    person_id: self.id,
                    reason: cmd.reason,
                    deactivated_at: Utc::now(),
                })]
            }

            PersonCommand::ReactivatePerson(cmd) => {
                if !matches!(self.lifecycle, PersonLifecycle::Deactivated { .. }) {
                    return vec![]; // Not deactivated
                }
                vec![PersonEvent::PersonReactivated(PersonReactivated {
                    person_id: self.id,
                    reason: cmd.reason,
                    reactivated_at: Utc::now(),
                })]
            }

            PersonCommand::MergePersons(cmd) => {
                if !self.is_active() {
                    return vec![]; // Not active
                }
                vec![PersonEvent::PersonMergedInto(PersonMergedInto {
                    source_person_id: self.id,
                    merged_into_id: cmd.target_person_id,
                    merge_reason: cmd.merge_reason,
                    merged_at: Utc::now(),
                })]
            }

            PersonCommand::RecordAttribute(cmd) => {
                if !self.is_active() {
                    return vec![]; // Can only add attributes to active persons
                }
                vec![PersonEvent::AttributeRecorded(crate::events::AttributeRecorded {
                    person_id: self.id,
                    attribute: cmd.attribute,
                    recorded_at: Utc::now(),
                })]
            }

            PersonCommand::UpdateAttribute(cmd) => {
                if !self.is_active() {
                    return vec![]; // Can only update attributes for active persons
                }
                // Find the existing attribute
                if let Some(old_attr) = self.get_attribute(&cmd.attribute_type) {
                    vec![PersonEvent::AttributeUpdated(crate::events::AttributeUpdated {
                        person_id: self.id,
                        attribute_type: cmd.attribute_type,
                        old_attribute: old_attr.clone(),
                        new_attribute: cmd.new_attribute,
                        updated_at: Utc::now(),
                    })]
                } else {
                    vec![] // Attribute not found, no event
                }
            }

            PersonCommand::InvalidateAttribute(cmd) => {
                if !self.is_active() {
                    return vec![]; // Can only invalidate attributes for active persons
                }
                vec![PersonEvent::AttributeInvalidated(crate::events::AttributeInvalidated {
                    person_id: self.id,
                    attribute_type: cmd.attribute_type,
                    invalidated_at: Utc::now(),
                    reason: cmd.reason,
                })]
            }

            // Commands not yet fully implemented
            PersonCommand::ArchivePerson(_) => vec![],
        }
    }
}

/// FormalDomainEntity implementation - Required for Aggregate trait
impl FormalDomainEntity for Person {
    type Id = PersonId;

    fn id(&self) -> Self::Id {
        self.id
    }
}

/// Aggregate trait implementation - The formal Category Theory aggregate
///
/// This combines FormalDomainEntity + MealyStateMachine + additional requirements
/// to form a complete formal aggregate according to CIM domain algebra.
impl Aggregate for Person {
    type State = PersonState;
    type Command = PersonCommand;
    type Event = PersonEvent;

    fn state(&self) -> PersonState {
        PersonState::from(self.lifecycle.clone())
    }

    fn handle(self, cmd: PersonCommand) -> Result<(Self, Vec<PersonEvent>), DomainError> {
        // Get current state
        let current_state = self.state();

        // Validate state transition using state machine
        if let Some(state_cmd) = PersonStateCommand::from_person_command(&cmd) {
            let state_machine = create_person_state_machine();
            let _new_state = state_machine
                .validate_transition(&current_state, &state_cmd)
                .map_err(|e| DomainError::ValidationError(e.to_string()))?;
        }

        // Compute events using MealyStateMachine::output
        let events = MealyStateMachine::output(&self, current_state.clone(), cmd.clone());

        // Apply events to get new aggregate state
        let new_self = events.iter().try_fold(self, |person, event| {
            person.apply_event_pure(event)
        })?;

        Ok((new_self, events))
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

// ============================================================================
// EVENT SOURCED IMPLEMENTATION - Pure Functional
// ============================================================================

impl super::EventSourced for Person {
    type Event = PersonEvent;

    fn apply_event(self, event: &PersonEvent) -> DomainResult<Self> {
        self.apply_event_pure(event)
    }
}

impl Person {
    // ============================================================================
    // PURE FUNCTIONAL EVENT APPLICATION METHODS (Category Theory compliant)
    // ============================================================================

    fn apply_person_created_pure(self, event: &PersonCreated) -> DomainResult<Self> {
        Ok(Self {
            id: event.person_id,
            core_identity: CoreIdentity {
                legal_name: event.name.clone(),
                birth_date: None,
                death_date: None,
                created_at: event.created_at,
                updated_at: event.created_at,
            },
            attributes: PersonAttributeSet::empty(),
            lifecycle: PersonLifecycle::Active,
            version: self.version + 1,
        })
    }

    fn apply_name_updated_pure(self, event: &NameUpdated) -> DomainResult<Self> {
        Ok(Self {
            core_identity: CoreIdentity {
                legal_name: event.new_name.clone(),
                updated_at: event.updated_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }

    fn apply_birth_date_set_pure(self, event: &BirthDateSet) -> DomainResult<Self> {
        Ok(Self {
            core_identity: CoreIdentity {
                birth_date: Some(event.birth_date),
                updated_at: event.set_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }

    fn apply_death_recorded_pure(self, event: &DeathRecorded) -> DomainResult<Self> {
        Ok(Self {
            core_identity: CoreIdentity {
                death_date: Some(event.date_of_death),
                updated_at: event.recorded_at,
                ..self.core_identity
            },
            lifecycle: PersonLifecycle::Deceased { date_of_death: event.date_of_death },
            version: self.version + 1,
            ..self
        })
    }


    fn apply_person_deactivated_pure(self, event: &PersonDeactivated) -> DomainResult<Self> {
        Ok(Self {
            lifecycle: PersonLifecycle::Deactivated {
                reason: event.reason.clone(),
                since: event.deactivated_at,
            },
            core_identity: CoreIdentity {
                updated_at: event.deactivated_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }

    fn apply_person_reactivated_pure(self, event: &PersonReactivated) -> DomainResult<Self> {
        Ok(Self {
            lifecycle: PersonLifecycle::Active,
            core_identity: CoreIdentity {
                updated_at: event.reactivated_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }

    fn apply_person_merged_into_pure(self, event: &PersonMergedInto) -> DomainResult<Self> {
        Ok(Self {
            lifecycle: PersonLifecycle::MergedInto {
                target_id: event.merged_into_id,
                merged_at: event.merged_at,
            },
            core_identity: CoreIdentity {
                updated_at: event.merged_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }

    fn apply_person_updated_pure(self, event: &PersonUpdated) -> DomainResult<Self> {
        Ok(Self {
            core_identity: CoreIdentity {
                legal_name: event.name.clone(),
                updated_at: event.updated_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }

    // ========================================================================
    // ATTRIBUTE EVENT HANDLERS - Pure Functional
    // ========================================================================

    fn apply_attribute_recorded_pure(mut self, event: &crate::events::AttributeRecorded) -> DomainResult<Self> {
        // Add the attribute to the set
        self.attributes.attributes.push(event.attribute.clone());
        Ok(Self {
            core_identity: CoreIdentity {
                updated_at: event.recorded_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }

    fn apply_attribute_updated_pure(mut self, event: &crate::events::AttributeUpdated) -> DomainResult<Self> {
        // Find and replace the attribute
        if let Some(pos) = self.attributes.attributes.iter().position(|attr| &attr.attribute_type == &event.attribute_type) {
            self.attributes.attributes[pos] = event.new_attribute.clone();
        }
        Ok(Self {
            core_identity: CoreIdentity {
                updated_at: event.updated_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }

    fn apply_attribute_invalidated_pure(mut self, event: &crate::events::AttributeInvalidated) -> DomainResult<Self> {
        // Update the attribute's valid_until field
        if let Some(attr) = self.attributes.attributes.iter_mut().find(|attr| &attr.attribute_type == &event.attribute_type) {
            attr.temporal.valid_until = Some(event.invalidated_at.date_naive());
        }
        Ok(Self {
            core_identity: CoreIdentity {
                updated_at: event.invalidated_at,
                ..self.core_identity
            },
            version: self.version + 1,
            ..self
        })
    }
}

// Command and Event structs are now in commands/mod.rs and events/mod.rs 