//! Policy for auto-archiving inactive persons

use async_trait::async_trait;
use chrono::{Duration, Utc};
use cim_domain::DomainResult;

use crate::commands::{PersonCommand, ArchivePerson};
use crate::events::PersonEventV2;
use super::Policy;

/// Policy that archives persons after a period of inactivity
pub struct AutoArchiveInactivePersonsPolicy {
    inactivity_threshold: Duration,
}

impl AutoArchiveInactivePersonsPolicy {
    pub fn new(inactivity_threshold: Duration) -> Self {
        Self {
            inactivity_threshold,
        }
    }
}

#[async_trait]
impl Policy for AutoArchiveInactivePersonsPolicy {
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>> {
        // This is a simplified example - in reality, you'd check activity from a projection
        if let PersonEventV2::Created { person_id, metadata, .. } = event {
            // Check if this is a reactivation after long inactivity
            let age = Utc::now().signed_duration_since(metadata.timestamp);
            
            if age > self.inactivity_threshold {
                // Generate archive command
                return Ok(vec![
                    PersonCommand::ArchivePerson(ArchivePerson {
                        person_id: *person_id,
                        reason: format!(
                            "Auto-archived due to {} days of inactivity",
                            self.inactivity_threshold.num_days()
                        ),
                    })
                ]);
            }
        }
        
        Ok(vec![])
    }
    
    fn name(&self) -> &str {
        "AutoArchiveInactivePersons"
    }
    
    fn applies_to(&self, event: &PersonEventV2) -> bool {
        // Only check on certain events
        matches!(
            event,
            PersonEventV2::Created { .. } | 
            PersonEventV2::Activated { .. }
        )
    }
}