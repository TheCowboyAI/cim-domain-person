//! Component data structures for Person domain
//! 
//! This module contains the actual data storage for various person components.
//! Components are stored separately from the Person aggregate and referenced by ID.

use cim_domain::DomainResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub mod contact;
pub mod skills;
pub mod preferences;
pub mod social;
pub mod professional;
pub mod location;

pub use contact::*;
pub use skills::*;
pub use preferences::*;
pub use social::*;
pub use professional::*;
pub use location::*;

/// Unique identifier for a component instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentInstanceId(Uuid);

impl ComponentInstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ComponentInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ComponentInstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Base trait for all component data
pub trait ComponentDataTrait: Send + Sync + Clone {
    /// Get the component type
    fn component_type(&self) -> crate::aggregate::ComponentType;
    
    /// Validate the component data
    fn validate(&self) -> DomainResult<()>;
    
    /// Get a summary for display
    fn summary(&self) -> String;
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub updated_by: String,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

impl Default for ComponentMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            created_by: "system".to_string(),
            updated_by: "system".to_string(),
            tags: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
}

/// Container for a component with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInstance<T: ComponentDataTrait> {
    pub id: ComponentInstanceId,
    pub person_id: crate::aggregate::PersonId,
    pub data: T,
    pub metadata: ComponentMetadata,
    pub is_primary: bool,
    pub is_verified: bool,
}

impl<T: ComponentDataTrait> ComponentInstance<T> {
    pub fn new(person_id: crate::aggregate::PersonId, data: T) -> DomainResult<Self> {
        data.validate()?;
        
        Ok(Self {
            id: ComponentInstanceId::new(),
            person_id,
            data,
            metadata: ComponentMetadata::default(),
            is_primary: false,
            is_verified: false,
        })
    }
    
    pub fn with_metadata(mut self, metadata: ComponentMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn as_primary(mut self) -> Self {
        self.is_primary = true;
        self
    }
    
    pub fn as_verified(mut self) -> Self {
        self.is_verified = true;
        self
    }
}

/// Enum wrapper for all component data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentData {
    Contact(ContactData),
    Professional(ProfessionalData),
    Location(LocationData),
    Social(SocialData),
    Preferences(PreferencesData),
}

impl ComponentData {
    pub fn component_type(&self) -> crate::aggregate::ComponentType {
        match self {
            ComponentData::Contact(data) => data.component_type(),
            ComponentData::Professional(data) => data.component_type(),
            ComponentData::Location(data) => data.component_type(),
            ComponentData::Social(data) => data.component_type(),
            ComponentData::Preferences(data) => data.component_type(),
        }
    }
    
    pub fn validate(&self) -> DomainResult<()> {
        match self {
            ComponentData::Contact(data) => data.validate(),
            ComponentData::Professional(data) => data.validate(),
            ComponentData::Location(data) => data.validate(),
            ComponentData::Social(data) => data.validate(),
            ComponentData::Preferences(data) => data.validate(),
        }
    }
    
    pub fn summary(&self) -> String {
        match self {
            ComponentData::Contact(data) => data.summary(),
            ComponentData::Professional(data) => data.summary(),
            ComponentData::Location(data) => data.summary(),
            ComponentData::Social(data) => data.summary(),
            ComponentData::Preferences(data) => data.summary(),
        }
    }
} 