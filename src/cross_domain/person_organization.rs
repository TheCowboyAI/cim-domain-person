//! Person-Organization cross-domain relationships
//!
//! This module manages employment relationships between persons and organizations
//! without duplicating organization domain concepts.

use crate::aggregate::PersonId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

// Placeholder types until organization domain is available
pub type OrganizationId = Uuid;
pub type DepartmentId = Uuid;
pub type Role = String;

/// Type of employment relationship
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Consultant,
    Intern,
    Temporary,
}

/// Represents an employment relationship between a person and an organization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmploymentRelationship {
    /// The employed person
    pub person_id: PersonId,
    
    /// The employing organization
    pub organization_id: OrganizationId,
    
    /// Role/position in the organization
    pub role: EmploymentRole,
    
    /// Department within the organization
    pub department_id: Option<DepartmentId>,
    
    /// Start date of employment
    pub start_date: NaiveDate,
    
    /// End date of employment (if terminated)
    pub end_date: Option<NaiveDate>,
    
    /// Type of employment
    pub employment_type: EmploymentType,
    
    /// Direct manager (another person)
    pub reporting_to: Option<PersonId>,
    
    /// Whether this is the person's primary employment
    pub is_primary: bool,
    
    /// Additional employment details
    pub metadata: EmploymentMetadata,
}

/// Employment role information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmploymentRole {
    /// Job title
    pub title: String,
    
    /// Role level (e.g., Junior, Senior, Lead, Principal)
    pub level: Option<String>,
    
    /// Role category (e.g., Engineering, Sales, Marketing)
    pub category: Option<String>,
}

/// Additional employment metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmploymentMetadata {
    /// Salary information (if applicable)
    pub compensation: Option<CompensationInfo>,
    
    /// Work location (reference to location domain)
    pub work_location_id: Option<String>, // Location ID as string since cim_domain_location is not available
    
    /// Remote work arrangement
    pub remote_work: RemoteWorkArrangement,
    
    /// Employment agreement ID
    pub agreement_id: Option<String>,
    
    /// Custom attributes
    pub custom_attributes: std::collections::HashMap<String, String>,
}

/// Compensation information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompensationInfo {
    /// Base salary
    pub base_salary: Option<Money>,
    
    /// Bonus structure
    pub bonus: Option<BonusStructure>,
    
    /// Equity compensation
    pub equity: Option<EquityCompensation>,
}

/// Money representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
}

/// Bonus structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BonusStructure {
    pub target_percentage: f32,
    pub frequency: BonusFrequency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonusFrequency {
    Annual,
    Quarterly,
    Monthly,
    ProjectBased,
}

/// Equity compensation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquityCompensation {
    pub shares: u64,
    pub vesting_schedule: String,
}

/// Remote work arrangement
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteWorkArrangement {
    OnSite,
    Remote,
    Hybrid { office_days_per_week: u8 },
}

/// Commands for managing employment relationships
#[derive(Debug, Clone)]
pub enum EmploymentCommand {
    /// Start a new employment
    StartEmployment {
        person_id: PersonId,
        organization_id: OrganizationId,
        role: EmploymentRole,
        department_id: Option<DepartmentId>,
        start_date: NaiveDate,
        employment_type: EmploymentType,
        is_primary: bool,
    },
    
    /// Update employment details
    UpdateEmployment {
        person_id: PersonId,
        organization_id: OrganizationId,
        updates: EmploymentUpdates,
    },
    
    /// End employment
    EndEmployment {
        person_id: PersonId,
        organization_id: OrganizationId,
        end_date: NaiveDate,
        reason: TerminationReason,
    },
    
    /// Change reporting manager
    ChangeManager {
        person_id: PersonId,
        organization_id: OrganizationId,
        new_manager_id: Option<PersonId>,
    },
}

/// Employment update fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentUpdates {
    pub role: Option<EmploymentRole>,
    pub department_id: Option<Option<DepartmentId>>,
    pub is_primary: Option<bool>,
    pub metadata: Option<EmploymentMetadata>,
}

/// Reason for employment termination
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminationReason {
    Resignation,
    Termination,
    Layoff,
    Retirement,
    ContractEnd,
    Other(String),
}

/// Events for employment relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmploymentEvent {
    /// Employment started
    EmploymentStarted {
        person_id: PersonId,
        organization_id: OrganizationId,
        role: EmploymentRole,
        department_id: Option<DepartmentId>,
        start_date: NaiveDate,
        employment_type: EmploymentType,
        is_primary: bool,
        started_at: DateTime<Utc>,
    },
    
    /// Employment details updated
    EmploymentUpdated {
        person_id: PersonId,
        organization_id: OrganizationId,
        updates: EmploymentUpdates,
        updated_at: DateTime<Utc>,
    },
    
    /// Employment ended
    EmploymentEnded {
        person_id: PersonId,
        organization_id: OrganizationId,
        end_date: NaiveDate,
        reason: TerminationReason,
        ended_at: DateTime<Utc>,
    },
    
    /// Manager changed
    ManagerChanged {
        person_id: PersonId,
        organization_id: OrganizationId,
        old_manager_id: Option<PersonId>,
        new_manager_id: Option<PersonId>,
        changed_at: DateTime<Utc>,
    },
}

/// Service for coordinating employment operations across domains
#[async_trait::async_trait]
pub trait EmploymentService {
    /// Get all employments for a person
    async fn get_person_employments(&self, person_id: PersonId) -> Result<Vec<EmploymentRelationship>, String>;
    
    /// Get all employees of an organization
    async fn get_organization_employees(&self, organization_id: OrganizationId) -> Result<Vec<EmploymentRelationship>, String>;
    
    /// Start new employment
    async fn start_employment(
        &self,
        person_id: PersonId,
        organization_id: OrganizationId,
        role: EmploymentRole,
        start_date: NaiveDate,
        employment_type: EmploymentType,
    ) -> Result<(), String>;
    
    /// End employment
    async fn end_employment(
        &self,
        person_id: PersonId,
        organization_id: OrganizationId,
        end_date: NaiveDate,
        reason: TerminationReason,
    ) -> Result<(), String>;
} 