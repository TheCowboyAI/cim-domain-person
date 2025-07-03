//! Preferences component data structures

use super::ComponentDataTrait;
use crate::aggregate::ComponentType;
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Communication preferences component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPreferencesData {
    pub preferred_language: String,
    pub secondary_languages: Vec<String>,
    pub preferred_channels: Vec<CommunicationChannel>,
    pub contact_frequency: ContactFrequency,
    pub best_time_to_contact: Option<String>,
    pub do_not_disturb_hours: Option<DoNotDisturbHours>,
    pub email_format: EmailFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationChannel {
    Email,
    Phone,
    SMS,
    WhatsApp,
    InAppNotification,
    PostalMail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    OnlyImportant,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoNotDisturbHours {
    pub start_hour: u8,
    pub end_hour: u8,
    pub timezone: String,
    pub include_weekends: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmailFormat {
    PlainText,
    HTML,
    Both,
}

impl ComponentDataTrait for CommunicationPreferencesData {
    fn component_type(&self) -> ComponentType {
        ComponentType::CommunicationPreferences
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.preferred_language.trim().is_empty() {
            return Err(DomainError::ValidationError("Preferred language cannot be empty".to_string()));
        }
        
        if let Some(dnd) = &self.do_not_disturb_hours {
            if dnd.start_hour > 23 || dnd.end_hour > 23 {
                return Err(DomainError::ValidationError("Invalid hour values".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("Language: {}, Frequency: {:?}", self.preferred_language, self.contact_frequency)
    }
}

/// Privacy preferences component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPreferencesData {
    pub data_sharing: DataSharingPreferences,
    pub visibility_settings: VisibilitySettings,
    pub consent_records: Vec<ConsentRecord>,
    pub data_retention_preference: DataRetentionPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSharingPreferences {
    pub allow_analytics: bool,
    pub allow_marketing: bool,
    pub allow_third_party_sharing: bool,
    pub allow_profiling: bool,
    pub share_with_partners: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilitySettings {
    pub profile_visibility: ProfileVisibility,
    pub contact_info_visibility: ContactInfoVisibility,
    pub activity_visibility: ActivityVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileVisibility {
    Public,
    RegisteredUsers,
    Connections,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactInfoVisibility {
    Everyone,
    Connections,
    Nobody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityVisibility {
    Public,
    Connections,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub consent_type: String,
    pub granted: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataRetentionPreference {
    MinimumRequired,
    OneYear,
    ThreeYears,
    FiveYears,
    Indefinite,
}

impl ComponentDataTrait for PrivacyPreferencesData {
    fn component_type(&self) -> ComponentType {
        ComponentType::PrivacyPreferences
    }
    
    fn validate(&self) -> DomainResult<()> {
        // Validate consent records
        for consent in &self.consent_records {
            if consent.consent_type.trim().is_empty() {
                return Err(DomainError::ValidationError("Consent type cannot be empty".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("Profile: {:?}, Data Retention: {:?}", 
            self.visibility_settings.profile_visibility,
            self.data_retention_preference
        )
    }
}

/// General preferences component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralPreferencesData {
    pub theme: ThemePreference,
    pub date_format: String,
    pub time_format: TimeFormat,
    pub currency: String,
    pub units: UnitSystem,
    pub custom_preferences: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeFormat {
    TwelveHour,
    TwentyFourHour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitSystem {
    Metric,
    Imperial,
}

impl ComponentDataTrait for GeneralPreferencesData {
    fn component_type(&self) -> ComponentType {
        ComponentType::GeneralPreferences
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.currency.len() != 3 {
            return Err(DomainError::ValidationError("Currency must be 3-letter ISO code".to_string()));
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("Theme: {:?}, Units: {:?}", 
            self.theme,
            self.units
        )
    }
}

/// Wrapper enum for all preferences data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreferencesData {
    Communication(CommunicationPreferencesData),
    Privacy(PrivacyPreferencesData),
    General(GeneralPreferencesData),
}

impl PreferencesData {
    pub fn preferences_type(&self) -> &'static str {
        match self {
            PreferencesData::Communication(_) => "Communication",
            PreferencesData::Privacy(_) => "Privacy",
            PreferencesData::General(_) => "General",
        }
    }
    
    pub fn component_type(&self) -> ComponentType {
        match self {
            PreferencesData::Communication(data) => data.component_type(),
            PreferencesData::Privacy(data) => data.component_type(),
            PreferencesData::General(data) => data.component_type(),
        }
    }
    
    pub fn validate(&self) -> DomainResult<()> {
        match self {
            PreferencesData::Communication(data) => data.validate(),
            PreferencesData::Privacy(data) => data.validate(),
            PreferencesData::General(data) => data.validate(),
        }
    }
    
    pub fn summary(&self) -> String {
        match self {
            PreferencesData::Communication(data) => data.summary(),
            PreferencesData::Privacy(data) => data.summary(),
            PreferencesData::General(data) => data.summary(),
        }
    }
} 