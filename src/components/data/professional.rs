//! Professional and employment component data structures

use super::ComponentDataTrait;
use crate::aggregate::ComponentType;
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Employment history component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentHistoryData {
    pub company: String,
    pub position: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub employment_type: EmploymentType,
    pub is_current: bool,
    pub description: Option<String>,
    pub achievements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Freelance,
    Internship,
    Temporary,
    Volunteer,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteType {
    OnSite,
    Remote,
    Hybrid,
    Unknown,
}

impl ComponentDataTrait for EmploymentHistoryData {
    fn validate(&self) -> DomainResult<()> {
        if self.company.is_empty() {
            return Err(DomainError::ValidationError("Company name cannot be empty".to_string()));
        }
        if self.position.is_empty() {
            return Err(DomainError::ValidationError("Position cannot be empty".to_string()));
        }
        if let Some(end_date) = self.end_date {
            if end_date < self.start_date {
                return Err(DomainError::ValidationError("End date cannot be before start date".to_string()));
            }
        }
        Ok(())
    }

    fn summary(&self) -> String {
        format!("{} at {}", self.position, self.company)
    }

    fn component_type(&self) -> ComponentType {
        ComponentType::Employment
    }
}

/// Professional affiliation component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionalAffiliationData {
    pub organization_name: String,
    pub membership_type: String,
    pub member_id: Option<String>,
    pub role: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub committees: Vec<String>,
    pub contributions: Vec<String>,
}

impl ComponentDataTrait for ProfessionalAffiliationData {
    fn component_type(&self) -> ComponentType {
        ComponentType::ProfessionalAffiliation
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.organization_name.trim().is_empty() {
            return Err(DomainError::ValidationError("Organization name cannot be empty".to_string()));
        }
        
        if self.membership_type.trim().is_empty() {
            return Err(DomainError::ValidationError("Membership type cannot be empty".to_string()));
        }
        
        if let Some(end) = self.end_date {
            if end < self.start_date {
                return Err(DomainError::ValidationError("End date must be after start date".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        let status = if self.is_active { " (Active)" } else { "" };
        format!("{} - {}{}", self.organization_name, self.membership_type, status)
    }
}

/// Project experience component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectExperienceData {
    pub project_name: String,
    pub description: String,
    pub role: String,
    pub company: Option<String>,
    pub client: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_ongoing: bool,
    pub team_size: Option<u32>,
    pub budget: Option<String>,
    pub technologies: Vec<String>,
    pub methodologies: Vec<String>,
    pub deliverables: Vec<String>,
    pub outcomes: Vec<String>,
    pub url: Option<String>,
}

impl ComponentDataTrait for ProjectExperienceData {
    fn component_type(&self) -> ComponentType {
        ComponentType::Project
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.project_name.trim().is_empty() {
            return Err(DomainError::ValidationError("Project name cannot be empty".to_string()));
        }
        
        if self.description.trim().is_empty() {
            return Err(DomainError::ValidationError("Description cannot be empty".to_string()));
        }
        
        if self.role.trim().is_empty() {
            return Err(DomainError::ValidationError("Role cannot be empty".to_string()));
        }
        
        if let Some(end) = self.end_date {
            if end < self.start_date {
                return Err(DomainError::ValidationError("End date must be after start date".to_string()));
            }
            
            if self.is_ongoing {
                return Err(DomainError::ValidationError("Ongoing project cannot have an end date".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        let status = if self.is_ongoing { " (Ongoing)" } else { "" };
        format!("{} - {}{}", self.project_name, self.role, status)
    }
}

/// Wrapper enum for all professional data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfessionalData {
    Employment(EmploymentData),
    Affiliation(AffiliationData),
    Project(ProjectData),
    Skills(crate::components::data::skills::SkillsData),
}

// Type aliases for clarity
pub type EmploymentData = EmploymentHistoryData;
pub type AffiliationData = ProfessionalAffiliationData;
pub type ProjectData = ProjectExperienceData;

impl ProfessionalData {
    pub fn professional_type(&self) -> &'static str {
        match self {
            ProfessionalData::Employment(_) => "Employment",
            ProfessionalData::Affiliation(_) => "Affiliation",
            ProfessionalData::Project(_) => "Project",
            ProfessionalData::Skills(_) => "Skills",
        }
    }
    
    pub fn component_type(&self) -> ComponentType {
        match self {
            ProfessionalData::Employment(data) => data.component_type(),
            ProfessionalData::Affiliation(data) => data.component_type(),
            ProfessionalData::Project(data) => data.component_type(),
            ProfessionalData::Skills(data) => data.component_type(),
        }
    }
    
    pub fn validate(&self) -> DomainResult<()> {
        match self {
            ProfessionalData::Employment(data) => data.validate(),
            ProfessionalData::Affiliation(data) => data.validate(),
            ProfessionalData::Project(data) => data.validate(),
            ProfessionalData::Skills(data) => data.validate(),
        }
    }
    
    pub fn summary(&self) -> String {
        match self {
            ProfessionalData::Employment(data) => data.summary(),
            ProfessionalData::Affiliation(data) => data.summary(),
            ProfessionalData::Project(data) => data.summary(),
            ProfessionalData::Skills(data) => data.summary(),
        }
    }
} 