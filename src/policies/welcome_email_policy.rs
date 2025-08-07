//! Policy for sending welcome emails to new persons

use async_trait::async_trait;
use cim_domain::DomainResult;

use crate::aggregate::ComponentType;
use crate::commands::{PersonCommand, AddComponent};
use crate::events::PersonEventV2;
use super::Policy;

/// Policy that triggers welcome email for new persons
pub struct WelcomeEmailPolicy;

impl Default for WelcomeEmailPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl WelcomeEmailPolicy {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Policy for WelcomeEmailPolicy {
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>> {
        match event {
            PersonEventV2::Created { person_id, name, .. } => {
                // Generate a command to send welcome email
                // In a real system, this might trigger an email service
                Ok(vec![
                    PersonCommand::AddComponent(AddComponent {
                        person_id: *person_id,
                        component_type: ComponentType::CustomAttribute,
                        data: serde_json::json!({
                            "type": "notification",
                            "action": "send_welcome_email",
                            "recipient_name": name.full_name(),
                            "template": "welcome_new_person",
                            "scheduled_at": chrono::Utc::now(),
                        }),
                    })
                ])
            }
            PersonEventV2::ComponentAdded { person_id, component_type, component_data, .. } => {
                // If email was just added, send welcome email
                if matches!(component_type, ComponentType::EmailAddress) {
                    if let Some(email) = component_data.get("email").and_then(|v| v.as_str()) {
                        return Ok(vec![
                            PersonCommand::AddComponent(AddComponent {
                                person_id: *person_id,
                                component_type: ComponentType::CustomAttribute,
                                data: serde_json::json!({
                                    "type": "notification",
                                    "action": "send_welcome_email",
                                    "recipient_email": email,
                                    "template": "welcome_email_added",
                                    "scheduled_at": chrono::Utc::now(),
                                }),
                            })
                        ]);
                    }
                }
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }
    
    fn name(&self) -> &str {
        "WelcomeEmail"
    }
    
    fn applies_to(&self, event: &PersonEventV2) -> bool {
        matches!(
            event,
            PersonEventV2::Created { .. } | 
            PersonEventV2::ComponentAdded { .. }
        )
    }
}