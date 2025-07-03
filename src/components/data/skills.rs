//! Skills and expertise component data structures

use super::ComponentDataTrait;
use crate::aggregate::ComponentType;
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Skill component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillComponentData {
    pub name: String,
    pub category: SkillCategory,
    pub proficiency: ProficiencyLevel,
    pub years_of_experience: Option<f32>,
    pub last_used: Option<DateTime<Utc>>,
    pub verified: bool,
    pub endorsements: Vec<Endorsement>,
    pub certifications: Vec<String>,
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillCategory {
    Technical,
    Programming,
    Language,
    Management,
    Creative,
    Analytical,
    Communication,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endorsement {
    pub endorser_id: Option<crate::aggregate::PersonId>,
    pub endorser_name: String,
    pub date: DateTime<Utc>,
    pub comment: Option<String>,
}

impl ComponentDataTrait for SkillComponentData {
    fn validate(&self) -> DomainResult<()> {
        if self.name.is_empty() {
            return Err(DomainError::ValidationError("Skill name cannot be empty".to_string()));
        }
        if let Some(years) = self.years_of_experience {
            if years < 0.0 {
                return Err(DomainError::ValidationError("Years of experience cannot be negative".to_string()));
            }
        }
        Ok(())
    }

    fn summary(&self) -> String {
        format!("{} - {:?} level", self.name, self.proficiency)
    }

    fn component_type(&self) -> ComponentType {
        ComponentType::Skill
    }
}

/// Certification component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationComponentData {
    pub name: String,
    pub issuing_organization: String,
    pub credential_id: Option<String>,
    pub issue_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub verification_url: Option<String>,
    pub skills_covered: Vec<String>,
    pub certification_level: Option<String>,
}

impl ComponentDataTrait for CertificationComponentData {
    fn component_type(&self) -> ComponentType {
        ComponentType::Certification
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.name.trim().is_empty() {
            return Err(DomainError::ValidationError("Certification name cannot be empty".to_string()));
        }
        
        if self.issuing_organization.trim().is_empty() {
            return Err(DomainError::ValidationError("Issuing organization cannot be empty".to_string()));
        }
        
        if let Some(expiry) = self.expiry_date {
            if expiry < self.issue_date {
                return Err(DomainError::ValidationError("Expiry date must be after issue date".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        let status = if let Some(expiry) = self.expiry_date {
            if expiry < Utc::now() {
                " (Expired)"
            } else {
                " (Active)"
            }
        } else {
            " (No Expiry)"
        };
        
        format!("{} - {}{}", self.name, self.issuing_organization, status)
    }
}

/// Education component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationComponentData {
    pub institution: String,
    pub degree_type: DegreeType,
    pub field_of_study: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub grade: Option<String>,
    pub activities: Vec<String>,
    pub honors: Vec<String>,
    pub thesis_title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DegreeType {
    HighSchool,
    Associate,
    Bachelor,
    Master,
    Doctorate,
    Professional,
    Certificate,
    Diploma,
    Other,
}

impl ComponentDataTrait for EducationComponentData {
    fn component_type(&self) -> ComponentType {
        ComponentType::Education
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.institution.trim().is_empty() {
            return Err(DomainError::ValidationError("Institution name cannot be empty".to_string()));
        }
        
        if self.field_of_study.trim().is_empty() {
            return Err(DomainError::ValidationError("Field of study cannot be empty".to_string()));
        }
        
        if let Some(end) = self.end_date {
            if end < self.start_date {
                return Err(DomainError::ValidationError("End date must be after start date".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("{:?} in {} from {}", self.institution, self.field_of_study, self.institution)
    }
}

/// Skills data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsData {
    pub skills: Vec<Skill>,
    pub specializations: Vec<String>,
}

/// Individual skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub category: String,
    pub proficiency: String,
    pub years_experience: Option<f32>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub endorsement_count: Option<usize>,
    pub certifications: Vec<String>,
}

impl ComponentDataTrait for SkillsData {
    fn component_type(&self) -> ComponentType {
        ComponentType::Skill
    }
    
    fn validate(&self) -> DomainResult<()> {
        for skill in &self.skills {
            if skill.name.is_empty() {
                return Err(DomainError::ValidationError("Skill name cannot be empty".to_string()));
            }
            if skill.category.is_empty() {
                return Err(DomainError::ValidationError("Skill category cannot be empty".to_string()));
            }
        }
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("{} skills", self.skills.len())
    }
} 