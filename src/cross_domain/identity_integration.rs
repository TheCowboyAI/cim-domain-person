//! Integration with Identity domain for organization relationships

use crate::aggregate::PersonId;
use crate::events::ComponentDataEvent;
use crate::components::data::{ComponentInstanceId, EmploymentType};
use crate::components::{ComponentStore, InMemoryComponentStore};
use crate::infrastructure::PersonRepository;
use cim_domain::{DomainResult, DomainError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Events from Identity domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityDomainEvent {
    OrganizationCreated {
        org_id: String,
        name: String,
        domain: Option<String>,
        industry: Option<String>,
        created_at: DateTime<Utc>,
    },
    PersonJoinedOrganization {
        person_id: PersonId,
        org_id: String,
        role: String,
        department: Option<String>,
        start_date: DateTime<Utc>,
        employment_type: String,
    },
    PersonLeftOrganization {
        person_id: PersonId,
        org_id: String,
        end_date: DateTime<Utc>,
        reason: Option<String>,
    },
    PersonRoleChanged {
        person_id: PersonId,
        org_id: String,
        old_role: String,
        new_role: String,
        effective_date: DateTime<Utc>,
    },
}

/// Handler for Identity domain events
pub struct IdentityEventHandler {
    person_repository: Arc<PersonRepository>,
    component_store: Arc<InMemoryComponentStore>,
}

impl IdentityEventHandler {
    pub fn new(
        person_repository: Arc<PersonRepository>,
        component_store: Arc<InMemoryComponentStore>,
    ) -> Self {
        Self {
            person_repository,
            component_store,
        }
    }
    
    /// Process an event from the Identity domain
    pub async fn handle_event(&self, event: IdentityDomainEvent) -> DomainResult<Vec<ComponentDataEvent>> {
        match event {
            IdentityDomainEvent::PersonJoinedOrganization {
                person_id,
                org_id,
                role,
                department,
                start_date,
                employment_type,
            } => {
                self.handle_person_joined_organization(
                    person_id,
                    org_id,
                    role,
                    department,
                    start_date,
                    employment_type,
                ).await
            }
            IdentityDomainEvent::PersonLeftOrganization {
                person_id,
                org_id,
                end_date,
                ..
            } => {
                self.handle_person_left_organization(person_id, org_id, end_date).await
            }
            IdentityDomainEvent::PersonRoleChanged {
                person_id,
                org_id,
                new_role,
                ..
            } => {
                self.handle_person_role_changed(person_id, org_id, new_role).await
            }
            _ => Ok(vec![]),
        }
    }
    
    async fn handle_person_joined_organization(
        &self,
        person_id: PersonId,
        org_id: String,
        role: String,
        _department: Option<String>,
        start_date: DateTime<Utc>,
        employment_type: String,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Verify person exists
        let person = self.person_repository.load(person_id).await?;
        if person.is_none() {
            return Err(DomainError::AggregateNotFound(format!("Person {person_id}")));
        }
        
        // Create employment component
        let employment_data = crate::components::data::EmploymentHistoryData {
            company: format!("Organization {org_id}"),
            position: role.clone(),
            start_date: start_date.date_naive(),
            end_date: None,
            employment_type: self.map_employment_type(&employment_type),
            is_current: true,
            description: None,
            achievements: Vec::new(),
        };
        
        let component = crate::components::data::ComponentInstance::new(person_id, employment_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        // Create event
        let event = ComponentDataEvent::EmploymentAdded {
            person_id,
            component_id,
            company_name: format!("Organization {org_id}"),
            job_title: role,
            employment_type: self.map_employment_type(&employment_type),
            start_date,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    async fn handle_person_left_organization(
        &self,
        person_id: PersonId,
        _org_id: String,
        end_date: DateTime<Utc>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Find the employment component for this organization
        let components = self.component_store.get_components_by_type(
            person_id,
            crate::aggregate::ComponentType::Employment,
        ).await?;
        
        // Find the matching employment
        for component_json in components {
            // Extract the component ID from the JSON
            if let Some(id_value) = component_json.get("id") {
                if let Ok(component_id) = serde_json::from_value::<ComponentInstanceId>(id_value.clone()) {
                    // This would need to check if the component matches the org_id
                    // For now, we'll create an employment ended event
                    return Ok(vec![ComponentDataEvent::EmploymentEnded {
                        person_id,
                        component_id,
                        end_date,
                        timestamp: Utc::now(),
                    }]);
                }
            }
        }
        
        Ok(vec![])
    }
    
    async fn handle_person_role_changed(
        &self,
        person_id: PersonId,
        _org_id: String,
        _new_role: String,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Find the employment component for this organization
        let _components = self.component_store.get_components_by_type(
            person_id,
            crate::aggregate::ComponentType::Employment,
        ).await?;
        
        // This would update the employment component with the new role
        // For simplicity, we'll return an empty vec for now
        Ok(vec![])
    }
    
    fn map_employment_type(&self, employment_type: &str) -> EmploymentType {
        match employment_type.to_lowercase().as_str() {
            "full-time" | "fulltime" => EmploymentType::FullTime,
            "part-time" | "parttime" => EmploymentType::PartTime,
            "contract" | "contractor" => EmploymentType::Contract,
            "freelance" => EmploymentType::Freelance,
            "intern" | "internship" => EmploymentType::Internship,
            _ => EmploymentType::Other,
        }
    }
}

/// Commands to send to Identity domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityDomainCommand {
    /// Request details about an organization
    GetOrganizationDetails {
        org_id: String,
        requester_id: PersonId,
    },
    
    /// Verify a person's employment at an organization
    VerifyEmployment {
        person_id: PersonId,
        org_id: String,
        as_of_date: Option<DateTime<Utc>>,
    },
    
    /// Get all organizations a person is associated with
    GetPersonOrganizations {
        person_id: PersonId,
    },
} 