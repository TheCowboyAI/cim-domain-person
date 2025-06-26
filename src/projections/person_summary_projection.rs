//! Person summary projection providing a quick overview of person data

use super::{PersonProjection, PersonSummary};
use crate::aggregate::PersonId;
use crate::events::*;
use crate::components::data::{ComponentData, ContactData, ProfessionalData};
use cim_domain::DomainResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Projection that maintains person summaries for quick access
pub struct PersonSummaryProjection {
    summaries: Arc<RwLock<HashMap<PersonId, PersonSummary>>>,
}

impl PersonSummaryProjection {
    pub fn new() -> Self {
        Self {
            summaries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get a person's summary
    pub async fn get_summary(&self, person_id: &PersonId) -> Option<PersonSummary> {
        let summaries = self.summaries.read().await;
        summaries.get(person_id).cloned()
    }
    
    /// Get all summaries
    pub async fn get_all_summaries(&self) -> Vec<PersonSummary> {
        let summaries = self.summaries.read().await;
        summaries.values().cloned().collect()
    }
    
    /// Get summaries for multiple persons
    pub async fn get_summaries(&self, person_ids: &[PersonId]) -> Vec<PersonSummary> {
        let summaries = self.summaries.read().await;
        person_ids.iter()
            .filter_map(|id| summaries.get(id).cloned())
            .collect()
    }
    
    /// Search summaries by name
    pub async fn search_by_name(&self, query: &str) -> Vec<PersonSummary> {
        let summaries = self.summaries.read().await;
        let query_lower = query.to_lowercase();
        
        summaries.values()
            .filter(|s| s.name.to_lowercase().contains(&query_lower))
            .cloned()
            .collect()
    }
    
    /// Get summaries by employer
    pub async fn get_by_employer(&self, employer: &str) -> Vec<PersonSummary> {
        let summaries = self.summaries.read().await;
        let employer_lower = employer.to_lowercase();
        
        summaries.values()
            .filter(|s| {
                s.current_employer
                    .as_ref()
                    .map(|e| e.to_lowercase().contains(&employer_lower))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}

#[async_trait::async_trait]
impl PersonProjection for PersonSummaryProjection {
    async fn handle_event(&self, event: &PersonEvent) -> DomainResult<()> {
        match event {
            PersonEvent::PersonCreated(e) => {
                let summary = PersonSummary {
                    person_id: e.person_id,
                    name: e.name.display_name(),
                    primary_email: None,
                    primary_phone: None,
                    current_employer: None,
                    current_role: None,
                    location: None,
                    skills_count: 0,
                    component_count: 0,
                    last_updated: e.created_at,
                };
                
                let mut summaries = self.summaries.write().await;
                summaries.insert(e.person_id, summary);
            }
            
            PersonEvent::NameUpdated(e) => {
                let mut summaries = self.summaries.write().await;
                if let Some(summary) = summaries.get_mut(&e.person_id) {
                    summary.name = e.new_name.display_name();
                    summary.last_updated = e.updated_at;
                }
            }
            
            PersonEvent::ComponentRegistered(e) => {
                let mut summaries = self.summaries.write().await;
                if let Some(summary) = summaries.get_mut(&e.person_id) {
                    summary.component_count += 1;
                    summary.last_updated = e.registered_at;
                }
            }
            
            PersonEvent::ComponentUnregistered(e) => {
                let mut summaries = self.summaries.write().await;
                if let Some(summary) = summaries.get_mut(&e.person_id) {
                    summary.component_count = summary.component_count.saturating_sub(1);
                    summary.last_updated = e.unregistered_at;
                }
            }
            
            PersonEvent::ComponentDataUpdated(e) => {
                let mut summaries = self.summaries.write().await;
                if let Some(summary) = summaries.get_mut(&e.person_id) {
                    // Update summary based on component data
                    match &e.data {
                        ComponentData::Contact(contact) => {
                            match contact {
                                ContactData::Email(email) if email.is_preferred_contact => {
                                    summary.primary_email = Some(email.email.to_string());
                                }
                                ContactData::Phone(phone) if phone.can_receive_calls => {
                                    summary.primary_phone = Some(phone.phone.to_string());
                                }
                                _ => {}
                            }
                        }
                        ComponentData::Professional(prof) => {
                            match prof {
                                ProfessionalData::Employment(emp) if emp.is_current => {
                                    summary.current_employer = Some(emp.company.clone());
                                    summary.current_role = Some(emp.position.clone());
                                }
                                ProfessionalData::Skills(skills) => {
                                    summary.skills_count = skills.skills.len();
                                }
                                _ => {}
                            }
                        }
                        ComponentData::Location(loc) => {
                            if let Some(addr) = &loc.address {
                                summary.location = Some(format!("{}, {}", addr.city, addr.country));
                            }
                        }
                        _ => {}
                    }
                    summary.last_updated = e.updated_at;
                }
            }
            
            PersonEvent::PersonDeactivated(e) => {
                let mut summaries = self.summaries.write().await;
                summaries.remove(&e.person_id);
            }
            
            PersonEvent::PersonMergedInto(e) => {
                let mut summaries = self.summaries.write().await;
                summaries.remove(&e.source_person_id);
            }
            
            _ => {} // Other events don't affect summaries
        }
        
        Ok(())
    }
    
    fn projection_name(&self) -> &str {
        "PersonSummaryProjection"
    }
    
    async fn clear(&self) -> DomainResult<()> {
        let mut summaries = self.summaries.write().await;
        summaries.clear();
        Ok(())
    }
} 