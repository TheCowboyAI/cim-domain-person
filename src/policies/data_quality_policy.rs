//! Policy for ensuring data quality

use async_trait::async_trait;
use cim_domain::DomainResult;
use regex::Regex;

use crate::aggregate::ComponentType;
use crate::commands::{PersonCommand, UpdateComponent};
use crate::events::PersonEventV2;
use super::Policy;

/// Policy that validates and standardizes data
pub struct DataQualityPolicy {
    email_regex: Regex,
    phone_regex: Regex,
}

impl DataQualityPolicy {
    pub fn new() -> Self {
        Self {
            email_regex: Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap(),
            phone_regex: Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap(),
        }
    }
}

#[async_trait]
impl Policy for DataQualityPolicy {
    async fn evaluate(&self, event: &PersonEventV2) -> DomainResult<Vec<PersonCommand>> {
        match event {
            PersonEventV2::ComponentAdded { person_id, component_type, component_data, .. } => {
                match component_type {
                    ComponentType::EmailAddress => {
                        if let Some(email) = component_data.get("email").and_then(|v| v.as_str()) {
                            // Validate email format
                            if !self.email_regex.is_match(email) {
                                // Generate command to mark as invalid
                                return Ok(vec![
                                    PersonCommand::UpdateComponent(UpdateComponent {
                                        person_id: *person_id,
                                        component_id: uuid::Uuid::new_v4(), // Would need actual ID
                                        component_type: ComponentType::EmailAddress,
                                        updates: serde_json::json!({
                                            "validation_status": "invalid",
                                            "validation_error": "Invalid email format",
                                            "validated_at": chrono::Utc::now(),
                                        }),
                                    })
                                ]);
                            }
                        }
                    }
                    ComponentType::PhoneNumber => {
                        if let Some(phone) = component_data.get("phone_number").and_then(|v| v.as_str()) {
                            // Validate phone format using regex
                            if !self.phone_regex.is_match(phone) {
                                // Try to clean and validate the phone number
                                let cleaned = phone.chars()
                                    .filter(|c| c.is_numeric() || *c == '+')
                                    .collect::<String>();
                                
                                if self.phone_regex.is_match(&cleaned) {
                                    // Phone can be cleaned and is valid
                                    return Ok(vec![
                                        PersonCommand::UpdateComponent(UpdateComponent {
                                            person_id: *person_id,
                                            component_id: uuid::Uuid::new_v4(), // Would need actual ID
                                            component_type: ComponentType::PhoneNumber,
                                            updates: serde_json::json!({
                                                "phone_number": cleaned,
                                                "standardized": true,
                                                "original_format": phone,
                                                "standardized_at": chrono::Utc::now(),
                                            }),
                                        })
                                    ]);
                                } else {
                                    // Invalid phone format even after cleaning
                                    return Ok(vec![
                                        PersonCommand::UpdateComponent(UpdateComponent {
                                            person_id: *person_id,
                                            component_id: uuid::Uuid::new_v4(),
                                            component_type: ComponentType::PhoneNumber,
                                            updates: serde_json::json!({
                                                "validation_status": "invalid",
                                                "validation_error": "Invalid phone format",
                                                "validated_at": chrono::Utc::now(),
                                            }),
                                        })
                                    ]);
                                }
                            }
                            
                            // Standardize phone format even if valid
                            let standardized = standardize_phone_number(phone);
                            
                            if standardized != phone {
                                return Ok(vec![
                                    PersonCommand::UpdateComponent(UpdateComponent {
                                        person_id: *person_id,
                                        component_id: uuid::Uuid::new_v4(), // Would need actual ID
                                        component_type: ComponentType::PhoneNumber,
                                        updates: serde_json::json!({
                                            "phone_number": standardized,
                                            "standardized": true,
                                            "standardized_at": chrono::Utc::now(),
                                        }),
                                    })
                                ]);
                            }
                        }
                    }
                    _ => {}
                }
                Ok(vec![])
            }
            PersonEventV2::NameUpdated { person_id, new_name, .. } => {
                // Check for potential duplicates or data quality issues
                if new_name.given_name.len() < 2 || new_name.family_name.len() < 2 {
                    // Flag for review
                    return Ok(vec![
                        PersonCommand::AddComponent(crate::commands::AddComponent {
                            person_id: *person_id,
                            component_type: ComponentType::CustomAttribute,
                            data: serde_json::json!({
                                "type": "data_quality_flag",
                                "issue": "Name too short",
                                "field": "name",
                                "severity": "warning",
                                "flagged_at": chrono::Utc::now(),
                            }),
                        })
                    ]);
                }
                Ok(vec![])
            }
            _ => Ok(vec![])
        }
    }
    
    fn name(&self) -> &str {
        "DataQuality"
    }
}

fn standardize_phone_number(phone: &str) -> String {
    // Simple standardization - remove non-digits and ensure + prefix for international
    let digits_only: String = phone.chars()
        .filter(|c| c.is_digit(10) || *c == '+')
        .collect();
    
    if digits_only.starts_with('+') {
        digits_only
    } else if digits_only.len() == 10 {
        // Assume US number
        format!("+1{}", digits_only)
    } else {
        digits_only
    }
}