//! Example demonstrating event versioning and policy engine

use cim_domain_person::{
    aggregate::PersonId,
    commands::{PersonCommand, CreatePerson},
    events::{
        PersonEventV2, EventMetadata, VersionedEventEnvelope,
        create_event_registry, PersonCreatedV2
    },
    policies::{create_default_policy_engine, Policy},
    value_objects::PersonName,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("=== Event Versioning and Policies Demo ===");
    
    // Example 1: Event versioning
    demo_event_versioning()?;
    
    // Example 2: Policy engine
    demo_policy_engine().await?;
    
    // Example 3: Custom policies
    demo_custom_policy().await?;
    
    Ok(())
}

fn demo_event_versioning() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Event Versioning Demo ---");
    
    // Create event registry
    let registry = create_event_registry();
    
    // Create a v1 event (legacy format)
    let v1_event_data = serde_json::json!({
        "version": "1.0",
        "person_id": "12345",
        "name": {
            "first_name": "Jane",
            "middle_name": "A",
            "last_name": "Smith"
        },
        "source": "legacy_system",
        "created_at": "2023-01-01T00:00:00Z"
    });
    
    info!("V1 Event: {:?}", v1_event_data);
    
    // Migrate to current version
    let migrated = registry.migrate_to_current("PersonCreated", v1_event_data)?;
    info!("Migrated to V2: {:?}", migrated);
    
    // Create a new v2 event directly
    let person_id = PersonId::new();
    let v2_event = PersonCreatedV2 {
        person_id,
        name: PersonName::new("John", Some("Q".to_string()), "Doe")?,
        source: "api".to_string(),
        metadata: EventMetadata::new(),
    };
    
    // Wrap in versioned envelope
    let envelope = VersionedEventEnvelope::new(v2_event, EventMetadata::new())?;
    info!("\nVersioned Envelope: {:?}", envelope);
    
    Ok(())
}

async fn demo_policy_engine() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Policy Engine Demo ---");
    
    // Create default policy engine
    let policy_engine = create_default_policy_engine();
    
    // Test 1: New person created
    let person_id = PersonId::new();
    let created_event = PersonEventV2::Created {
        person_id,
        name: PersonName::new("Alice", None, "Johnson")?,
        source: "web".to_string(),
        metadata: EventMetadata::new(),
    };
    
    info!("Event: Person created");
    let commands = policy_engine.evaluate(&created_event).await;
    info!("Generated {} commands from policies", commands.len());
    for cmd in &commands {
        info!("  - Command: {:?}", cmd);
    }
    
    // Test 2: Email component added
    let email_added = PersonEventV2::ComponentAdded {
        person_id,
        component_type: cim_domain_person::aggregate::ComponentType::Email,
        component_data: serde_json::json!({
            "email": "alice@example.com",
            "type": "primary",
            "verified": false
        }),
        metadata: EventMetadata::new(),
    };
    
    info!("\nEvent: Email added");
    let commands = policy_engine.evaluate(&email_added).await;
    info!("Generated {} commands from policies", commands.len());
    
    // Test 3: Git profile added (triggers skill recommendations)
    let git_profile_added = PersonEventV2::ComponentAdded {
        person_id,
        component_type: cim_domain_person::aggregate::ComponentType::CustomAttribute,
        component_data: serde_json::json!({
            "type": "git_profile",
            "username": "alice-dev",
            "languages": ["rust", "python", "typescript"],
            "repositories": 42,
            "contributions": 1337
        }),
        metadata: EventMetadata::new(),
    };
    
    info!("\nEvent: Git profile added");
    let commands = policy_engine.evaluate(&git_profile_added).await;
    info!("Generated {} commands from policies", commands.len());
    for cmd in &commands {
        match cmd {
            PersonCommand::AddComponent(add) => {
                info!("  - Skill recommendations: {:?}", add.data);
            }
            _ => {}
        }
    }
    
    Ok(())
}

async fn demo_custom_policy() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n--- Custom Policy Demo ---");
    
    // Create a custom policy
    struct BirthdayReminderPolicy;
    
    #[async_trait::async_trait]
    impl Policy for BirthdayReminderPolicy {
        async fn evaluate(&self, event: &PersonEventV2) -> cim_domain::DomainResult<Vec<PersonCommand>> {
            match event {
                PersonEventV2::BirthDateSet { person_id, birth_date, .. } => {
                    // Schedule birthday reminder
                    Ok(vec![
                        PersonCommand::AddComponent(cim_domain_person::commands::AddComponent {
                            person_id: *person_id,
                            component_type: cim_domain_person::aggregate::ComponentType::CustomAttribute,
                            data: serde_json::json!({
                                "type": "scheduled_reminder",
                                "occasion": "birthday",
                                "date": birth_date,
                                "recurring": true,
                                "message": "Birthday reminder"
                            }),
                        })
                    ])
                }
                _ => Ok(vec![])
            }
        }
        
        fn name(&self) -> &str {
            "BirthdayReminder"
        }
    }
    
    // Create policy engine with custom policy
    let mut engine = cim_domain_person::policies::PolicyEngine::new();
    engine.register(Arc::new(BirthdayReminderPolicy));
    
    // Test the policy
    let person_id = PersonId::new();
    let birth_date_set = PersonEventV2::BirthDateSet {
        person_id,
        birth_date: chrono::NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
        metadata: EventMetadata::new(),
    };
    
    info!("Event: Birth date set");
    let commands = engine.evaluate(&birth_date_set).await;
    info!("Generated {} commands from custom policy", commands.len());
    for cmd in &commands {
        info!("  - Command: {:?}", cmd);
    }
    
    Ok(())
}

// Example: Combining versioning and policies
#[allow(dead_code)]
async fn integrated_example() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Receive legacy event
    let legacy_event = serde_json::json!({
        "version": "1.0",
        "person_id": "abc123",
        "name": {
            "first_name": "Test",
            "last_name": "User"
        },
        "source": "import",
        "created_at": "2020-01-01T00:00:00Z"
    });
    
    // 2. Migrate to current version
    let registry = create_event_registry();
    let migrated = registry.migrate_to_current("PersonCreated", legacy_event)?;
    
    // 3. Deserialize to typed event
    let event: PersonCreatedV2 = serde_json::from_value(migrated)?;
    
    // 4. Convert to PersonEventV2 for policy evaluation
    let event_v2 = PersonEventV2::Created {
        person_id: event.person_id,
        name: event.name,
        source: event.source,
        metadata: event.metadata,
    };
    
    // 5. Apply policies
    let policy_engine = create_default_policy_engine();
    let commands = policy_engine.evaluate(&event_v2).await;
    
    // 6. Process generated commands
    for command in commands {
        info!("Processing command: {:?}", command);
        // In real system, these would be sent to command processor
    }
    
    Ok(())
}