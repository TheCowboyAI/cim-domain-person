//! Component management commands with actual data

use crate::aggregate::PersonId;
use crate::components::data::{ComponentInstanceId};
use serde::{Deserialize, Serialize};

/// Commands for managing component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentCommand {
    // Email component commands
    AddEmail {
        person_id: PersonId,
        email: String,
        email_type: crate::components::data::EmailType,
        is_preferred: bool,
        can_receive_notifications: bool,
        can_receive_marketing: bool,
    },
    UpdateEmail {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        email: Option<String>,
        email_type: Option<crate::components::data::EmailType>,
        is_preferred: Option<bool>,
        can_receive_notifications: Option<bool>,
        can_receive_marketing: Option<bool>,
    },
    RemoveEmail {
        person_id: PersonId,
        component_id: ComponentInstanceId,
    },
    
    // Phone component commands
    AddPhone {
        person_id: PersonId,
        phone_number: String,
        phone_type: crate::components::data::PhoneType,
        country_code: String,
        is_mobile: bool,
        can_receive_sms: bool,
        can_receive_calls: bool,
    },
    UpdatePhone {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        phone_number: Option<String>,
        phone_type: Option<crate::components::data::PhoneType>,
        can_receive_sms: Option<bool>,
        can_receive_calls: Option<bool>,
    },
    RemovePhone {
        person_id: PersonId,
        component_id: ComponentInstanceId,
    },
    
    // Skill component commands
    AddSkill {
        person_id: PersonId,
        skill_name: String,
        category: crate::components::data::SkillCategory,
        proficiency: crate::components::data::ProficiencyLevel,
        years_of_experience: Option<f32>,
    },
    UpdateSkill {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        proficiency: Option<crate::components::data::ProficiencyLevel>,
        years_of_experience: Option<f32>,
        last_used: Option<chrono::DateTime<chrono::Utc>>,
    },
    EndorseSkill {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        endorser_id: Option<PersonId>,
        endorser_name: String,
        comment: Option<String>,
    },
    RemoveSkill {
        person_id: PersonId,
        component_id: ComponentInstanceId,
    },
    
    // Social profile commands
    AddSocialProfile {
        person_id: PersonId,
        platform: crate::components::data::SocialPlatform,
        username: String,
        profile_url: String,
        display_name: Option<String>,
        bio: Option<String>,
    },
    UpdateSocialProfile {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        username: Option<String>,
        profile_url: Option<String>,
        display_name: Option<String>,
        bio: Option<String>,
        is_verified: Option<bool>,
    },
    RemoveSocialProfile {
        person_id: PersonId,
        component_id: ComponentInstanceId,
    },
    
    // Employment commands
    AddEmployment {
        person_id: PersonId,
        company_name: String,
        company_id: Option<String>,
        job_title: String,
        department: Option<String>,
        employment_type: crate::components::data::EmploymentType,
        start_date: chrono::DateTime<chrono::Utc>,
        is_current: bool,
        location: Option<String>,
        remote_type: crate::components::data::RemoteType,
    },
    UpdateEmployment {
        person_id: PersonId,
        component_id: ComponentInstanceId,
        job_title: Option<String>,
        department: Option<String>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        is_current: Option<bool>,
        responsibilities: Option<Vec<String>>,
        achievements: Option<Vec<String>>,
    },
    RemoveEmployment {
        person_id: PersonId,
        component_id: ComponentInstanceId,
    },
    
    // Preference commands
    UpdateCommunicationPreferences {
        person_id: PersonId,
        preferred_language: Option<String>,
        preferred_channels: Option<Vec<crate::components::data::CommunicationChannel>>,
        contact_frequency: Option<crate::components::data::ContactFrequency>,
        email_format: Option<crate::components::data::EmailFormat>,
    },
    UpdatePrivacyPreferences {
        person_id: PersonId,
        allow_analytics: Option<bool>,
        allow_marketing: Option<bool>,
        allow_third_party_sharing: Option<bool>,
        profile_visibility: Option<crate::components::data::ProfileVisibility>,
        data_retention_preference: Option<crate::components::data::DataRetentionPreference>,
    },
    RecordConsent {
        person_id: PersonId,
        consent_type: String,
        granted: bool,
        version: String,
        ip_address: Option<String>,
    },
} 