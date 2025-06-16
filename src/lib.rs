//! Person/People domain for the Composable Information Machine
//!
//! This crate provides the person domain implementation, including:
//! - Person aggregate with business logic
//! - Commands for person operations
//! - Events representing person state changes
//! - Command and query handlers
//! - Projections for read models
//! - Value objects specific to people

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;

// Re-export main types
pub use aggregate::{Person, PersonId, PersonMarker};
pub use commands::PersonCommand;
pub use events::PersonEvent;
pub use projections::{PersonProjection, EmployeeView, LdapProjection};
pub use queries::PersonQuery;
pub use value_objects::{
    IdentityComponent, ContactComponent, EmailAddress, PhoneNumber,
    EmploymentComponent, PositionComponent, SkillsComponent,
    SkillProficiency, Certification, Education, AccessComponent,
    ExternalIdentifiersComponent
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
