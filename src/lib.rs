//! Person domain for the Composable Information Machine
//!
//! This crate provides a pure functional person domain implementation:
//! - Person aggregate with core identity (name, birth/death dates)
//! - Pure functional event sourcing
//! - Category Theory compliance (Functors, Monads, Mealy State Machines)
//! - NATS-first messaging with comprehensive subject algebra
//!
//! ## Architecture
//!
//! Pure functional DDD with FRP principles:
//! - Aggregate: Person (immutable, pure functions only)
//! - Commands: Express intent (validated before execution)
//! - Events: Immutable facts (what happened)
//! - State Machines: Mealy machines for state transitions

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;
pub mod cross_domain;
pub mod infrastructure;
pub mod services;
pub mod policies;
pub mod nats;
pub mod workflow;

// Re-export main types
pub use aggregate::{Person, PersonId, PersonMarker};
pub use commands::PersonCommand;
pub use events::PersonEvent;

// Re-export core value objects (minimal set)
pub use value_objects::PersonName;

// Re-export cross-domain types
pub use cross_domain::person_organization::{EmploymentRelationship, EmploymentRole};

// Re-export services
pub use services::{NetworkAnalysisService, NetworkMetrics, NetworkPath, NetworkCommunity};

// Re-export infrastructure types
pub use infrastructure::{
    EventStore, InMemoryEventStore, EventEnvelope,
    PersonRepository, InMemorySnapshotStore,
    NatsEventStore, PersonCommandHandler,
};

// Re-export NATS types
pub use nats::{
    PersonSubject, PersonSubjectRoot, PersonAggregate, PersonEventType, PersonCommandType, 
    PersonQueryType, PersonScope, PersonSubjectBuilder,
    MessageIdentity, MessageId, CorrelationId, CausationId, PersonActor,
    PersonMessageEnvelope, PersonTracingContext,
};

// Re-export workflow types
pub use workflow::{
    WorkflowId, WorkflowState, PersonWorkflowType, WorkflowDefinition, WorkflowInstance,
    WorkflowManager, WorkflowEngine, DefaultWorkflowEngine, WorkflowEvent, WorkflowError,
    create_person_onboarding_workflow, create_employment_lifecycle_workflow,
    create_skills_certification_workflow, create_privacy_compliance_workflow,
    get_predefined_workflows,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
