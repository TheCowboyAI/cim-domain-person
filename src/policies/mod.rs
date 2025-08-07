//! Policy engine for event-driven business rules

use async_trait::async_trait;
use cim_domain::DomainResult;
use std::sync::Arc;
use tracing::{info, debug};

use crate::commands::PersonCommand;
use crate::events::PersonEventV2;

/// Policy trait for event-driven rules
#[async_trait]
pub trait Policy: Send + Sync {
    /// Evaluate an event and potentially generate commands
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>>;
    
    /// Get the policy name
    fn name(&self) -> &str;
    
    /// Check if this policy applies to the given event
    fn applies_to(&self, event: &PersonEventV2) -> bool {
        // By default, filter policies based on event type
        match event {
            // Most policies care about these core events
            PersonEventV2::Created { .. } |
            PersonEventV2::Updated { .. } |
            PersonEventV2::NameUpdated { .. } |
            PersonEventV2::ComponentAdded { .. } |
            PersonEventV2::ComponentUpdated { .. } => true,
            
            // Skip archived/merged persons for most policies
            PersonEventV2::Archived { .. } |
            PersonEventV2::PersonMerged { .. } => false,
            
            // Other events depend on specific policy logic
            _ => true,
        }
    }
}

/// Policy engine that evaluates events against registered policies
pub struct PolicyEngine {
    policies: Vec<Arc<dyn Policy>>,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }
    
    /// Register a policy
    pub fn register(&mut self, policy: Arc<dyn Policy>) {
        info!("Registered policy: {}", policy.name());
        self.policies.push(policy);
    }
    
    /// Evaluate an event against all policies
    pub async fn evaluate(&self, event: &PersonEventV2) -> Vec<PersonCommand> {
        let mut commands = Vec::new();
        
        for policy in &self.policies {
            if !policy.applies_to(event) {
                continue;
            }
            
            debug!("Evaluating policy {} for event {}", policy.name(), event.event_type());
            
            match policy.evaluate(event).await {
                Ok(policy_commands) => {
                    if !policy_commands.is_empty() {
                        info!(
                            "Policy {} generated {} commands",
                            policy.name(),
                            policy_commands.len()
                        );
                        commands.extend(policy_commands);
                    }
                }
                Err(e) => {
                    tracing::error!("Policy {} failed: {}", policy.name(), e);
                    // Continue with other policies even if one fails
                }
            }
        }
        
        commands
    }
}

// Example policies

mod auto_archive_policy;
mod skill_recommendation_policy;
mod welcome_email_policy;
mod data_quality_policy;

pub use auto_archive_policy::AutoArchiveInactivePersonsPolicy;
pub use skill_recommendation_policy::SkillRecommendationPolicy;
pub use welcome_email_policy::WelcomeEmailPolicy;
pub use data_quality_policy::DataQualityPolicy;

/// Create a default policy engine with standard policies
pub fn create_default_policy_engine() -> PolicyEngine {
    let mut engine = PolicyEngine::new();
    
    // Register standard policies
    engine.register(Arc::new(AutoArchiveInactivePersonsPolicy::new(
        chrono::Duration::days(365)
    )));
    
    engine.register(Arc::new(WelcomeEmailPolicy::new()));
    
    engine.register(Arc::new(DataQualityPolicy::new()));
    
    engine
}