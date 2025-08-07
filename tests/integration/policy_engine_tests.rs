//! Integration tests for policy engine

use cim_domain_person::{
    policies::{PolicyEngine, Policy, create_default_policy_engine},
    events::{PersonEventV2, EventMetadata},
    commands::{PersonCommand, AddComponent},
    aggregate::{PersonId, ComponentType},
    value_objects::PersonName,
};
use std::sync::Arc;
use async_trait::async_trait;

#[tokio::test]
async fn test_default_policies() -> Result<(), Box<dyn std::error::Error>> {
    let engine = create_default_policy_engine();
    
    // Test welcome email policy
    let person_id = PersonId::new();
    let created_event = PersonEventV2::Created {
        person_id,
        name: PersonName::new("Policy", None, "Test")?,
        source: "test".to_string(),
        metadata: EventMetadata::new(),
    };
    
    let commands = engine.evaluate(&created_event).await;
    
    // Should generate welcome email command
    assert!(!commands.is_empty());
    
    let has_welcome_email = commands.iter().any(|cmd| {
        matches!(cmd, PersonCommand::AddComponent(add) if 
            add.data["type"] == "welcome_email"
        )
    });
    assert!(has_welcome_email);
    
    Ok(())
}

#[tokio::test]
async fn test_skill_recommendation_policy() -> Result<(), Box<dyn std::error::Error>> {
    let engine = create_default_policy_engine();
    
    let person_id = PersonId::new();
    let git_profile_added = PersonEventV2::ComponentAdded {
        person_id,
        component_type: ComponentType::CustomAttribute,
        component_data: serde_json::json!({
            "type": "git_profile",
            "languages": ["rust", "python", "go"],
            "repositories": 50,
            "contributions": 2000
        }),
        metadata: EventMetadata::new(),
    };
    
    let commands = engine.evaluate(&git_profile_added).await;
    
    // Should generate skill recommendations
    let skill_commands: Vec<_> = commands.iter()
        .filter_map(|cmd| {
            match cmd {
                PersonCommand::AddComponent(add) if add.data["type"] == "skill" => {
                    Some(&add.data)
                },
                _ => None
            }
        })
        .collect();
    
    assert!(!skill_commands.is_empty());
    
    // Verify Rust skill was recommended
    let has_rust = skill_commands.iter().any(|data| 
        data["skill_name"] == "Rust"
    );
    assert!(has_rust);
    
    Ok(())
}

#[tokio::test]
async fn test_custom_policy() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom policy
    struct TestPolicy {
        counter: Arc<tokio::sync::Mutex<u32>>,
    }
    
    #[async_trait]
    impl Policy for TestPolicy {
        async fn evaluate(&self, event: &PersonEventV2) -> cim_domain::DomainResult<Vec<PersonCommand>> {
            let mut counter = self.counter.lock().await;
            *counter += 1;
            
            match event {
                PersonEventV2::Created { person_id, .. } => {
                    Ok(vec![
                        PersonCommand::AddComponent(AddComponent {
                            person_id: *person_id,
                            component_type: ComponentType::CustomAttribute,
                            data: serde_json::json!({
                                "type": "test_counter",
                                "value": *counter
                            }),
                        })
                    ])
                },
                _ => Ok(vec![])
            }
        }
        
        fn name(&self) -> &str {
            "TestPolicy"
        }
    }
    
    // Create engine with custom policy
    let counter = Arc::new(tokio::sync::Mutex::new(0));
    let mut engine = PolicyEngine::new();
    engine.register(Arc::new(TestPolicy { counter: counter.clone() }));
    
    // Test multiple events
    for i in 0..3 {
        let event = PersonEventV2::Created {
            person_id: PersonId::new(),
            name: PersonName::new(&format!("Test{}", i), None, "User")?,
            source: "test".to_string(),
            metadata: EventMetadata::new(),
        };
        
        let commands = engine.evaluate(&event).await;
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            PersonCommand::AddComponent(add) => {
                assert_eq!(add.data["value"], i + 1);
            },
            _ => panic!("Unexpected command type"),
        }
    }
    
    let final_count = *counter.lock().await;
    assert_eq!(final_count, 3);
    
    Ok(())
}

#[tokio::test]
async fn test_data_quality_policy() -> Result<(), Box<dyn std::error::Error>> {
    let engine = create_default_policy_engine();
    
    let person_id = PersonId::new();
    
    // Test email normalization
    let email_added = PersonEventV2::ComponentAdded {
        person_id,
        component_type: ComponentType::Email,
        component_data: serde_json::json!({
            "email": "Test.User@EXAMPLE.com  ",
            "type": "primary"
        }),
        metadata: EventMetadata::new(),
    };
    
    let commands = engine.evaluate(&email_added).await;
    
    // Should generate update command with normalized email
    let update_commands: Vec<_> = commands.iter()
        .filter_map(|cmd| {
            match cmd {
                PersonCommand::UpdateComponent(update) => Some(update),
                _ => None
            }
        })
        .collect();
    
    assert!(!update_commands.is_empty());
    
    // Check normalized email
    let normalized_email = &update_commands[0].updates["email"];
    assert_eq!(normalized_email, "test.user@example.com");
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_policies_same_event() -> Result<(), Box<dyn std::error::Error>> {
    let engine = create_default_policy_engine();
    
    let person_id = PersonId::new();
    let created_event = PersonEventV2::Created {
        person_id,
        name: PersonName::new("Multi", None, "Policy")?,
        source: "test".to_string(),
        metadata: EventMetadata::new(),
    };
    
    let commands = engine.evaluate(&created_event).await;
    
    // Multiple policies should have generated commands
    assert!(commands.len() > 1);
    
    // Count different command types
    let welcome_emails = commands.iter().filter(|cmd| {
        matches!(cmd, PersonCommand::AddComponent(add) if add.data["type"] == "welcome_email")
    }).count();
    
    let other_commands = commands.len() - welcome_emails;
    
    assert_eq!(welcome_emails, 1);
    assert!(other_commands > 0);
    
    Ok(())
}