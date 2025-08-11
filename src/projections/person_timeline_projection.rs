//! Person timeline projection for activity history

use super::{PersonProjection, TimelineEntry};
use crate::aggregate::PersonId;
use crate::events::*;
use crate::components::data::ComponentData;
use cim_domain::DomainResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Projection that maintains activity timelines for persons
pub struct PersonTimelineProjection {
    timelines: Arc<RwLock<HashMap<PersonId, Vec<TimelineEntry>>>>,
}

impl Default for PersonTimelineProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl PersonTimelineProjection {
    pub fn new() -> Self {
        Self {
            timelines: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get timeline for a person
    pub async fn get_timeline(
        &self,
        person_id: &PersonId,
        limit: Option<usize>
    ) -> Vec<TimelineEntry> {
        let timelines = self.timelines.read().await;
        
        if let Some(timeline) = timelines.get(person_id) {
            match limit {
                Some(n) => timeline.iter().rev().take(n).cloned().collect(),
                None => timeline.clone(),
            }
        } else {
            Vec::new()
        }
    }
    
    /// Get timeline entries within a date range
    pub async fn get_timeline_range(
        &self,
        person_id: &PersonId,
        start: DateTime<Utc>,
        end: DateTime<Utc>
    ) -> Vec<TimelineEntry> {
        let timelines = self.timelines.read().await;
        
        if let Some(timeline) = timelines.get(person_id) {
            timeline.iter()
                .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get timeline entries by event type
    pub async fn get_timeline_by_type(
        &self,
        person_id: &PersonId,
        event_type: &str
    ) -> Vec<TimelineEntry> {
        let timelines = self.timelines.read().await;
        
        if let Some(timeline) = timelines.get(person_id) {
            timeline.iter()
                .filter(|entry| entry.event_type == event_type)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Add an entry to the timeline
    async fn add_timeline_entry(&self, person_id: PersonId, entry: TimelineEntry) {
        let mut timelines = self.timelines.write().await;
        let timeline = timelines.entry(person_id).or_insert_with(Vec::new);
        
        // Insert in chronological order
        match timeline.binary_search_by_key(&entry.timestamp, |e| e.timestamp) {
            Ok(pos) | Err(pos) => timeline.insert(pos, entry),
        }
    }
}

#[async_trait::async_trait]
impl PersonProjection for PersonTimelineProjection {
    async fn handle_event(&self, event: &PersonEvent) -> DomainResult<()> {
        match event {
            PersonEvent::PersonCreated(e) => {
                let entry = TimelineEntry {
                    timestamp: e.created_at,
                    event_type: "person_created".to_string(),
                    title: "Person Record Created".to_string(),
                    description: format!("Created person record for {}", e.name.display_name()),
                    metadata: HashMap::new(),
                };
                
                self.add_timeline_entry(e.person_id, entry).await;
            }
            
            PersonEvent::NameUpdated(e) => {
                let mut metadata = HashMap::new();
                metadata.insert("old_name".to_string(), serde_json::to_value(&e.old_name).unwrap());
                metadata.insert("new_name".to_string(), serde_json::to_value(&e.new_name).unwrap());
                
                let entry = TimelineEntry {
                    timestamp: e.updated_at,
                    event_type: "name_updated".to_string(),
                    title: "Name Changed".to_string(),
                    description: format!("Name changed from {} to {}", 
                        e.old_name.display_name(),
                        e.new_name.display_name()
                    ),
                    metadata,
                };
                
                self.add_timeline_entry(e.person_id, entry).await;
            }
            
            PersonEvent::BirthDateSet(e) => {
                let mut metadata = HashMap::new();
                metadata.insert("birth_date".to_string(), serde_json::to_value(e.birth_date).unwrap());
                
                let entry = TimelineEntry {
                    timestamp: e.set_at,
                    event_type: "birth_date_set".to_string(),
                    title: "Birth Date Recorded".to_string(),
                    description: format!("Birth date set to {}", e.birth_date),
                    metadata,
                };
                
                self.add_timeline_entry(e.person_id, entry).await;
            }
            
            PersonEvent::DeathRecorded(e) => {
                let mut metadata = HashMap::new();
                metadata.insert("date_of_death".to_string(), serde_json::to_value(e.date_of_death).unwrap());
                
                let entry = TimelineEntry {
                    timestamp: e.recorded_at,
                    event_type: "death_recorded".to_string(),
                    title: "Death Recorded".to_string(),
                    description: format!("Death recorded on {}", e.date_of_death),
                    metadata,
                };
                
                self.add_timeline_entry(e.person_id, entry).await;
            }
            
            PersonEvent::ComponentRegistered(e) => {
                let mut metadata = HashMap::new();
                metadata.insert("component_type".to_string(), serde_json::to_value(&e.component_type).unwrap());
                
                let entry = TimelineEntry {
                    timestamp: e.registered_at,
                    event_type: "component_registered".to_string(),
                    title: format!("{} Component Added", e.component_type),
                    description: format!("Registered {} component", e.component_type),
                    metadata,
                };
                
                self.add_timeline_entry(e.person_id, entry).await;
            }
            
            PersonEvent::ComponentDataUpdated(e) => {
                let (event_type, title, description) = match &e.data {
                    ComponentData::Contact(contact) => {
                        let contact_type = contact.contact_type();
                        (
                            format!("{contact_type}_updated"),
                            format!("{contact_type} Updated"),
                            format!("{contact_type} information updated")
                        )
                    }
                    ComponentData::Professional(prof) => {
                        let prof_type = prof.professional_type();
                        (
                            format!("{prof_type}_updated"),
                            format!("{prof_type} Updated"),
                            format!("{prof_type} information updated")
                        )
                    }
                    ComponentData::Location(_) => (
                        "location_updated".to_string(),
                        "Location Updated".to_string(),
                        "Location information updated".to_string()
                    ),
                    ComponentData::Social(social) => {
                        let social_type = social.social_type();
                        (
                            format!("{social_type}_updated"),
                            format!("{social_type} Updated"),
                            format!("{social_type} information updated")
                        )
                    }
                    ComponentData::Preferences(_) => (
                        "preferences_updated".to_string(),
                        "Preferences Updated".to_string(),
                        "Preferences updated".to_string()
                    ),
                };
                
                let mut metadata = HashMap::new();
                metadata.insert("component_id".to_string(), serde_json::to_value(e.component_id).unwrap());
                
                let entry = TimelineEntry {
                    timestamp: e.updated_at,
                    event_type,
                    title,
                    description,
                    metadata,
                };
                
                self.add_timeline_entry(e.person_id, entry).await;
            }
            
            PersonEvent::PersonDeactivated(e) => {
                let mut metadata = HashMap::new();
                metadata.insert("reason".to_string(), serde_json::to_value(&e.reason).unwrap());
                
                let entry = TimelineEntry {
                    timestamp: e.deactivated_at,
                    event_type: "person_deactivated".to_string(),
                    title: "Person Deactivated".to_string(),
                    description: format!("Person deactivated: {}", e.reason),
                    metadata,
                };
                
                self.add_timeline_entry(e.person_id, entry).await;
            }
            
            PersonEvent::PersonReactivated(e) => {
                let entry = TimelineEntry {
                    timestamp: e.reactivated_at,
                    event_type: "person_reactivated".to_string(),
                    title: "Person Reactivated".to_string(),
                    description: "Person record reactivated".to_string(),
                    metadata: HashMap::new(),
                };
                
                self.add_timeline_entry(e.person_id, entry).await;
            }
            
            PersonEvent::PersonMergedInto(e) => {
                let mut metadata = HashMap::new();
                metadata.insert("merged_into_id".to_string(), serde_json::to_value(e.merged_into_id).unwrap());
                
                let entry = TimelineEntry {
                    timestamp: e.merged_at,
                    event_type: "person_merged".to_string(),
                    title: "Person Record Merged".to_string(),
                    description: format!("Merged into person {}", e.merged_into_id),
                    metadata,
                };
                
                self.add_timeline_entry(e.source_person_id, entry).await;
            }
            
            _ => {} // Other events handled above
        }
        
        Ok(())
    }
    
    fn projection_name(&self) -> &str {
        "PersonTimelineProjection"
    }
    
    async fn clear(&self) -> DomainResult<()> {
        let mut timelines = self.timelines.write().await;
        timelines.clear();
        Ok(())
    }
} 