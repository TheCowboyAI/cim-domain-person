//! Contact-related components for Person entities

use super::{ComponentMetadata, PersonComponent};
use crate::aggregate::person_ecs::ComponentType;
use crate::value_objects::{EmailAddress, PhoneNumber};
use serde::{Deserialize, Serialize};

/// Email component - can have multiple per person
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailComponent {
    /// The email address
    pub email: EmailAddress,
    
    /// Whether this is the primary email
    pub is_primary: bool,
    
    /// Usage context (personal, work, etc.)
    pub context: ContactContext,
    
    /// Component metadata
    pub metadata: ComponentMetadata,
}

impl PersonComponent for EmailComponent {
    fn component_type() -> ComponentType {
        ComponentType::EmailAddress
    }
}

/// Phone component - can have multiple per person
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneComponent {
    /// The phone number
    pub phone: PhoneNumber,
    
    /// Whether this is the primary phone
    pub is_primary: bool,
    
    /// Usage context
    pub context: ContactContext,
    
    /// Component metadata
    pub metadata: ComponentMetadata,
}

impl PersonComponent for PhoneComponent {
    fn component_type() -> ComponentType {
        ComponentType::PhoneNumber
    }
}

/// Context for contact information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactContext {
    Personal,
    Work,
    Emergency,
    Other(String),
} 