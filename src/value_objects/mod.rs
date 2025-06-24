//! Value objects for the Person domain
//!
//! This module contains value objects specific to the Person domain.
//! In the ECS architecture, these are minimal types focused on person identity.
//!
//! ## Architecture Notes
//! 
//! - Addresses are managed by the location domain
//! - Employment is a relationship between Person and Organization domains
//! - Skills, certifications, etc. are ECS components

use serde::{Deserialize, Serialize};
use std::fmt;
use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;

// ===== Core Identity =====

/// Person's name components
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonName {
    pub given_name: String,
    pub family_name: String,
    pub middle_names: Vec<String>,
    pub preferred_name: Option<String>,
    pub honorific: Option<String>,
    pub suffix: Option<String>,
}

impl PersonName {
    /// Create a simple name
    pub fn new(given_name: String, family_name: String) -> Self {
        Self {
            given_name,
            family_name,
            middle_names: Vec::new(),
            preferred_name: None,
            honorific: None,
            suffix: None,
        }
    }
    
    /// Get display name (preferred or full)
    pub fn display_name(&self) -> String {
        if let Some(preferred) = &self.preferred_name {
            preferred.clone()
        } else {
            format!("{} {}", self.given_name, self.family_name)
        }
    }
    
    /// Get full name with title and suffix
    pub fn full_name(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(honorific) = &self.honorific {
            parts.push(honorific.clone());
        }
        
        parts.push(self.given_name.clone());
        
        for middle in &self.middle_names {
            parts.push(middle.clone());
        }
        
        parts.push(self.family_name.clone());
        
        if let Some(suffix) = &self.suffix {
            parts.push(suffix.clone());
        }
        
        parts.join(" ")
    }
}

impl fmt::Display for PersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ===== Contact Information =====

/// Email address with verification status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    pub verified: bool,
}

impl EmailAddress {
    /// Create a new unverified email
    pub fn new(address: String) -> Self {
        Self {
            address,
            verified: false,
        }
    }
    
    /// Create a verified email
    pub fn verified(address: String) -> Self {
        Self {
            address,
            verified: true,
        }
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

/// Phone number with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhoneNumber {
    pub number: String,
    pub country_code: Option<String>,
    pub extension: Option<String>,
    pub sms_capable: bool,
}

impl PhoneNumber {
    /// Create a simple phone number
    pub fn new(number: String) -> Self {
        Self {
            number,
            country_code: None,
            extension: None,
            sms_capable: false,
        }
    }
    
    /// Create with country code
    pub fn with_country(number: String, country_code: String) -> Self {
        Self {
            number,
            country_code: Some(country_code),
            extension: None,
            sms_capable: false,
        }
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref cc) = self.country_code {
            write!(f, "+{} {}", cc, self.number)?;
        } else {
            write!(f, "{}", self.number)?;
        }
        if let Some(ref ext) = self.extension {
            write!(f, " x{}", ext)?;
        }
        Ok(())
    }
}

// ===== Enums used by components and cross-domain =====

/// Address type (used for cross-domain relationships)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressType {
    Home,
    Work,
    Billing,
    Shipping,
    Other(String),
}

/// Employment type (used for cross-domain relationships)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Consultant,
    Partner,
    Advisor,
}

// ===== Skills & Qualifications (Now managed as components) =====

/// Proficiency levels (used by skill components)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

// ===== Relationships (Now managed as cross-domain) =====

/// Relationship type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipType {
    // Professional
    Manager,
    DirectReport,
    Colleague,
    Mentor,
    Mentee,
    
    // Business
    Customer,
    Vendor,
    Partner,
    Investor,
    Advisor,
    
    // Personal
    Family,
    Friend,
    Emergency,
    
    // Social
    Follower,
    Following,
    Connection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipStatus {
    Active,
    Inactive,
    Pending,
    Blocked,
}

// ===== Social Media (Now managed as components) =====

/// Social platform enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SocialPlatform {
    LinkedIn,
    Twitter,
    Facebook,
    Instagram,
    YouTube,
    TikTok,
    GitHub,
    Other(String),
}

// ===== Customer/Business Attributes (Now managed as components) =====

/// Segment type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SegmentType {
    VIP,
    Regular,
    New,
    Churned,
    Prospect,
    Custom(String),
}

/// Value tier enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueTier {
    Platinum,
    Gold,
    Silver,
    Bronze,
}

/// Lifecycle stage enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecycleStage {
    Awareness,
    Consideration,
    Purchase,
    Retention,
    Advocacy,
}

/// Communication channel enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommunicationChannel {
    Email,
    Phone,
    SMS,
    InApp,
    Push,
    Mail,
}

/// Contact frequency enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContactFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    AsNeeded,
}

// Note: The actual data structures (Skill, Certification, Education, Relationship,
// SocialProfile, CustomerSegment, BehavioralData, CommunicationPreferences,
// PrivacyPreferences, Tag, CustomAttribute) are now ECS components in the
// components module, not value objects.
