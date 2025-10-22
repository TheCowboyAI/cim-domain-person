//! Workflow management for Person domain
//!
//! This module provides workflow orchestration for person-related processes
//! including onboarding, verification, employment transitions, and privacy operations.

pub mod definitions;
pub mod manager;
pub mod person_workflows;

// Re-export specific items to avoid conflicts
pub use definitions::{
    WorkflowId, WorkflowState, PersonWorkflowType, WorkflowDefinition, WorkflowInstance,
};
pub use manager::{
    WorkflowManager, WorkflowEngine, DefaultWorkflowEngine,
    WorkflowError, WorkflowEvent, // Both are in manager
};
pub use person_workflows::*;