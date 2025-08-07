//! Read model projections for Person domain
//! 
//! This module contains various projections that provide optimized
//! read models for different query patterns.

use cim_domain::DomainResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};

pub mod person_summary_projection;
pub mod person_search_projection;
pub mod person_skills_projection;
pub mod person_network_projection;
pub mod person_timeline_projection;

pub use person_summary_projection::*;
pub use person_search_projection::*;
pub use person_skills_projection::*;
pub use person_network_projection::*;
pub use person_timeline_projection::*;

mod async_handlers;
pub use async_handlers::{
    AsyncProjectionHandler, SummaryProjectionHandler, SkillsProjectionHandler,
    ProjectionStorage, register_projection_handlers
};

use crate::aggregate::PersonId;
use crate::events::PersonEvent;

/// Trait for projections that process person events
#[async_trait::async_trait]
pub trait PersonProjection: Send + Sync {
    /// Process a person event to update the projection
    async fn handle_event(&self, event: &PersonEvent) -> DomainResult<()>;
    
    /// Get the name of this projection
    fn projection_name(&self) -> &str;
    
    /// Clear all data in the projection
    async fn clear(&self) -> DomainResult<()>;
}

/// Manager for coordinating multiple projections
pub struct ProjectionManager {
    projections: Vec<Arc<dyn PersonProjection>>,
}

impl Default for ProjectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectionManager {
    pub fn new() -> Self {
        Self {
            projections: Vec::new(),
        }
    }
    
    /// Register a projection with the manager
    pub fn register_projection(&mut self, projection: Arc<dyn PersonProjection>) {
        self.projections.push(projection);
    }
    
    /// Process an event through all registered projections
    pub async fn handle_event(&self, event: &PersonEvent) -> DomainResult<()> {
        for projection in &self.projections {
            if let Err(e) = projection.handle_event(event).await {
                tracing::error!(
                    "Error in projection {}: {}",
                    projection.projection_name(),
                    e
                );
                // Continue processing other projections even if one fails
            }
        }
        Ok(())
    }
    
    /// Clear all projections
    pub async fn clear_all(&self) -> DomainResult<()> {
        for projection in &self.projections {
            projection.clear().await?;
        }
        Ok(())
    }
}

/// Common data structures used across projections

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSummary {
    pub person_id: PersonId,
    pub name: String,
    pub primary_email: Option<String>,
    pub primary_phone: Option<String>,
    pub current_employer: Option<String>,
    pub current_role: Option<String>,
    pub location: Option<String>,
    pub skills_count: usize,
    pub component_count: usize,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSearchResult {
    pub person_id: PersonId,
    pub name: String,
    pub email: Option<String>,
    pub employer: Option<String>,
    pub role: Option<String>,
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub skill_name: String,
    pub category: String,
    pub proficiency: String,
    pub years_experience: Option<f32>,
    pub last_used: Option<DateTime<Utc>>,
    pub endorsement_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub title: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}
