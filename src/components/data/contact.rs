//! Contact component data structures

use super::ComponentDataTrait;
use crate::aggregate::ComponentType;
use crate::value_objects::{EmailAddress, PhoneNumber};
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};

/// Email component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailComponentData {
    pub email: EmailAddress,
    pub email_type: EmailType,
    pub is_preferred_contact: bool,
    pub can_receive_notifications: bool,
    pub can_receive_marketing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmailType {
    Personal,
    Work,
    School,
    Other,
}

impl ComponentDataTrait for EmailComponentData {
    fn component_type(&self) -> ComponentType {
        ComponentType::EmailAddress
    }
    
    fn validate(&self) -> DomainResult<()> {
        // Email validation is done in EmailAddress value object
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("{} ({})", self.email.value(), match self.email_type {
            EmailType::Personal => "Personal",
            EmailType::Work => "Work",
            EmailType::School => "School",
            EmailType::Other => "Other",
        })
    }
}

/// Phone component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneComponentData {
    pub phone: PhoneNumber,
    pub phone_type: PhoneType,
    pub country_code: String,
    pub is_mobile: bool,
    pub can_receive_sms: bool,
    pub can_receive_calls: bool,
    pub preferred_contact_time: Option<ContactTimePreference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhoneType {
    Mobile,
    Home,
    Work,
    Emergency,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactTimePreference {
    pub timezone: String,
    pub preferred_hours: Vec<HourRange>,
    pub preferred_days: Vec<chrono::Weekday>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HourRange {
    pub start: u8, // 0-23
    pub end: u8,   // 0-23
}

impl ComponentDataTrait for PhoneComponentData {
    fn component_type(&self) -> ComponentType {
        ComponentType::PhoneNumber
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.country_code.is_empty() {
            return Err(DomainError::ValidationError("Country code is required".to_string()));
        }
        
        if let Some(pref) = &self.preferred_contact_time {
            for range in &pref.preferred_hours {
                if range.start > 23 || range.end > 23 {
                    return Err(DomainError::ValidationError("Invalid hour range".to_string()));
                }
                if range.start >= range.end {
                    return Err(DomainError::ValidationError("Start hour must be before end hour".to_string()));
                }
            }
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("{} {} ({})", 
            self.country_code,
            self.phone.value(), 
            match self.phone_type {
                PhoneType::Mobile => "Mobile",
                PhoneType::Home => "Home",
                PhoneType::Work => "Work",
                PhoneType::Emergency => "Emergency",
                PhoneType::Other => "Other",
            }
        )
    }
}

/// Messaging app component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingComponentData {
    pub platform: MessagingPlatform,
    pub handle: String,
    pub display_name: Option<String>,
    pub is_verified: bool,
    pub preferences: MessagingPreferences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagingPlatform {
    WhatsApp,
    Telegram,
    Signal,
    WeChat,
    Line,
    Viber,
    Discord,
    Slack,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingPreferences {
    pub allow_voice_calls: bool,
    pub allow_video_calls: bool,
    pub allow_group_invites: bool,
    pub notification_enabled: bool,
}

impl ComponentDataTrait for MessagingComponentData {
    fn component_type(&self) -> ComponentType {
        ComponentType::MessagingApp
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.handle.is_empty() {
            return Err(DomainError::ValidationError("Handle cannot be empty".to_string()));
        }
        
        // Platform-specific validation
        match self.platform {
            MessagingPlatform::Discord => {
                if !self.handle.contains('#') {
                    return Err(DomainError::ValidationError("Discord handle must include discriminator".to_string()));
                }
            }
            MessagingPlatform::Telegram => {
                if !self.handle.starts_with('@') && !self.handle.starts_with('+') {
                    return Err(DomainError::ValidationError("Telegram handle must start with @ or +".to_string()));
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("{} on {:?}", self.handle, self.platform)
    }
}

/// Wrapper enum for all contact data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactData {
    Email(EmailData),
    Phone(PhoneData),
    Messaging(MessagingData),
}

// Type aliases for clarity
pub type EmailData = EmailComponentData;
pub type PhoneData = PhoneComponentData;
pub type MessagingData = MessagingComponentData;

impl ContactData {
    pub fn contact_type(&self) -> &'static str {
        match self {
            ContactData::Email(_) => "Email",
            ContactData::Phone(_) => "Phone",
            ContactData::Messaging(_) => "Messaging",
        }
    }
    
    pub fn component_type(&self) -> ComponentType {
        match self {
            ContactData::Email(data) => data.component_type(),
            ContactData::Phone(data) => data.component_type(),
            ContactData::Messaging(data) => data.component_type(),
        }
    }
    
    pub fn validate(&self) -> DomainResult<()> {
        match self {
            ContactData::Email(data) => data.validate(),
            ContactData::Phone(data) => data.validate(),
            ContactData::Messaging(data) => data.validate(),
        }
    }
    
    pub fn summary(&self) -> String {
        match self {
            ContactData::Email(data) => data.summary(),
            ContactData::Phone(data) => data.summary(),
            ContactData::Messaging(data) => data.summary(),
        }
    }
} 