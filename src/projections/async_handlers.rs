//! Async projection update handlers

use async_trait::async_trait;
use cim_domain::DomainResult;
use std::sync::Arc;
use tracing::{info, debug};

use crate::aggregate::PersonId;
use crate::events::{PersonEventV2, StreamingEventEnvelope};
use crate::infrastructure::{StreamingEventHandler, SubscriptionManager};
use crate::projections::PersonSummary;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Base trait for async projection handlers
#[async_trait]
pub trait AsyncProjectionHandler: StreamingEventHandler {
    /// Get the projection name
    fn projection_name(&self) -> &str;
    
    /// Handle a specific event type
    async fn handle_specific_event(&self, event: &PersonEventV2) -> DomainResult<()>;
}

/// Summary projection handler
pub struct SummaryProjectionHandler {
    storage: Arc<dyn ProjectionStorage<PersonSummary>>,
}

impl SummaryProjectionHandler {
    pub fn new(storage: Arc<dyn ProjectionStorage<PersonSummary>>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl StreamingEventHandler for SummaryProjectionHandler {
    async fn handle_event(&self, envelope: StreamingEventEnvelope) -> DomainResult<()> {
        self.handle_specific_event(&envelope.event).await
    }
    
    fn name(&self) -> &str {
        "summary-projection"
    }
}

#[async_trait]
impl AsyncProjectionHandler for SummaryProjectionHandler {
    fn projection_name(&self) -> &str {
        "PersonSummary"
    }
    
    async fn handle_specific_event(&self, event: &PersonEventV2) -> DomainResult<()> {
        match event {
            PersonEventV2::Created { person_id, name, metadata, .. } => {
                debug!("Creating summary for person {}", person_id);
                
                let summary = PersonSummary {
                    person_id: *person_id,
                    name: name.full_name(),
                    primary_email: None,
                    primary_phone: None,
                    location: None,
                    current_employer: None,
                    current_role: None,
                    skills_count: 0,
                    component_count: 0,
                    last_updated: metadata.timestamp,
                };
                
                self.storage.save(person_id, &summary).await?;
                info!("Created summary projection for person {}", person_id);
            }
            
            PersonEventV2::NameUpdated { person_id, new_name, metadata, .. } => {
                if let Some(mut summary) = self.storage.get(person_id).await? {
                    summary.name = new_name.full_name();
                    summary.last_updated = metadata.timestamp;
                    self.storage.save(person_id, &summary).await?;
                    debug!("Updated name in summary for person {}", person_id);
                }
            }
            
            PersonEventV2::Suspended { person_id, metadata, .. } => {
                if let Some(mut summary) = self.storage.get(person_id).await? {
                    // Mark as suspended in some way, perhaps via component_count or a custom field
                    summary.last_updated = metadata.timestamp;
                    self.storage.save(person_id, &summary).await?;
                    debug!("Suspended person {} in summary", person_id);
                }
            }
            
            PersonEventV2::Activated { person_id, metadata, .. } => {
                if let Some(mut summary) = self.storage.get(person_id).await? {
                    // Mark as active again
                    summary.last_updated = metadata.timestamp;
                    self.storage.save(person_id, &summary).await?;
                    debug!("Activated person {} in summary", person_id);
                }
            }
            
            PersonEventV2::ComponentAdded { person_id, component_type, component_data, metadata } => {
                if let Some(mut summary) = self.storage.get(person_id).await? {
                    // Update based on component type
                    match component_type.to_string().as_str() {
                        "Email" => {
                            if let Some(email) = component_data.get("email").and_then(|v| v.as_str()) {
                                summary.primary_email = Some(email.to_string());
                            }
                        }
                        "Phone" => {
                            if let Some(phone) = component_data.get("phone_number").and_then(|v| v.as_str()) {
                                summary.primary_phone = Some(phone.to_string());
                            }
                        }
                        "Skill" => {
                            summary.skills_count += 1;
                        }
                        _ => {}
                    }
                    
                    summary.last_updated = metadata.timestamp;
                    summary.component_count += 1;
                    self.storage.save(person_id, &summary).await?;
                    debug!("Added component {} to summary for person {}", component_type, person_id);
                }
            }
            
            _ => {
                // Other events don't affect summary
            }
        }
        
        Ok(())
    }
}

/// Person skills view projection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonSkillsView {
    pub person_id: PersonId,
    pub person_name: String,
    pub skills: Vec<SkillData>,
    pub skill_categories: HashMap<String, usize>,
    pub total_endorsements: u32,
    pub average_proficiency: f32,
    pub last_skill_update: Option<DateTime<Utc>>,
}

/// Skills projection handler
pub struct SkillsProjectionHandler {
    storage: Arc<dyn ProjectionStorage<PersonSkillsView>>,
}

impl SkillsProjectionHandler {
    pub fn new(storage: Arc<dyn ProjectionStorage<PersonSkillsView>>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl StreamingEventHandler for SkillsProjectionHandler {
    async fn handle_event(&self, envelope: StreamingEventEnvelope) -> DomainResult<()> {
        self.handle_specific_event(&envelope.event).await
    }
    
    fn name(&self) -> &str {
        "skills-projection"
    }
}

#[async_trait]
impl AsyncProjectionHandler for SkillsProjectionHandler {
    fn projection_name(&self) -> &str {
        "PersonSkillsView"
    }
    
    async fn handle_specific_event(&self, event: &PersonEventV2) -> DomainResult<()> {
        match event {
            PersonEventV2::Created { person_id, name, .. } => {
                let skills_view = PersonSkillsView {
                    person_id: *person_id,
                    person_name: name.full_name(),
                    skills: Vec::new(),
                    skill_categories: std::collections::HashMap::new(),
                    total_endorsements: 0,
                    average_proficiency: 0.0,
                    last_skill_update: None,
                };
                
                self.storage.save(person_id, &skills_view).await?;
                info!("Created skills projection for person {}", person_id);
            }
            
            PersonEventV2::ComponentAdded { person_id, component_type, component_data, metadata } => {
                if component_type.to_string() == "Skill" {
                    if let Some(mut skills_view) = self.storage.get(person_id).await? {
                        // Parse skill data and add to view
                        if let Ok(skill) = serde_json::from_value::<SkillData>(component_data.clone()) {
                            skills_view.skills.push(skill);
                            skills_view.last_skill_update = Some(metadata.timestamp);
                            
                            // Recalculate aggregates
                            skills_view.recalculate_aggregates();
                            
                            self.storage.save(person_id, &skills_view).await?;
                            debug!("Added skill to projection for person {}", person_id);
                        }
                    }
                }
            }
            
            PersonEventV2::ComponentUpdated { person_id, component_type,  metadata, .. } => {
                if component_type.to_string() == "Skill" {
                    if let Some(mut skills_view) = self.storage.get(person_id).await? {
                        // Update skill in view
                        skills_view.last_skill_update = Some(metadata.timestamp);
                        skills_view.recalculate_aggregates();
                        
                        self.storage.save(person_id, &skills_view).await?;
                        debug!("Updated skill in projection for person {}", person_id);
                    }
                }
            }
            
            _ => {}
        }
        
        Ok(())
    }
}

/// Trait for projection storage
#[async_trait]
pub trait ProjectionStorage<T>: Send + Sync {
    /// Save a projection
    async fn save(&self, id: &PersonId, projection: &T) -> DomainResult<()>;
    
    /// Get a projection
    async fn get(&self, id: &PersonId) -> DomainResult<Option<T>>;
    
    /// Delete a projection
    async fn delete(&self, id: &PersonId) -> DomainResult<()>;
}

/// Skill data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillData {
    pub name: String,
    pub category: String,
    pub proficiency: f32,
    pub endorsements: u32,
}

impl PersonSkillsView {
    fn recalculate_aggregates(&mut self) {
        // Update skill categories
        self.skill_categories.clear();
        for skill in &self.skills {
            *self.skill_categories.entry(skill.category.clone()).or_insert(0) += 1;
        }
        
        // Calculate total endorsements
        self.total_endorsements = self.skills.iter().map(|s| s.endorsements).sum();
        
        // Calculate average proficiency
        if !self.skills.is_empty() {
            let total_proficiency: f32 = self.skills.iter().map(|s| s.proficiency).sum();
            self.average_proficiency = total_proficiency / self.skills.len() as f32;
        }
    }
}

/// Register all projection handlers
pub fn register_projection_handlers(
    subscription_manager: &mut SubscriptionManager,
    summary_storage: Arc<dyn ProjectionStorage<PersonSummary>>,
    skills_storage: Arc<dyn ProjectionStorage<PersonSkillsView>>,
) {
    // Register summary projection handler
    subscription_manager.register_handler(
        Box::new(SummaryProjectionHandler::new(summary_storage))
    );
    
    // Register skills projection handler
    subscription_manager.register_handler(
        Box::new(SkillsProjectionHandler::new(skills_storage))
    );
    
    info!("Registered async projection handlers");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::PersonName;
    use crate::infrastructure::EventMetadata;
    
    #[tokio::test]
    async fn test_summary_projection_handler() {
        // Mock storage implementation would go here
    }
}