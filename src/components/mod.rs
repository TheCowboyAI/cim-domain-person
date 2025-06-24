//! ECS Components for Person domain
//!
//! In ECS architecture, components are data containers that can be composed
//! onto entities. Each component represents a single aspect or capability.
//!
//! Components should:
//! - Be pure data (no behavior)
//! - Represent a single concern
//! - Be composable with other components
//! - Not reference other entities directly (use IDs)

pub mod contact;
pub mod skills;
pub mod preferences;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Base trait for all person components
pub trait PersonComponent: Send + Sync + 'static {
    /// Get the component type identifier
    fn component_type() -> super::aggregate::person_ecs::ComponentType;
}

/// Metadata common to all components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// When this component was attached
    pub attached_at: DateTime<Utc>,
    
    /// Last update time
    pub updated_at: DateTime<Utc>,
    
    /// Source system that created this component
    pub source: String,
    
    /// Version for optimistic concurrency
    pub version: u64,
} 