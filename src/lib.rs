//! Person domain for the Composable Information Machine
//!
//! This crate provides the person domain implementation following ECS architecture:
//! - Person aggregate with minimal core identity
//! - Components for composable capabilities
//! - Cross-domain relationships
//!
//! ## ECS Architecture
//!
//! In ECS (Entity Component System), a Person is just an ID with components:
//! - Entity: PersonId (unique identifier)
//! - Components: EmailComponent, SkillComponent, etc. (data)
//! - Systems: Handle commands and process components (behavior)

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;
pub mod cross_domain;
pub mod components;
pub mod infrastructure;

// Re-export main types
pub use aggregate::{Person, PersonId, PersonMarker};
pub use commands::PersonCommand;
pub use events::PersonEvent;

// Re-export core value objects (minimal set)
pub use value_objects::PersonName;

// Re-export component types
pub use components::{
    contact::{EmailComponent, PhoneComponent, ContactContext},
    skills::{SkillComponent, CertificationComponent, EducationComponent},
    preferences::{CommunicationPreferencesComponent, PrivacyPreferencesComponent},
};

// Re-export cross-domain types
pub use cross_domain::person_location::{PersonAddress, PersonAddressType};
pub use cross_domain::person_organization::{EmploymentRelationship, EmploymentRole};

// Re-export infrastructure types
pub use infrastructure::{
    EventStore, InMemoryEventStore, EventEnvelope,
    PersonRepository, InMemorySnapshotStore,
    NatsEventStore, PersonCommandHandler,
    ComponentStore, InMemoryComponentStore,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
