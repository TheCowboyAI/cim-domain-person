//! Preference components for Person entities

use super::{ComponentMetadata, PersonComponent};
use crate::aggregate::person_ecs::ComponentType;
use serde::{Deserialize, Serialize};

/// Communication preferences component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunicationPreferencesComponent {
    /// Preferred language (ISO 639-1 code)
    pub language: String,
    
    /// Preferred communication channels
    pub channels: Vec<CommunicationChannel>,
    
    /// Contact frequency preference
    pub frequency: ContactFrequency,
    
    /// Do not contact flag
    pub do_not_contact: bool,
    
    /// Timezone (IANA timezone)
    pub timezone: String,
    
    /// Best time to contact (24h format)
    pub contact_hours: Option<ContactHours>,
    
    /// Component metadata
    pub metadata: ComponentMetadata,
}

impl PersonComponent for CommunicationPreferencesComponent {
    fn component_type() -> ComponentType {
        ComponentType::CommunicationPreferences
    }
}

/// Privacy preferences component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrivacyPreferencesComponent {
    /// Consent settings
    pub consents: ConsentSettings,
    
    /// Data retention preferences
    pub retention_days: Option<u32>,
    
    /// Component metadata
    pub metadata: ComponentMetadata,
}

impl PersonComponent for PrivacyPreferencesComponent {
    fn component_type() -> ComponentType {
        ComponentType::PrivacyPreferences
    }
}

/// Communication channels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationChannel {
    Email,
    Phone,
    SMS,
    InApp,
    Push,
    PostalMail,
}

/// Contact frequency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    AsNeeded,
    Never,
}

/// Contact hours preference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactHours {
    /// Start hour (0-23)
    pub start_hour: u8,
    
    /// End hour (0-23)
    pub end_hour: u8,
    
    /// Days of week when contactable
    pub days: Vec<DayOfWeek>,
}

/// Days of week
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Consent settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsentSettings {
    /// Data sharing with first parties
    pub data_sharing: bool,
    
    /// Marketing communications
    pub marketing: bool,
    
    /// Analytics and tracking
    pub analytics: bool,
    
    /// Third party sharing
    pub third_party: bool,
    
    /// Cookies and similar technologies
    pub cookies: bool,
} 