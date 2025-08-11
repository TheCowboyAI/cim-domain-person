//! Workflow management for Person domain
//!
//! This module provides workflow orchestration for person-related processes
//! including onboarding, verification, employment transitions, and privacy operations.

pub mod definitions;
pub mod manager;
pub mod person_workflows;

pub use definitions::*;
pub use manager::*;
pub use person_workflows::*;