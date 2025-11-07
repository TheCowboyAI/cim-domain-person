//! Services for working with person entities
//!
//! This module provides services for composing person entities and
//! creating read-only views of person data.
//!
//! # CQRS Architecture
//!
//! The `PersonService` provides explicit separation between:
//! - **Commands** (write operations that change state)
//! - **Queries** (read operations against projections)
//!
//! This follows the CQRS (Command Query Responsibility Segregation) pattern
//! for FRP/CT compliance.

pub mod composition;
pub mod views;
pub mod network_analysis;
pub mod person_service;

pub use composition::PersonCompositionService;
pub use views::{PersonIdentityView, PersonViewService, LifecycleStatus};
pub use network_analysis::*;
pub use person_service::{PersonService, CommandOperation, QueryOperation}; 