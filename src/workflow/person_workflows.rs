//! Pre-defined workflows for common person domain processes
//!
//! This module provides ready-to-use workflow definitions for typical
//! person-related business processes like onboarding, verification, etc.

use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde_json;

use super::definitions::*;

/// Create a person onboarding workflow
pub fn create_person_onboarding_workflow() -> WorkflowDefinition {
    let workflow_id = WorkflowId::new();
    
    let nodes = vec![
        // Start node
        WorkflowNode {
            id: "start".to_string(),
            name: "Start Onboarding".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "PersonService".to_string(),
                operation: "initialize_onboarding".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // Validate identity
        WorkflowNode {
            id: "validate_identity".to_string(),
            name: "Validate Identity".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "IdentityService".to_string(),
                operation: "validate_identity".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(60)),
            retry_policy: Some(RetryPolicy {
                max_retries: 3,
                initial_backoff: std::time::Duration::from_millis(500),
                max_backoff: std::time::Duration::from_secs(10),
                multiplier: 2.0,
                retry_conditions: vec![RetryCondition::AnyError],
            }),
        },
        
        // Decision: Identity valid?
        WorkflowNode {
            id: "identity_valid_check".to_string(),
            name: "Check Identity Validity".to_string(),
            node_type: NodeType::DecisionGateway {
                condition: ConditionExpression::Comparison {
                    left: "identity_valid".to_string(),
                    operator: ComparisonOperator::Equal,
                    right: serde_json::json!(true),
                },
                branches: vec![
                    Branch {
                        name: "valid".to_string(),
                        condition: Some(ConditionExpression::Boolean(true)),
                        target_node_id: "create_profile".to_string(),
                    },
                    Branch {
                        name: "invalid".to_string(),
                        condition: Some(ConditionExpression::Boolean(false)),
                        target_node_id: "identity_verification_failed".to_string(),
                    },
                ],
            },
            configuration: NodeConfiguration::default(),
            timeout: None,
            retry_policy: None,
        },
        
        // Create profile
        WorkflowNode {
            id: "create_profile".to_string(),
            name: "Create Person Profile".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "PersonService".to_string(),
                operation: "create_profile".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // Setup preferences
        WorkflowNode {
            id: "setup_preferences".to_string(),
            name: "Setup User Preferences".to_string(),
            node_type: NodeType::HumanTask {
                assignee: None,
                form_definition: Some("preferences_form".to_string()),
                due_date: Some(Utc::now() + Duration::days(7)),
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(604800)), // 7 days
            retry_policy: None,
        },
        
        // Complete onboarding
        WorkflowNode {
            id: "complete_onboarding".to_string(),
            name: "Complete Onboarding".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "PersonService".to_string(),
                operation: "complete_onboarding".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // Identity verification failed
        WorkflowNode {
            id: "identity_verification_failed".to_string(),
            name: "Identity Verification Failed".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "NotificationService".to_string(),
                operation: "send_failure_notification".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // End nodes
        WorkflowNode {
            id: "onboarding_complete".to_string(),
            name: "Onboarding Complete".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "PersonService".to_string(),
                operation: "finalize_onboarding".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        WorkflowNode {
            id: "onboarding_failed".to_string(),
            name: "Onboarding Failed".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "PersonService".to_string(),
                operation: "cleanup_failed_onboarding".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
    ];
    
    let transitions = vec![
        WorkflowTransition {
            from: "start".to_string(),
            to: "validate_identity".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "validate_identity".to_string(),
            to: "identity_valid_check".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "identity_valid_check".to_string(),
            to: "create_profile".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "identity_valid".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!(true),
            }),
            priority: 2,
        },
        WorkflowTransition {
            from: "identity_valid_check".to_string(),
            to: "identity_verification_failed".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "identity_valid".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!(false),
            }),
            priority: 1,
        },
        WorkflowTransition {
            from: "create_profile".to_string(),
            to: "setup_preferences".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "setup_preferences".to_string(),
            to: "complete_onboarding".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "complete_onboarding".to_string(),
            to: "onboarding_complete".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "identity_verification_failed".to_string(),
            to: "onboarding_failed".to_string(),
            condition: None,
            priority: 1,
        },
    ];
    
    WorkflowDefinition {
        id: workflow_id,
        name: "Person Onboarding".to_string(),
        version: "1.0.0".to_string(),
        workflow_type: PersonWorkflowType::PersonOnboarding,
        description: Some("Complete person onboarding process with identity verification".to_string()),
        nodes,
        transitions,
        start_node_id: "start".to_string(),
        end_node_ids: vec!["onboarding_complete".to_string(), "onboarding_failed".to_string()],
        global_config: WorkflowGlobalConfig::default(),
        metadata: WorkflowMetadata {
            created_by: "PersonDomainSystem".to_string(),
            tags: vec!["onboarding".to_string(), "identity".to_string(), "verification".to_string()],
            ..Default::default()
        },
    }
}

/// Create an employment lifecycle workflow
pub fn create_employment_lifecycle_workflow() -> WorkflowDefinition {
    let workflow_id = WorkflowId::new();
    
    let nodes = vec![
        // Start employment process
        WorkflowNode {
            id: "start_employment".to_string(),
            name: "Start Employment Process".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "EmploymentService".to_string(),
                operation: "initialize_employment".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // Background check
        WorkflowNode {
            id: "background_check".to_string(),
            name: "Conduct Background Check".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "BackgroundCheckService".to_string(),
                operation: "conduct_check".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(300)), // 5 minutes
            retry_policy: Some(RetryPolicy {
                max_retries: 2,
                initial_backoff: std::time::Duration::from_secs(1),
                max_backoff: std::time::Duration::from_secs(30),
                multiplier: 2.0,
                retry_conditions: vec![RetryCondition::Timeout, RetryCondition::AnyError],
            }),
        },
        
        // Wait for HR approval
        WorkflowNode {
            id: "hr_approval".to_string(),
            name: "HR Approval".to_string(),
            node_type: NodeType::HumanTask {
                assignee: Some("hr-manager".to_string()),
                form_definition: Some("employment_approval_form".to_string()),
                due_date: Some(Utc::now() + Duration::days(3)),
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(259200)), // 3 days
            retry_policy: None,
        },
        
        // Create employment record
        WorkflowNode {
            id: "create_employment_record".to_string(),
            name: "Create Employment Record".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "EmploymentService".to_string(),
                operation: "create_employment_record".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(60)),
            retry_policy: None,
        },
        
        // Setup employee systems
        WorkflowNode {
            id: "setup_employee_systems".to_string(),
            name: "Setup Employee Systems".to_string(),
            node_type: NodeType::ParallelGateway {
                branches: vec![
                    ParallelBranch {
                        name: "it_provisioning".to_string(),
                        nodes: vec!["provision_it_access".to_string()],
                    },
                    ParallelBranch {
                        name: "benefits_enrollment".to_string(),
                        nodes: vec!["enroll_benefits".to_string()],
                    },
                    ParallelBranch {
                        name: "payroll_setup".to_string(),
                        nodes: vec!["setup_payroll".to_string()],
                    },
                ],
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(1800)), // 30 minutes
            retry_policy: None,
        },
        
        // Complete employment setup
        WorkflowNode {
            id: "complete_employment".to_string(),
            name: "Complete Employment Setup".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "EmploymentService".to_string(),
                operation: "complete_employment_setup".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // End node
        WorkflowNode {
            id: "employment_active".to_string(),
            name: "Employment Active".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "NotificationService".to_string(),
                operation: "send_welcome_notification".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
    ];
    
    let transitions = vec![
        WorkflowTransition {
            from: "start_employment".to_string(),
            to: "background_check".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "background_check".to_string(),
            to: "hr_approval".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "background_check_passed".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!(true),
            }),
            priority: 1,
        },
        WorkflowTransition {
            from: "hr_approval".to_string(),
            to: "create_employment_record".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "hr_approved".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!(true),
            }),
            priority: 1,
        },
        WorkflowTransition {
            from: "create_employment_record".to_string(),
            to: "setup_employee_systems".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "setup_employee_systems".to_string(),
            to: "complete_employment".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "complete_employment".to_string(),
            to: "employment_active".to_string(),
            condition: None,
            priority: 1,
        },
    ];
    
    WorkflowDefinition {
        id: workflow_id,
        name: "Employment Lifecycle".to_string(),
        version: "1.0.0".to_string(),
        workflow_type: PersonWorkflowType::EmploymentLifecycle,
        description: Some("Complete employment lifecycle from hiring to active employment".to_string()),
        nodes,
        transitions,
        start_node_id: "start_employment".to_string(),
        end_node_ids: vec!["employment_active".to_string()],
        global_config: WorkflowGlobalConfig::default(),
        metadata: WorkflowMetadata {
            created_by: "PersonDomainSystem".to_string(),
            tags: vec!["employment".to_string(), "hiring".to_string(), "hr".to_string()],
            ..Default::default()
        },
    }
}

/// Create a skills certification workflow
pub fn create_skills_certification_workflow() -> WorkflowDefinition {
    let workflow_id = WorkflowId::new();
    
    let nodes = vec![
        // Start certification process
        WorkflowNode {
            id: "start_certification".to_string(),
            name: "Start Skills Certification".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "SkillsService".to_string(),
                operation: "initialize_certification".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // Skill assessment
        WorkflowNode {
            id: "skill_assessment".to_string(),
            name: "Conduct Skill Assessment".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "AssessmentService".to_string(),
                operation: "conduct_assessment".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(1800)), // 30 minutes
            retry_policy: None,
        },
        
        // Peer review
        WorkflowNode {
            id: "peer_review".to_string(),
            name: "Peer Review".to_string(),
            node_type: NodeType::HumanTask {
                assignee: None, // Will be assigned based on skill domain
                form_definition: Some("peer_review_form".to_string()),
                due_date: Some(Utc::now() + Duration::days(5)),
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(432000)), // 5 days
            retry_policy: None,
        },
        
        // Issue certification
        WorkflowNode {
            id: "issue_certification".to_string(),
            name: "Issue Certification".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "CertificationService".to_string(),
                operation: "issue_certificate".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(60)),
            retry_policy: None,
        },
        
        // Update skill profile
        WorkflowNode {
            id: "update_skill_profile".to_string(),
            name: "Update Skill Profile".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "SkillsService".to_string(),
                operation: "update_skill_profile".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // End node
        WorkflowNode {
            id: "certification_complete".to_string(),
            name: "Certification Complete".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "NotificationService".to_string(),
                operation: "send_certification_notification".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
    ];
    
    let transitions = vec![
        WorkflowTransition {
            from: "start_certification".to_string(),
            to: "skill_assessment".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "skill_assessment".to_string(),
            to: "peer_review".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "assessment_passed".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!(true),
            }),
            priority: 1,
        },
        WorkflowTransition {
            from: "peer_review".to_string(),
            to: "issue_certification".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "peer_review_approved".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!(true),
            }),
            priority: 1,
        },
        WorkflowTransition {
            from: "issue_certification".to_string(),
            to: "update_skill_profile".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "update_skill_profile".to_string(),
            to: "certification_complete".to_string(),
            condition: None,
            priority: 1,
        },
    ];
    
    WorkflowDefinition {
        id: workflow_id,
        name: "Skills Certification".to_string(),
        version: "1.0.0".to_string(),
        workflow_type: PersonWorkflowType::SkillsCertification,
        description: Some("Skills assessment and certification workflow".to_string()),
        nodes,
        transitions,
        start_node_id: "start_certification".to_string(),
        end_node_ids: vec!["certification_complete".to_string()],
        global_config: WorkflowGlobalConfig::default(),
        metadata: WorkflowMetadata {
            created_by: "PersonDomainSystem".to_string(),
            tags: vec!["skills".to_string(), "certification".to_string(), "assessment".to_string()],
            ..Default::default()
        },
    }
}

/// Create a privacy compliance workflow (GDPR data request)
pub fn create_privacy_compliance_workflow() -> WorkflowDefinition {
    let workflow_id = WorkflowId::new();
    
    let nodes = vec![
        // Start privacy request processing
        WorkflowNode {
            id: "start_privacy_request".to_string(),
            name: "Start Privacy Request Processing".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "PrivacyService".to_string(),
                operation: "initialize_request".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // Validate request
        WorkflowNode {
            id: "validate_request".to_string(),
            name: "Validate Privacy Request".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "PrivacyService".to_string(),
                operation: "validate_request".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(60)),
            retry_policy: None,
        },
        
        // Decision: What type of request?
        WorkflowNode {
            id: "request_type_check".to_string(),
            name: "Determine Request Type".to_string(),
            node_type: NodeType::DecisionGateway {
                condition: ConditionExpression::Boolean(true),
                branches: vec![
                    Branch {
                        name: "data_export".to_string(),
                        condition: Some(ConditionExpression::Comparison {
                            left: "request_type".to_string(),
                            operator: ComparisonOperator::Equal,
                            right: serde_json::json!("export"),
                        }),
                        target_node_id: "export_data".to_string(),
                    },
                    Branch {
                        name: "data_deletion".to_string(),
                        condition: Some(ConditionExpression::Comparison {
                            left: "request_type".to_string(),
                            operator: ComparisonOperator::Equal,
                            right: serde_json::json!("deletion"),
                        }),
                        target_node_id: "delete_data".to_string(),
                    },
                ],
            },
            configuration: NodeConfiguration::default(),
            timeout: None,
            retry_policy: None,
        },
        
        // Export data
        WorkflowNode {
            id: "export_data".to_string(),
            name: "Export Personal Data".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "DataExportService".to_string(),
                operation: "export_personal_data".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(600)), // 10 minutes
            retry_policy: Some(RetryPolicy {
                max_retries: 3,
                initial_backoff: std::time::Duration::from_secs(30),
                max_backoff: std::time::Duration::from_secs(300),
                multiplier: 2.0,
                retry_conditions: vec![RetryCondition::AnyError],
            }),
        },
        
        // Delete data
        WorkflowNode {
            id: "delete_data".to_string(),
            name: "Delete Personal Data".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "DataDeletionService".to_string(),
                operation: "delete_personal_data".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(300)), // 5 minutes
            retry_policy: Some(RetryPolicy {
                max_retries: 2,
                initial_backoff: std::time::Duration::from_secs(10),
                max_backoff: std::time::Duration::from_secs(60),
                multiplier: 2.0,
                retry_conditions: vec![RetryCondition::AnyError],
            }),
        },
        
        // Notify completion
        WorkflowNode {
            id: "notify_completion".to_string(),
            name: "Notify Request Completion".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "NotificationService".to_string(),
                operation: "send_privacy_completion_notification".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
        
        // End node
        WorkflowNode {
            id: "privacy_request_complete".to_string(),
            name: "Privacy Request Complete".to_string(),
            node_type: NodeType::ServiceInvocation {
                service: "PrivacyService".to_string(),
                operation: "complete_request".to_string(),
                input_mapping: None,
                output_mapping: None,
            },
            configuration: NodeConfiguration::default(),
            timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: None,
        },
    ];
    
    let transitions = vec![
        WorkflowTransition {
            from: "start_privacy_request".to_string(),
            to: "validate_request".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "validate_request".to_string(),
            to: "request_type_check".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "request_valid".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!(true),
            }),
            priority: 1,
        },
        WorkflowTransition {
            from: "request_type_check".to_string(),
            to: "export_data".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "request_type".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!("export"),
            }),
            priority: 2,
        },
        WorkflowTransition {
            from: "request_type_check".to_string(),
            to: "delete_data".to_string(),
            condition: Some(ConditionExpression::Comparison {
                left: "request_type".to_string(),
                operator: ComparisonOperator::Equal,
                right: serde_json::json!("deletion"),
            }),
            priority: 1,
        },
        WorkflowTransition {
            from: "export_data".to_string(),
            to: "notify_completion".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "delete_data".to_string(),
            to: "notify_completion".to_string(),
            condition: None,
            priority: 1,
        },
        WorkflowTransition {
            from: "notify_completion".to_string(),
            to: "privacy_request_complete".to_string(),
            condition: None,
            priority: 1,
        },
    ];
    
    WorkflowDefinition {
        id: workflow_id,
        name: "Privacy Compliance (GDPR)".to_string(),
        version: "1.0.0".to_string(),
        workflow_type: PersonWorkflowType::PrivacyCompliance,
        description: Some("GDPR compliance workflow for data export and deletion requests".to_string()),
        nodes,
        transitions,
        start_node_id: "start_privacy_request".to_string(),
        end_node_ids: vec!["privacy_request_complete".to_string()],
        global_config: WorkflowGlobalConfig {
            max_execution_time: Some(std::time::Duration::from_secs(2592000)), // 30 days (GDPR requirement)
            ..Default::default()
        },
        metadata: WorkflowMetadata {
            created_by: "PersonDomainSystem".to_string(),
            tags: vec!["privacy".to_string(), "gdpr".to_string(), "compliance".to_string()],
            ..Default::default()
        },
    }
}

/// Get all predefined workflows
pub fn get_predefined_workflows() -> Vec<WorkflowDefinition> {
    vec![
        create_person_onboarding_workflow(),
        create_employment_lifecycle_workflow(),
        create_skills_certification_workflow(),
        create_privacy_compliance_workflow(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_onboarding_workflow_creation() {
        let workflow = create_person_onboarding_workflow();
        assert_eq!(workflow.name, "Person Onboarding");
        assert_eq!(workflow.workflow_type, PersonWorkflowType::PersonOnboarding);
        assert!(!workflow.nodes.is_empty());
        assert!(!workflow.transitions.is_empty());
        assert_eq!(workflow.start_node_id, "start");
        assert!(workflow.end_node_ids.contains(&"onboarding_complete".to_string()));
    }
    
    #[test]
    fn test_employment_workflow_creation() {
        let workflow = create_employment_lifecycle_workflow();
        assert_eq!(workflow.name, "Employment Lifecycle");
        assert_eq!(workflow.workflow_type, PersonWorkflowType::EmploymentLifecycle);
        assert!(!workflow.nodes.is_empty());
        assert!(!workflow.transitions.is_empty());
    }
    
    #[test]
    fn test_skills_certification_workflow_creation() {
        let workflow = create_skills_certification_workflow();
        assert_eq!(workflow.name, "Skills Certification");
        assert_eq!(workflow.workflow_type, PersonWorkflowType::SkillsCertification);
        assert!(!workflow.nodes.is_empty());
        assert!(!workflow.transitions.is_empty());
    }
    
    #[test]
    fn test_privacy_compliance_workflow_creation() {
        let workflow = create_privacy_compliance_workflow();
        assert_eq!(workflow.name, "Privacy Compliance (GDPR)");
        assert_eq!(workflow.workflow_type, PersonWorkflowType::PrivacyCompliance);
        
        // Check GDPR-specific configuration
        assert_eq!(
            workflow.global_config.max_execution_time,
            Some(std::time::Duration::from_secs(2592000)) // 30 days
        );
        
        assert!(workflow.metadata.tags.contains(&"gdpr".to_string()));
        assert!(!workflow.nodes.is_empty());
        assert!(!workflow.transitions.is_empty());
    }
    
    #[test]
    fn test_predefined_workflows_collection() {
        let workflows = get_predefined_workflows();
        assert_eq!(workflows.len(), 4);
        
        let workflow_types: Vec<_> = workflows.iter()
            .map(|w| &w.workflow_type)
            .collect();
        
        assert!(workflow_types.contains(&&PersonWorkflowType::PersonOnboarding));
        assert!(workflow_types.contains(&&PersonWorkflowType::EmploymentLifecycle));
        assert!(workflow_types.contains(&&PersonWorkflowType::SkillsCertification));
        assert!(workflow_types.contains(&&PersonWorkflowType::PrivacyCompliance));
    }
}