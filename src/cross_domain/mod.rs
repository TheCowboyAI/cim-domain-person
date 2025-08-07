//! Cross-domain integration for Person domain
//!
//! This module handles integration with other domains in the CIM ecosystem:
//! - Identity domain: For organization relationships
//! - Location domain: For address components
//! - Git domain: For contribution tracking
//! - Agent domain: For AI agent associations
//!
//! It also defines how Person domain interacts with other domains
//! without violating domain boundaries.

pub mod person_organization;
pub mod identity_integration;
pub mod location_integration;
pub mod git_integration;
pub mod agent_integration;

// Re-export commonly used types
pub use identity_integration::{IdentityDomainEvent, IdentityEventHandler};
pub use location_integration::{LocationDomainEvent, LocationEventHandler, AddressUsageType};
pub use git_integration::{GitDomainEvent, GitEventHandler, LanguageStats};
pub use agent_integration::{AgentDomainEvent, AgentEventHandler, AgentType, AssignmentType, AgentPermission};

use cim_domain::DomainResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use async_trait::async_trait;
use crate::events::PersonEvent;


/// Cross-domain event that Person domain listens to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossDomainEvent {
    // From Identity domain
    OrganizationCreated {
        org_id: String,
        name: String,
        created_at: chrono::DateTime<chrono::Utc>,
    },
    PersonAddedToOrganization {
        person_id: crate::aggregate::PersonId,
        org_id: String,
        role: String,
        added_at: chrono::DateTime<chrono::Utc>,
    },
    PersonRemovedFromOrganization {
        person_id: crate::aggregate::PersonId,
        org_id: String,
        removed_at: chrono::DateTime<chrono::Utc>,
    },
    
    // From Location domain
    AddressCreated {
        address_id: String,
        street: String,
        city: String,
        country: String,
        postal_code: String,
    },
    AddressAssignedToPerson {
        person_id: crate::aggregate::PersonId,
        address_id: String,
        address_type: AddressType,
    },
    
    // From Git domain
    CommitAuthorIdentified {
        person_id: crate::aggregate::PersonId,
        commit_hash: String,
        repository: String,
        author_email: String,
        committed_at: chrono::DateTime<chrono::Utc>,
    },
    ContributionMetricsUpdated {
        person_id: crate::aggregate::PersonId,
        total_commits: u64,
        repositories: Vec<String>,
        languages: Vec<String>,
    },
    
    // From Agent domain
    AgentAssignedToPerson {
        person_id: crate::aggregate::PersonId,
        agent_id: String,
        agent_type: String,
        capabilities: Vec<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    Home,
    Work,
    Billing,
    Shipping,
    Other,
}

/// Cross-domain command that Person domain can send
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossDomainCommand {
    // To Identity domain
    RequestOrganizationDetails {
        org_id: String,
        requester_id: crate::aggregate::PersonId,
    },
    
    // To Location domain
    CreateAddressForPerson {
        person_id: crate::aggregate::PersonId,
        street: String,
        city: String,
        state: Option<String>,
        country: String,
        postal_code: String,
        address_type: AddressType,
    },
    
    // To Git domain
    LinkGitIdentity {
        person_id: crate::aggregate::PersonId,
        git_email: String,
        git_username: Option<String>,
    },
    
    // To Agent domain
    RequestAgentAssignment {
        person_id: crate::aggregate::PersonId,
        agent_type: String,
        required_capabilities: Vec<String>,
    },
}

/// Service for handling cross-domain integration
pub struct CrossDomainIntegrationService {
    #[allow(dead_code)]
    event_publisher: Arc<dyn EventPublisher>,
    command_sender: Arc<dyn CommandSender>,
}

/// Trait for publishing events to other domains
#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, topic: &str, event: CrossDomainEvent) -> DomainResult<()>;
}

/// Trait for sending commands to other domains
#[async_trait::async_trait]
pub trait CommandSender: Send + Sync {
    async fn send(&self, target: &str, command: CrossDomainCommand) -> DomainResult<()>;
}

/// Trait for handling events from other domains
#[async_trait]
pub trait DomainEventHandler: Send + Sync {
    /// Handle an event from another domain
    async fn handle_event(&self, event: CrossDomainEvent) -> DomainResult<Vec<PersonEvent>>;
}

impl CrossDomainIntegrationService {
    pub fn new(
        event_publisher: Arc<dyn EventPublisher>,
        command_sender: Arc<dyn CommandSender>,
    ) -> Self {
        Self {
            event_publisher,
            command_sender,
        }
    }
    
    /// Handle incoming cross-domain event
    pub async fn handle_event(&self, event: CrossDomainEvent) -> DomainResult<()> {
        match event {
            CrossDomainEvent::PersonAddedToOrganization { person_id, org_id, role, .. } => {
                // Update person's employment component
                // This would trigger component updates in the Person domain
                tracing::info!(
                    "Person {} added to organization {} with role {}",
                    person_id, org_id, role
                );
                Ok(())
            }
            CrossDomainEvent::AddressAssignedToPerson { person_id, address_id, address_type } => {
                // Update person's address component
                tracing::info!(
                    "Address {} assigned to person {} as {:?}",
                    address_id, person_id, address_type
                );
                Ok(())
            }
            CrossDomainEvent::CommitAuthorIdentified { person_id, repository, .. } => {
                // Update person's contribution metrics
                tracing::info!(
                    "Commit identified for person {} in repository {}",
                    person_id, repository
                );
                Ok(())
            }
            _ => Ok(()),
        }
    }
    
    /// Send command to another domain
    pub async fn send_command(&self, command: CrossDomainCommand) -> DomainResult<()> {
        let target = match &command {
            CrossDomainCommand::RequestOrganizationDetails { .. } => "identity",
            CrossDomainCommand::CreateAddressForPerson { .. } => "location",
            CrossDomainCommand::LinkGitIdentity { .. } => "git",
            CrossDomainCommand::RequestAgentAssignment { .. } => "agent",
        };
        
        self.command_sender.send(target, command).await
    }
} 