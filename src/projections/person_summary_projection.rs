//! Person summary projection providing a quick overview of person data

use super::{PersonProjection, PersonSummary};
use crate::aggregate::PersonId;
use crate::events::*;
use cim_domain::DomainResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Extract person ID from any PersonEvent variant
fn extract_person_id(event: &PersonEvent) -> PersonId {
    match event {
        PersonEvent::PersonCreated(e) => e.person_id,
        PersonEvent::PersonUpdated(e) => e.person_id,
        PersonEvent::NameUpdated(e) => e.person_id,
        PersonEvent::BirthDateSet(e) => e.person_id,
        PersonEvent::DeathRecorded(e) => e.person_id,
        PersonEvent::PersonDeactivated(e) => e.person_id,
        PersonEvent::PersonReactivated(e) => e.person_id,
        PersonEvent::PersonMergedInto(e) => e.source_person_id,
        PersonEvent::AttributeRecorded(e) => e.person_id,
        PersonEvent::AttributeUpdated(e) => e.person_id,
        PersonEvent::AttributeInvalidated(e) => e.person_id,
    }
}

/// Projection that maintains person summaries for quick access
pub struct PersonSummaryProjection {
    summaries: Arc<RwLock<HashMap<PersonId, PersonSummary>>>,
}

impl Default for PersonSummaryProjection {
    fn default() -> Self {
        Self::new()
    }
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
        // Infrastructure adapter: Load → Apply Pure Function → Save

        // Get person ID from event (helper function extracts from any variant)
        let person_id = extract_person_id(event);

        // Load current state
        let current = {
            let summaries = self.summaries.read().await;
            summaries.get(&person_id).cloned()
        };

        // Apply pure projection function (no side effects!)
        let new_state = super::pure_projections::project_person_summary(current, event);

        // Save new state (side effect isolated to infrastructure)
        let mut summaries = self.summaries.write().await;
        match new_state {
            Some(summary) => {
                summaries.insert(person_id, summary);
            }
            None => {
                summaries.remove(&person_id);
            }
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