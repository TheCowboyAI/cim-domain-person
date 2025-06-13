//! Person aggregate and related components
//!
//! A Person is an aggregate with an ID and various components that can be
//! composed to create different views (Employee, Customer, etc.)

use cim_domain::{AggregateRoot, Entity, EntityId, DomainError, DomainResult};
use cim_domain::{Component, ComponentStorage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::any::Any;

/// Person aggregate - represents an individual with composable components
#[derive(Debug, Clone)]
pub struct Person {
    /// Core entity data
    entity: Entity<PersonMarker>,

    /// Version for optimistic concurrency control
    version: u64,

    /// Components attached to this person
    components: ComponentStorage,

    /// Component metadata (when added, by whom, etc.)
    component_metadata: HashMap<String, ComponentMetadata>,
}

/// Marker type for Person entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PersonMarker;

/// Metadata about when and why a component was added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// When this component was added
    pub added_at: std::time::SystemTime,

    /// Who added this component
    pub added_by: String,

    /// Reason or context for adding
    pub reason: Option<String>,
}

impl Person {
    /// Create a new person with basic identity
    pub fn new(id: EntityId<PersonMarker>, identity: IdentityComponent) -> Self {
        let mut components = ComponentStorage::new();
        components.add(identity).unwrap();

        let mut component_metadata = HashMap::new();
        component_metadata.insert(
            "Identity".to_string(),
            ComponentMetadata {
                added_at: std::time::SystemTime::now(),
                added_by: "system".to_string(),
                reason: Some("Initial identity".to_string()),
            },
        );

        Self {
            entity: Entity::with_id(id),
            version: 0,
            components,
            component_metadata,
        }
    }

    /// Add a component to this person
    pub fn add_component<C: Component + 'static>(
        &mut self,
        component: C,
        added_by: &str,
        reason: Option<String>,
    ) -> DomainResult<()> {
        let component_type = component.type_name().to_string();

        // Add the component
        self.components.add(component)?;

        // Add metadata
        self.component_metadata.insert(
            component_type,
            ComponentMetadata {
                added_at: std::time::SystemTime::now(),
                added_by: added_by.to_string(),
                reason,
            },
        );

        self.entity.touch();
        self.version += 1;

        Ok(())
    }

    /// Remove a component
    pub fn remove_component<C: Component + 'static>(&mut self) -> DomainResult<()> {
        let component_type = std::any::type_name::<C>();

        if self.components.remove::<C>().is_some() {
            self.component_metadata.remove(component_type);
            self.entity.touch();
            self.version += 1;
            Ok(())
        } else {
            Err(DomainError::ComponentNotFound(format!(
                "Component {} not found",
                component_type
            )))
        }
    }

    /// Get a component
    pub fn get_component<C: Component + 'static>(&self) -> Option<&C> {
        self.components.get::<C>()
    }

    /// Check if person has a component
    pub fn has_component<C: Component + 'static>(&self) -> bool {
        self.components.has::<C>()
    }

    /// Get all component types
    pub fn component_types(&self) -> Vec<String> {
        self.component_metadata.keys().cloned().collect()
    }
}

impl AggregateRoot for Person {
    type Id = EntityId<PersonMarker>;

    fn id(&self) -> Self::Id {
        self.entity.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
        self.entity.touch();
    }
}

// Re-export person-specific components
pub use crate::value_objects::{
    IdentityComponent, ContactComponent, EmailAddress, PhoneNumber,
    EmploymentComponent, PositionComponent, SkillsComponent,
    SkillProficiency, Certification, Education, AccessComponent,
    ExternalIdentifiersComponent
};

// Re-export person ID type
pub type PersonId = EntityId<PersonMarker>;
