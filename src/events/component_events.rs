//! Component data events

use crate::aggregate::PersonId;
use crate::components::data::{ComponentInstanceId};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid;

/// Events for component data management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentDataEvent {
    // Email events
    EmailAdded {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        email: String,
        email_type: crate::components::data::EmailType,
        is_preferred: bool,
        timestamp: DateTime<Utc>,
    },
    EmailUpdated {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        changes: EmailChanges,
        timestamp: DateTime<Utc>,
    },
    EmailRemoved {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        timestamp: DateTime<Utc>,
    },
    
    // Phone events
    PhoneAdded {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        phone_number: String,
        phone_type: crate::components::data::PhoneType,
        country_code: String,
        is_mobile: bool,
        timestamp: DateTime<Utc>,
    },
    PhoneUpdated {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        changes: PhoneChanges,
        timestamp: DateTime<Utc>,
    },
    PhoneRemoved {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        timestamp: DateTime<Utc>,
    },
    
    // Skill events
    SkillAdded {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        skill_name: String,
        category: crate::components::data::SkillCategory,
        proficiency: crate::components::data::ProficiencyLevel,
        timestamp: DateTime<Utc>,
    },
    SkillUpdated {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        changes: SkillChanges,
        timestamp: DateTime<Utc>,
    },
    SkillEndorsed {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        endorser_id: Option<PersonId>,
        endorser_name: String,
        comment: Option<String>,
        timestamp: DateTime<Utc>,
    },
    SkillRemoved {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        timestamp: DateTime<Utc>,
    },
    
    // Social profile events
    SocialProfileAdded {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        platform: crate::components::data::SocialPlatform,
        username: String,
        profile_url: String,
        timestamp: DateTime<Utc>,
    },
    SocialProfileUpdated {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        changes: SocialProfileChanges,
        timestamp: DateTime<Utc>,
    },
    SocialProfileRemoved {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        timestamp: DateTime<Utc>,
    },
    
    // Employment events
    EmploymentAdded {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        company_name: String,
        job_title: String,
        employment_type: crate::components::data::EmploymentType,
        start_date: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },
    EmploymentUpdated {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        changes: EmploymentChanges,
        timestamp: DateTime<Utc>,
    },
    EmploymentEnded {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        end_date: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },
    EmploymentRemoved {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        timestamp: DateTime<Utc>,
    },
    
    // Preference events
    CommunicationPreferencesUpdated {
        person_id: PersonId,
        changes: CommunicationPreferenceChanges,
        timestamp: DateTime<Utc>,
    },
    PrivacyPreferencesUpdated {
        person_id: PersonId,
        changes: PrivacyPreferenceChanges,
        timestamp: DateTime<Utc>,
    },
    ConsentRecorded {
        person_id: PersonId,
        consent_type: String,
        granted: bool,
        version: String,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

// Change tracking structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailChanges {
    pub email: Option<String>,
    pub email_type: Option<crate::components::data::EmailType>,
    pub is_preferred: Option<bool>,
    pub can_receive_notifications: Option<bool>,
    pub can_receive_marketing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneChanges {
    pub phone_number: Option<String>,
    pub phone_type: Option<crate::components::data::PhoneType>,
    pub can_receive_sms: Option<bool>,
    pub can_receive_calls: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillChanges {
    pub proficiency: Option<crate::components::data::ProficiencyLevel>,
    pub years_of_experience: Option<f32>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfileChanges {
    pub username: Option<String>,
    pub profile_url: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub is_verified: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentChanges {
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub is_current: Option<bool>,
    pub responsibilities: Option<Vec<String>>,
    pub achievements: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPreferenceChanges {
    pub preferred_language: Option<String>,
    pub preferred_channels: Option<Vec<crate::components::data::CommunicationChannel>>,
    pub contact_frequency: Option<crate::components::data::ContactFrequency>,
    pub email_format: Option<crate::components::data::EmailFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPreferenceChanges {
    pub allow_analytics: Option<bool>,
    pub allow_marketing: Option<bool>,
    pub allow_third_party_sharing: Option<bool>,
    pub profile_visibility: Option<crate::components::data::ProfileVisibility>,
    pub data_retention_preference: Option<crate::components::data::DataRetentionPreference>,
}

/// Component data was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDataUpdated {
    pub person_id: PersonId,
    pub component_id: uuid::Uuid,
    pub data: crate::components::data::ComponentData,
    pub updated_at: DateTime<Utc>,
} 