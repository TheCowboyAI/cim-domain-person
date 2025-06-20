//! Value objects specific to the person domain

use cim_domain::Component;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::any::Any;
use uuid::Uuid;

// Component modules
pub mod name;
pub mod physical;
pub mod social;
pub mod behavioral;

// Re-export all component types
pub use name::{
    NameComponent, NameOrder, AlternativeNamesComponent, AlternativeName,
    AlternativeNameType, NamePeriod,
};

pub use physical::{
    PhysicalAttributesComponent, Build, VisionCorrection,
    DistinguishingMarksComponent, DistinguishingMark, MarkType,
    BiometricComponent, BiometricHash,
    MedicalIdentityComponent, BloodType,
};

pub use social::{
    RelationshipComponent, Relationship, RelationshipType, RelationshipStatus,
    SocialMediaComponent, SocialMediaProfile, SocialPlatform, PrivacySetting,
    SocialMetrics, InterestsComponent, InterestCategory, Interest, InterestProfile,
};

pub use behavioral::{
    PreferencesComponent, CommunicationPreferences, ContactChannel, ChannelSettings,
    ContactTimePreference, FrequencyPreference, ProductPreference, ContentPreferences,
    ContentType, ContentFormat, ComplexityLevel, PrivacyPreferences,
    BehavioralComponent, PurchaseBehavior, EngagementPatterns, InteractionSummary,
    PredictiveScores, SegmentationComponent, CustomerSegment, LifecycleStage, ValueTier,
};

/// Basic identity information (legacy - consider using NameComponent instead)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityComponent {
    /// Legal name
    pub legal_name: String,

    /// Preferred name (if different from legal)
    pub preferred_name: Option<String>,

    /// Date of birth
    pub date_of_birth: Option<chrono::NaiveDate>,

    /// Government ID number (SSN, etc.)
    pub government_id: Option<String>,
}

/// Contact information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactComponent {
    /// Email addresses
    pub emails: Vec<EmailAddress>,

    /// Phone numbers
    pub phones: Vec<PhoneNumber>,

    /// Physical addresses
    pub addresses: Vec<Uuid>, // References to Location aggregates
}

/// Email address with type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    /// Email address
    pub email: String,

    /// Type (work, personal, etc.)
    pub email_type: String,

    /// Is this the primary email?
    pub is_primary: bool,

    /// Is this verified?
    pub is_verified: bool,
}

/// Phone number with type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhoneNumber {
    /// Phone number (E.164 format preferred)
    pub number: String,

    /// Type (mobile, work, home, etc.)
    pub phone_type: String,

    /// Is this the primary phone?
    pub is_primary: bool,

    /// Can receive SMS?
    pub sms_capable: bool,
}

/// Employment information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmploymentComponent {
    /// Organization ID
    pub organization_id: Uuid,

    /// Employee ID within the organization
    pub employee_id: String,

    /// Job title
    pub title: String,

    /// Department
    pub department: Option<String>,

    /// Manager's person ID
    pub manager_id: Option<Uuid>,

    /// Employment status (active, terminated, on_leave, etc.)
    pub status: String,

    /// Start date
    pub start_date: chrono::NaiveDate,

    /// End date (if terminated)
    pub end_date: Option<chrono::NaiveDate>,
}

/// Position/role information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionComponent {
    /// Position ID
    pub position_id: Uuid,

    /// Position title
    pub title: String,

    /// Level/grade
    pub level: Option<String>,

    /// Responsibilities
    pub responsibilities: Vec<String>,

    /// Required skills
    pub required_skills: Vec<String>,

    /// Start date in this position
    pub start_date: chrono::NaiveDate,

    /// End date (if no longer in position)
    pub end_date: Option<chrono::NaiveDate>,
}

/// Skills and qualifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillsComponent {
    /// Skills with proficiency levels
    pub skills: HashMap<String, SkillProficiency>,

    /// Certifications
    pub certifications: Vec<Certification>,

    /// Education
    pub education: Vec<Education>,
}

/// Skill proficiency level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillProficiency {
    /// Skill name
    pub skill: String,

    /// Proficiency level (1-5, beginner/intermediate/expert, etc.)
    pub level: String,

    /// Years of experience
    pub years_experience: Option<f32>,

    /// Last used date
    pub last_used: Option<chrono::NaiveDate>,
}

/// Certification information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certification {
    /// Certification name
    pub name: String,

    /// Issuing organization
    pub issuer: String,

    /// Issue date
    pub issue_date: chrono::NaiveDate,

    /// Expiry date (if applicable)
    pub expiry_date: Option<chrono::NaiveDate>,

    /// Credential ID
    pub credential_id: Option<String>,
}

/// Education information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Education {
    /// Institution name
    pub institution: String,

    /// Degree/qualification
    pub degree: String,

    /// Field of study
    pub field_of_study: Option<String>,

    /// Start date
    pub start_date: chrono::NaiveDate,

    /// End date
    pub end_date: Option<chrono::NaiveDate>,

    /// Grade/GPA
    pub grade: Option<String>,
}

/// Access control and permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessComponent {
    /// Roles assigned to this person
    pub roles: Vec<String>,

    /// Direct permissions
    pub permissions: Vec<String>,

    /// Groups this person belongs to
    pub groups: Vec<Uuid>,

    /// Access level/clearance
    pub access_level: Option<String>,
}

/// External system identifiers (for projections)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalIdentifiersComponent {
    /// LDAP distinguished name
    pub ldap_dn: Option<String>,

    /// Active Directory SID
    pub ad_sid: Option<String>,

    /// OAuth subject identifiers
    pub oauth_subjects: HashMap<String, String>,

    /// Other system IDs
    pub external_ids: HashMap<String, String>,
}

// Component trait implementations

impl Component for IdentityComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Identity"
    }
}

impl Component for ContactComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Contact"
    }
}

impl Component for EmploymentComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Employment"
    }
}

impl Component for PositionComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Position"
    }
}

impl Component for SkillsComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Skills"
    }
}

impl Component for AccessComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Access"
    }
}

impl Component for ExternalIdentifiersComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "ExternalIdentifiers"
    }
}
