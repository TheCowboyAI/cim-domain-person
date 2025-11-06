//! Services for working with person entities
//!
//! This module provides services for composing person entities and
//! creating read-only views of person data.

pub mod composition;
pub mod views;
pub mod network_analysis;

pub use composition::PersonCompositionService;
pub use views::{PersonIdentityView, PersonViewService, LifecycleStatus};

pub use network_analysis::*; 