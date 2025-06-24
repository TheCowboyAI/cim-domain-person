//! Person aggregate module - ECS architecture

// Export the ECS-oriented person aggregate
pub mod person_ecs;
pub use person_ecs::{Person, PersonId, PersonMarker, CoreIdentity, PersonLifecycle, ComponentType};
