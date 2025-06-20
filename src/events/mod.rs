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

    /// Contact information was removed
    ContactRemoved {
        /// Person's ID
        person_id: Uuid,
        /// Old contact information
        old_contact: ContactComponent,
        /// When removed
        removed_at: DateTime<Utc>,
    },

    /// Contact information was added
    ContactAdded {
        /// Person's ID
        person_id: Uuid,
        /// New contact information
        new_contact: ContactComponent,
        /// When added
        added_at: DateTime<Utc>,
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

    /// Skills were removed
    SkillsRemoved {
        /// Person's ID
        person_id: Uuid,
        /// Old skills
        old_skills: SkillsComponent,
        /// When removed
        removed_at: DateTime<Utc>,
    },

    /// Skills were added
    SkillsAdded {
        /// Person's ID
        person_id: Uuid,
        /// New skills
        new_skills: SkillsComponent,
        /// When added
        added_at: DateTime<Utc>,
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

    // New CRM-specific events

    /// Name was updated
    NameUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old name (if any)
        old_name: Option<NameComponent>,
        /// New name
        new_name: NameComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Alternative names were added
    AlternativeNamesAdded {
        /// Person's ID
        person_id: Uuid,
        /// Alternative names
        alternative_names: AlternativeNamesComponent,
        /// When added
        added_at: DateTime<Utc>,
    },

    /// Physical attributes were updated
    PhysicalAttributesUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old attributes (if any)
        old_attributes: Option<PhysicalAttributesComponent>,
        /// New attributes
        new_attributes: PhysicalAttributesComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Distinguishing marks were added
    DistinguishingMarksAdded {
        /// Person's ID
        person_id: Uuid,
        /// Marks
        marks: DistinguishingMarksComponent,
        /// When added
        added_at: DateTime<Utc>,
    },

    /// Medical identity was updated
    MedicalIdentityUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old medical identity (if any)
        old_medical: Option<MedicalIdentityComponent>,
        /// New medical identity
        new_medical: MedicalIdentityComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Relationships were updated
    RelationshipsUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old relationships (if any)
        old_relationships: Option<RelationshipComponent>,
        /// New relationships
        new_relationships: RelationshipComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Social media was updated
    SocialMediaUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old social media (if any)
        old_social_media: Option<SocialMediaComponent>,
        /// New social media
        new_social_media: SocialMediaComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Interests were updated
    InterestsUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old interests (if any)
        old_interests: Option<InterestsComponent>,
        /// New interests
        new_interests: InterestsComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Preferences were updated
    PreferencesUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old preferences (if any)
        old_preferences: Option<PreferencesComponent>,
        /// New preferences
        new_preferences: PreferencesComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Behavioral data was updated
    BehavioralDataUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old behavioral data (if any)
        old_behavioral: Option<BehavioralComponent>,
        /// New behavioral data
        new_behavioral: BehavioralComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Segmentation was updated
    SegmentationUpdated {
        /// Person's ID
        person_id: Uuid,
        /// Old segmentation (if any)
        old_segmentation: Option<SegmentationComponent>,
        /// New segmentation
        new_segmentation: SegmentationComponent,
        /// When updated
        updated_at: DateTime<Utc>,
    },

    /// Biometric data was added
    BiometricDataAdded {
        /// Person's ID
        person_id: Uuid,
        /// Biometric data
        biometric: BiometricComponent,
        /// When added
        added_at: DateTime<Utc>,
    },

    /// Generic component was added
    ComponentAdded {
        /// Person's ID
        person_id: Uuid,
        /// Component type
        component_type: String,
        /// Component data
        component_data: serde_json::Value,
        /// Who added it
        added_by: String,
        /// Reason
        reason: Option<String>,
        /// When added
        added_at: DateTime<Utc>,
    },

    /// Component was removed
    ComponentRemoved {
        /// Person's ID
        person_id: Uuid,
        /// Component type
        component_type: String,
        /// Who removed it
        removed_by: String,
        /// Reason
        reason: Option<String>,
        /// When removed
        removed_at: DateTime<Utc>,
    },
}

impl PersonEvent {
    /// Get the aggregate ID this event relates to
    pub fn aggregate_id(&self) -> Uuid {
        match self {
            PersonEvent::PersonRegistered { person_id, .. } => *person_id,
            PersonEvent::ContactRemoved { person_id, .. } => *person_id,
            PersonEvent::ContactAdded { person_id, .. } => *person_id,
            PersonEvent::EmploymentAdded { person_id, .. } => *person_id,
            PersonEvent::EmploymentStatusChanged { person_id, .. } => *person_id,
            PersonEvent::PositionAdded { person_id, .. } => *person_id,
            PersonEvent::SkillsRemoved { person_id, .. } => *person_id,
            PersonEvent::SkillsAdded { person_id, .. } => *person_id,
            PersonEvent::AccessGranted { person_id, .. } => *person_id,
            PersonEvent::ExternalIdentifierAdded { person_id, .. } => *person_id,
            PersonEvent::NameUpdated { person_id, .. } => *person_id,
            PersonEvent::AlternativeNamesAdded { person_id, .. } => *person_id,
            PersonEvent::PhysicalAttributesUpdated { person_id, .. } => *person_id,
            PersonEvent::DistinguishingMarksAdded { person_id, .. } => *person_id,
            PersonEvent::MedicalIdentityUpdated { person_id, .. } => *person_id,
            PersonEvent::RelationshipsUpdated { person_id, .. } => *person_id,
            PersonEvent::SocialMediaUpdated { person_id, .. } => *person_id,
            PersonEvent::InterestsUpdated { person_id, .. } => *person_id,
            PersonEvent::PreferencesUpdated { person_id, .. } => *person_id,
            PersonEvent::BehavioralDataUpdated { person_id, .. } => *person_id,
            PersonEvent::SegmentationUpdated { person_id, .. } => *person_id,
            PersonEvent::BiometricDataAdded { person_id, .. } => *person_id,
            PersonEvent::ComponentAdded { person_id, .. } => *person_id,
            PersonEvent::ComponentRemoved { person_id, .. } => *person_id,
        }
    }

    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            PersonEvent::PersonRegistered { .. } => "PersonRegistered",
            PersonEvent::ContactRemoved { .. } => "ContactRemoved",
            PersonEvent::ContactAdded { .. } => "ContactAdded",
            PersonEvent::EmploymentAdded { .. } => "EmploymentAdded",
            PersonEvent::EmploymentStatusChanged { .. } => "EmploymentStatusChanged",
            PersonEvent::PositionAdded { .. } => "PositionAdded",
            PersonEvent::SkillsRemoved { .. } => "SkillsRemoved",
            PersonEvent::SkillsAdded { .. } => "SkillsAdded",
            PersonEvent::AccessGranted { .. } => "AccessGranted",
            PersonEvent::ExternalIdentifierAdded { .. } => "ExternalIdentifierAdded",
            PersonEvent::NameUpdated { .. } => "NameUpdated",
            PersonEvent::AlternativeNamesAdded { .. } => "AlternativeNamesAdded",
            PersonEvent::PhysicalAttributesUpdated { .. } => "PhysicalAttributesUpdated",
            PersonEvent::DistinguishingMarksAdded { .. } => "DistinguishingMarksAdded",
            PersonEvent::MedicalIdentityUpdated { .. } => "MedicalIdentityUpdated",
            PersonEvent::RelationshipsUpdated { .. } => "RelationshipsUpdated",
            PersonEvent::SocialMediaUpdated { .. } => "SocialMediaUpdated",
            PersonEvent::InterestsUpdated { .. } => "InterestsUpdated",
            PersonEvent::PreferencesUpdated { .. } => "PreferencesUpdated",
            PersonEvent::BehavioralDataUpdated { .. } => "BehavioralDataUpdated",
            PersonEvent::SegmentationUpdated { .. } => "SegmentationUpdated",
            PersonEvent::BiometricDataAdded { .. } => "BiometricDataAdded",
            PersonEvent::ComponentAdded { .. } => "ComponentAdded",
            PersonEvent::ComponentRemoved { .. } => "ComponentRemoved",
        }
    }

    /// Get the NATS subject for this event
    pub fn subject(&self) -> String {
        match self {
            PersonEvent::PersonRegistered { .. } => "people.person.registered.v1",
            PersonEvent::ContactRemoved { .. } => "people.person.contact_removed.v1",
            PersonEvent::ContactAdded { .. } => "people.person.contact_added.v1",
            PersonEvent::EmploymentAdded { .. } => "people.person.employment_added.v1",
            PersonEvent::EmploymentStatusChanged { .. } => "people.person.employment_status_changed.v1",
            PersonEvent::PositionAdded { .. } => "people.person.position_added.v1",
            PersonEvent::SkillsRemoved { .. } => "people.person.skills_removed.v1",
            PersonEvent::SkillsAdded { .. } => "people.person.skills_added.v1",
            PersonEvent::AccessGranted { .. } => "people.person.access_granted.v1",
            PersonEvent::ExternalIdentifierAdded { .. } => "people.person.external_identifier_added.v1",
            PersonEvent::NameUpdated { .. } => "people.person.name_updated.v1",
            PersonEvent::AlternativeNamesAdded { .. } => "people.person.alternative_names_added.v1",
            PersonEvent::PhysicalAttributesUpdated { .. } => "people.person.physical_attributes_updated.v1",
            PersonEvent::DistinguishingMarksAdded { .. } => "people.person.distinguishing_marks_added.v1",
            PersonEvent::MedicalIdentityUpdated { .. } => "people.person.medical_identity_updated.v1",
            PersonEvent::RelationshipsUpdated { .. } => "people.person.relationships_updated.v1",
            PersonEvent::SocialMediaUpdated { .. } => "people.person.social_media_updated.v1",
            PersonEvent::InterestsUpdated { .. } => "people.person.interests_updated.v1",
            PersonEvent::PreferencesUpdated { .. } => "people.person.preferences_updated.v1",
            PersonEvent::BehavioralDataUpdated { .. } => "people.person.behavioral_data_updated.v1",
            PersonEvent::SegmentationUpdated { .. } => "people.person.segmentation_updated.v1",
            PersonEvent::BiometricDataAdded { .. } => "people.person.biometric_data_added.v1",
            PersonEvent::ComponentAdded { .. } => "people.person.component_added.v1",
            PersonEvent::ComponentRemoved { .. } => "people.person.component_removed.v1",
        }.to_string()
    }
}
