//! Events for the person domain

use crate::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Events that can be emitted by the Person aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonEvent {
    /// Person was registered in the system
    PersonRegistered {
        /// Person's unique ID
        person_id: Uuid,
        /// Identity component that was registered
        identity: IdentityComponent,
        /// Contact component (if provided)
        contact: Option<ContactComponent>,
        /// When the person was registered
        registered_at: DateTime<Utc>,
    },

    /// Contact information was updated
    ContactUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old contact information
        old_contact: Option<ContactComponent>,
        /// New contact information
        new_contact: ContactComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Employment was added
    EmploymentAdded {
        /// Person's ID
        person_id: Uuid,
        /// Employment details
        employment: EmploymentComponent,
        /// When added
        added_at: DateTime<Utc>,
    },

    /// Employment status changed
    EmploymentStatusChanged {
        /// Person's ID
        person_id: Uuid,
        /// Organization ID
        organization_id: Uuid,
        /// Old status
        old_status: String,
        /// New status
        new_status: String,
        /// End date if terminated
        end_date: Option<chrono::NaiveDate>,
        /// When changed
        changed_at: DateTime<Utc>,
    },

    /// Position was added
    PositionAdded {
        /// Person's ID
        person_id: Uuid,
        /// Position details
        position: PositionComponent,
        /// When added
        added_at: DateTime<Utc>,
    },

    /// Skills were updated
    SkillsUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old skills
        old_skills: Option<SkillsComponent>,
        /// New skills
        new_skills: SkillsComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Access was granted
    AccessGranted {
        /// Person's ID
        person_id: Uuid,
        /// Access details
        access: AccessComponent,
        /// When granted
        granted_at: DateTime<Utc>,
    },

    /// External identifier was added
    ExternalIdentifierAdded {
        /// Person's ID
        person_id: Uuid,
        /// System name
        system: String,
        /// Identifier value
        identifier: String,
        /// When added
        added_at: DateTime<Utc>,
    },
}

impl PersonEvent {
    /// Get the aggregate ID this event relates to
    pub fn aggregate_id(&self) -> Uuid {
        match self {
            PersonEvent::PersonRegistered { person_id, .. } => *person_id,
            PersonEvent::ContactUpdated { person_id, .. } => *person_id,
            PersonEvent::EmploymentAdded { person_id, .. } => *person_id,
            PersonEvent::EmploymentStatusChanged { person_id, .. } => *person_id,
            PersonEvent::PositionAdded { person_id, .. } => *person_id,
            PersonEvent::SkillsUpdated { person_id, .. } => *person_id,
            PersonEvent::AccessGranted { person_id, .. } => *person_id,
            PersonEvent::ExternalIdentifierAdded { person_id, .. } => *person_id,
        }
    }

    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            PersonEvent::PersonRegistered { .. } => "PersonRegistered",
            PersonEvent::ContactUpdated { .. } => "ContactUpdated",
            PersonEvent::EmploymentAdded { .. } => "EmploymentAdded",
            PersonEvent::EmploymentStatusChanged { .. } => "EmploymentStatusChanged",
            PersonEvent::PositionAdded { .. } => "PositionAdded",
            PersonEvent::SkillsUpdated { .. } => "SkillsUpdated",
            PersonEvent::AccessGranted { .. } => "AccessGranted",
            PersonEvent::ExternalIdentifierAdded { .. } => "ExternalIdentifierAdded",
        }
    }

    /// Get the NATS subject for this event
    pub fn subject(&self) -> String {
        match self {
            PersonEvent::PersonRegistered { .. } => "people.person.registered.v1",
            PersonEvent::ContactUpdated { .. } => "people.person.contact_updated.v1",
            PersonEvent::EmploymentAdded { .. } => "people.person.employment_added.v1",
            PersonEvent::EmploymentStatusChanged { .. } => "people.person.employment_status_changed.v1",
            PersonEvent::PositionAdded { .. } => "people.person.position_added.v1",
            PersonEvent::SkillsUpdated { .. } => "people.person.skills_updated.v1",
            PersonEvent::AccessGranted { .. } => "people.person.access_granted.v1",
            PersonEvent::ExternalIdentifierAdded { .. } => "people.person.external_identifier_added.v1",
        }.to_string()
    }
}
