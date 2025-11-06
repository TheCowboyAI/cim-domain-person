//! Value objects for the Person domain
//!
//! This module contains value objects specific to the Person domain.
//! Core identity includes comprehensive name representation.
//!
//! ## Architecture Notes
//!
//! - Addresses are managed by the location domain
//! - Employment is a relationship between Person and Organization domains
//! - Skills, certifications, etc. belong in separate domains
//!
//! ## Name Handling
//!
//! See `doc/person-names-design.md` for comprehensive design rationale.
//! Names are culturally-aware structured value objects, not simple strings.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

pub mod relationships;
pub use relationships::*;

// ===== Core Identity: Names =====

pub mod person_name;
pub use person_name::{
    PersonName, NameComponents, NamingConvention, NameDisplayPolicy,
    PersonNameBuilder, PersonTitle, TitleType
};

// ===== Attributes: Extensible EAV System =====

pub mod person_attribute;
pub use person_attribute::{
    PersonAttribute, PersonAttributeSet, AttributeType, AttributeValue,
    IdentifyingAttributeType, PhysicalAttributeType, HealthcareAttributeType,
    DemographicAttributeType, CustomAttributeType, TemporalValidity,
    Provenance, AttributeSource, ConfidenceLevel, TransformationTrace,
    DatePrecision, BloodTypeValue, EyeColorValue, HairColorValue,
    BiologicalSexValue, HandednessValue,
};

// ===== Contact Information =====

/// Email address with verification status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    pub verified: bool,
}

impl EmailAddress {
    /// Create a new unverified email
    pub fn new(address: String) -> Result<Self, String> {
        // Simple validation
        if address.contains('@') && address.contains('.') {
            Ok(Self {
                address,
                verified: false,
            })
        } else {
            Err("Invalid email format".to_string())
        }
    }
    
    /// Create a verified email
    pub fn verified(address: String) -> Self {
        Self {
            address,
            verified: true,
        }
    }
    
    /// Get the email address value
    pub fn value(&self) -> &str {
        &self.address
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
    pub fn new(number: String) -> Result<Self, String> {
        // Simple validation - check if it has digits
        if number.chars().any(|c| c.is_numeric()) {
            Ok(Self {
                number,
                country_code: None,
                extension: None,
                sms_capable: false,
            })
        } else {
            Err("Phone number must contain digits".to_string())
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
    
    /// Get the phone number value
    pub fn value(&self) -> &str {
        &self.number
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
            write!(f, " x{ext}")?;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Freelance,
    Internship,
    Volunteer,
    Other,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteType {
    OnSite,
    Remote,
    Hybrid,
    Unknown,
}


