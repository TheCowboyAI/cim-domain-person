//! Value objects for the Person domain

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;

// ===== Basic Identity =====

/// Person's name with cultural support
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonName {
    pub given_name: String,
    pub family_name: String,
    pub middle_names: Vec<String>,
    pub preferred_name: Option<String>,
    pub title: Option<String>,           // Mr., Ms., Dr., etc.
    pub suffix: Option<String>,          // Jr., III, PhD, etc.
    pub cultural_context: Option<String>, // For name ordering/display
}

impl PersonName {
    /// Create a simple name
    pub fn new(given_name: String, family_name: String) -> Self {
        Self {
            given_name,
            family_name,
            middle_names: Vec::new(),
            preferred_name: None,
            title: None,
            suffix: None,
            cultural_context: None,
        }
    }
    
    /// Get display name (preferred or full)
    pub fn display_name(&self) -> String {
        if let Some(ref preferred) = self.preferred_name {
            preferred.clone()
        } else {
            self.full_name()
        }
    }
    
    /// Get full name with title and suffix
    pub fn full_name(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ref title) = self.title {
            parts.push(title.clone());
        }
        
        parts.push(self.given_name.clone());
        parts.extend(self.middle_names.clone());
        parts.push(self.family_name.clone());
        
        if let Some(ref suffix) = self.suffix {
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

/// Email address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Phone number
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// Physical address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicalAddress {
    pub street_lines: Vec<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
}

impl PhysicalAddress {
    /// Create a simple address
    pub fn new(street: String, city: String, country: String) -> Self {
        Self {
            street_lines: vec![street],
            city,
            state_province: None,
            postal_code: None,
            country,
        }
    }
}

impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.street_lines {
            writeln!(f, "{}", line)?;
        }
        write!(f, "{}", self.city)?;
        if let Some(ref state) = self.state_province {
            write!(f, ", {}", state)?;
        }
        if let Some(ref postal) = self.postal_code {
            write!(f, " {}", postal)?;
        }
        write!(f, ", {}", self.country)
    }
}

/// Address type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressType {
    Home,
    Work,
    Billing,
    Shipping,
    Other(String),
}

// ===== Employment & Organization =====

/// Employment information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Employment {
    pub organization_id: Uuid,
    pub organization_name: String,
    pub department: Option<String>,
    pub position: String,
    pub employment_type: EmploymentType,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub manager_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Consultant,
    Partner,
    Advisor,
}

// ===== Skills & Qualifications =====

/// Skills and expertise
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub category: String,
    pub proficiency: ProficiencyLevel,
    pub years_experience: Option<f32>,
    pub last_used: Option<NaiveDate>,
    pub certifications: Vec<Certification>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Professional certification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Certification {
    pub name: String,
    pub issuer: String,
    pub issue_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
    pub credential_id: Option<String>,
    pub verification_url: Option<String>,
}

/// Educational qualification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Education {
    pub institution: String,
    pub degree: String,
    pub field_of_study: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub grade: Option<String>,
    pub activities: Vec<String>,
}

// ===== Relationships =====

/// Relationship with another person
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub person_id: Uuid,
    pub relationship_type: RelationshipType,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub status: RelationshipStatus,
    pub notes: Option<String>,
}

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

// ===== Social Media =====

/// Social media profile
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocialProfile {
    pub platform: SocialPlatform,
    pub username: String,
    pub profile_url: Option<String>,
    pub verified: bool,
    pub follower_count: Option<u64>,
    pub following_count: Option<u64>,
    pub engagement_rate: Option<f32>,
    pub last_active: Option<DateTime<Utc>>,
}

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

// ===== Customer/Business Attributes =====

/// Customer segmentation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomerSegment {
    pub segment_type: SegmentType,
    pub value_tier: ValueTier,
    pub lifecycle_stage: LifecycleStage,
    pub persona: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SegmentType {
    VIP,
    Regular,
    New,
    Churned,
    Prospect,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueTier {
    Platinum,
    Gold,
    Silver,
    Bronze,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecycleStage {
    Awareness,
    Consideration,
    Purchase,
    Retention,
    Advocacy,
}

/// Behavioral data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehavioralData {
    pub purchase_frequency: Option<f32>,
    pub average_order_value: Option<f32>,
    pub lifetime_value: Option<f32>,
    pub churn_risk: Option<f32>,
    pub engagement_score: Option<f32>,
    pub last_interaction: Option<DateTime<Utc>>,
    pub preferred_channels: Vec<CommunicationChannel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommunicationChannel {
    Email,
    Phone,
    SMS,
    InApp,
    Push,
    Mail,
}

// ===== Preferences =====

/// Communication preferences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunicationPreferences {
    pub preferred_language: String,
    pub preferred_channel: CommunicationChannel,
    pub contact_frequency: ContactFrequency,
    pub do_not_contact: bool,
    pub timezone: String,
    pub best_time_to_contact: Option<(u8, u8)>, // (start_hour, end_hour)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContactFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    AsNeeded,
}

/// Privacy preferences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrivacyPreferences {
    pub data_sharing_consent: bool,
    pub marketing_consent: bool,
    pub cookies_consent: bool,
    pub analytics_consent: bool,
    pub third_party_sharing: bool,
    pub retention_period: Option<u32>, // days
}

// ===== Tags and Metadata =====

/// Flexible tagging system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub category: String,
    pub added_by: Uuid,
    pub added_at: DateTime<Utc>,
}

/// Custom attributes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomAttribute {
    pub name: String,
    pub value: String,
    pub data_type: String,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}
