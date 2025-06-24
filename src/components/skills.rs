//! Skill and capability components for Person entities

use super::{ComponentMetadata, PersonComponent};
use crate::aggregate::person_ecs::ComponentType;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// Skill component - represents a single skill/capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillComponent {
    /// Skill identifier (e.g., "rust-programming", "project-management")
    pub skill_id: String,
    
    /// Human-readable skill name
    pub name: String,
    
    /// Skill category
    pub category: SkillCategory,
    
    /// Proficiency level
    pub proficiency: ProficiencyLevel,
    
    /// Years of experience
    pub years_experience: Option<f32>,
    
    /// Last time this skill was used
    pub last_used: Option<NaiveDate>,
    
    /// Component metadata
    pub metadata: ComponentMetadata,
}

impl PersonComponent for SkillComponent {
    fn component_type() -> ComponentType {
        ComponentType::Skill
    }
}

/// Certification component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CertificationComponent {
    /// Certification identifier
    pub certification_id: String,
    
    /// Certification name
    pub name: String,
    
    /// Issuing organization
    pub issuer: String,
    
    /// Issue date
    pub issue_date: NaiveDate,
    
    /// Expiry date (if applicable)
    pub expiry_date: Option<NaiveDate>,
    
    /// Credential ID/number
    pub credential_id: Option<String>,
    
    /// Verification URL
    pub verification_url: Option<String>,
    
    /// Component metadata
    pub metadata: ComponentMetadata,
}

impl PersonComponent for CertificationComponent {
    fn component_type() -> ComponentType {
        ComponentType::Certification
    }
}

/// Education component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EducationComponent {
    /// Education record identifier
    pub education_id: String,
    
    /// Institution name
    pub institution: String,
    
    /// Degree/qualification
    pub degree: String,
    
    /// Field of study
    pub field_of_study: Option<String>,
    
    /// Start date
    pub start_date: NaiveDate,
    
    /// End date (if completed)
    pub end_date: Option<NaiveDate>,
    
    /// Grade/GPA
    pub grade: Option<String>,
    
    /// Component metadata
    pub metadata: ComponentMetadata,
}

impl PersonComponent for EducationComponent {
    fn component_type() -> ComponentType {
        ComponentType::Education
    }
}

/// Skill categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCategory {
    Technical,
    Management,
    Communication,
    Creative,
    Analytical,
    Other(String),
}

/// Proficiency levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
} 