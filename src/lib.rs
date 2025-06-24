//! Person domain for the Composable Information Machine
//!
//! This crate provides the person domain implementation, including:
//! - Person aggregate with business logic
//! - Commands for person operations
//! - Events representing person state changes
//! - Value objects for person data

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;
pub mod event_store;

// Re-export main types
pub use aggregate::{Person, PersonId, PersonMarker};
pub use commands::PersonCommand;
pub use events::PersonEvent;
pub use value_objects::{PersonName, EmailAddress, PhoneNumber, PhysicalAddress};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
