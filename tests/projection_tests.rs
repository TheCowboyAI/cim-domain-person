//! Tests for person projections

use cim_domain_person::{
    aggregate::PersonId,
    events::*,
    projections::*,
    value_objects::{PersonName, EmailAddress},
    components::data::{
        ComponentData, ContactData, EmailComponentData, EmailType,
        ProfessionalData, SkillsData, Skill, SkillCategory, ProficiencyLevel,
        SocialData, RelationshipData
    },
};
use chrono::Utc;

#[tokio::test]
async fn test_person_summary_projection() {
    let projection = PersonSummaryProjection::new();
    
    // Create a person
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let created_event = PersonCreated {
        person_id,
        name: name.clone(),
        created_at: Utc::now(),
        source: "test".to_string(),
    };
    
    // Process creation event
    projection.handle_event(&PersonEvent::PersonCreated(created_event)).await.unwrap();
    
    // Get summary
    let summary = projection.get_summary(&person_id).await.unwrap();
    assert_eq!(summary.person_id, person_id);
    assert_eq!(summary.full_name, "John Doe");
    assert!(summary.primary_email.is_none());
    assert!(summary.primary_phone.is_none());
    assert_eq!(summary.component_count, 0);
    assert_eq!(summary.skills_count, 0);
}

#[tokio::test]
async fn test_person_search_projection() {
    let projection = PersonSearchProjection::new();
    
    // Create multiple persons
    let person1_id = PersonId::new();
    let person1_name = PersonName::new("John".to_string(), "Smith".to_string());
    let person1_created = PersonCreated {
        person_id: person1_id,
        name: person1_name.clone(),
        created_at: Utc::now(),
        source: "test".to_string(),
    };
    
    let person2_id = PersonId::new();
    let person2_name = PersonName::new("Jane".to_string(), "Johnson".to_string());
    let person2_created = PersonCreated {
        person_id: person2_id,
        name: person2_name.clone(),
        created_at: Utc::now(),
        source: "test".to_string(),
    };
    
    // Process creation events
    projection.handle_event(&PersonEvent::PersonCreated(person1_created)).await.unwrap();
    projection.handle_event(&PersonEvent::PersonCreated(person2_created)).await.unwrap();
    
    // Search by name
    let results = projection.search("John", 10).await;
    assert_eq!(results.len(), 2);
    
    let results = projection.search("Smith", 10).await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].person_id, person1_id);
    
    let results = projection.search("Jane Johnson", 10).await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].person_id, person2_id);
}

#[tokio::test]
async fn test_person_skills_projection() {
    let projection = PersonSkillsProjection::new();
    
    // Create a person
    let person_id = PersonId::new();
    let name = PersonName::new("Tech".to_string(), "Expert".to_string());
    let created_event = PersonCreated {
        person_id,
        name: name.clone(),
        created_at: Utc::now(),
        source: "test".to_string(),
    };
    
    // Process creation event
    projection.handle_event(&PersonEvent::PersonCreated(created_event)).await.unwrap();
    
    // Add skills via component data
    let skills_data = ComponentData::Professional(ProfessionalData::Skills(SkillsData {
        skills: vec![
            Skill {
                name: "Rust".to_string(),
                category: SkillCategory::Technical,
                proficiency: ProficiencyLevel::Expert,
                years_of_experience: Some(5.0),
                last_used: Some(Utc::now()),
                verified: true,
                endorsements: vec![],
                certifications: vec![],
                projects: vec!["CIM".to_string()],
            },
            Skill {
                name: "Event Sourcing".to_string(),
                category: SkillCategory::Technical,
                proficiency: ProficiencyLevel::Advanced,
                years_of_experience: Some(3.0),
                last_used: Some(Utc::now()),
                verified: false,
                endorsements: vec![],
                certifications: vec![],
                projects: vec![],
            }
        ]
    }));
    
    let component_event = ComponentDataUpdated {
        person_id,
        component_data: skills_data,
        updated_at: Utc::now(),
    };
    
    projection.handle_event(&PersonEvent::ComponentDataUpdated(component_event)).await.unwrap();
    
    // Get skills
    let skills = projection.get_person_skills(&person_id).await.unwrap();
    assert_eq!(skills.len(), 2);
    
    // Get skill recommendations
    let recommendations = projection.get_skill_recommendations(&person_id).await;
    assert!(!recommendations.is_empty());
}

#[tokio::test]
async fn test_person_network_projection() {
    let projection = PersonNetworkProjection::new();
    
    // Create two persons
    let person1_id = PersonId::new();
    let person1_name = PersonName::new("Alice".to_string(), "Smith".to_string());
    let person1_created = PersonCreated {
        person_id: person1_id,
        name: person1_name.clone(),
        created_at: Utc::now(),
        source: "test".to_string(),
    };
    
    let person2_id = PersonId::new();
    let person2_name = PersonName::new("Bob".to_string(), "Jones".to_string());
    let person2_created = PersonCreated {
        person_id: person2_id,
        name: person2_name.clone(),
        created_at: Utc::now(),
        source: "test".to_string(),
    };
    
    // Process creation events
    projection.handle_event(&PersonEvent::PersonCreated(person1_created)).await.unwrap();
    projection.handle_event(&PersonEvent::PersonCreated(person2_created)).await.unwrap();
    
    // Add relationship via component data
    let rel_data = ComponentData::Social(SocialData::Relationship(RelationshipData {
        other_person_id: person2_id,
        relationship_type: "colleague".to_string(),
        start_date: Some(Utc::now()),
        end_date: None,
        notes: Some("Works in same department".to_string()),
    }));
    
    let component_event = ComponentDataUpdated {
        person_id: person1_id,
        component_data: rel_data,
        updated_at: Utc::now(),
    };
    
    projection.handle_event(&PersonEvent::ComponentDataUpdated(component_event)).await.unwrap();
    
    // Add reverse relationship
    let rel_data2 = ComponentData::Social(SocialData::Relationship(RelationshipData {
        other_person_id: person1_id,
        relationship_type: "colleague".to_string(),
        start_date: Some(Utc::now()),
        end_date: None,
        notes: Some("Works in same department".to_string()),
    }));
    
    let component_event2 = ComponentDataUpdated {
        person_id: person2_id,
        component_data: rel_data2,
        updated_at: Utc::now(),
    };
    
    projection.handle_event(&PersonEvent::ComponentDataUpdated(component_event2)).await.unwrap();
    
    // Get connections
    let connections = projection.get_connections(&person1_id).await;
    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0], person2_id);
    
    // Check network analysis
    let analysis = projection.get_network_analysis(&person1_id).await.unwrap();
    assert_eq!(analysis.total_connections, 1);
    assert_eq!(analysis.connection_types["colleague"], 1);
}

#[tokio::test]
async fn test_person_timeline_projection() {
    let projection = PersonTimelineProjection::new();
    
    // Create a person
    let person_id = PersonId::new();
    let name = PersonName::new("Timeline".to_string(), "Test".to_string());
    let created_at = Utc::now();
    let created_event = PersonCreated {
        person_id,
        name: name.clone(),
        created_at,
        source: "test".to_string(),
    };
    
    // Process creation event
    projection.handle_event(&PersonEvent::PersonCreated(created_event)).await.unwrap();
    
    // Update person
    let updated_event = PersonUpdated {
        person_id,
        name: PersonName::new("Timeline".to_string(), "Updated".to_string()),
        updated_at: Utc::now(),
    };
    
    projection.handle_event(&PersonEvent::PersonUpdated(updated_event)).await.unwrap();
    
    // Add email component
    let email_data = ComponentData::Contact(ContactData::Email(EmailComponentData {
        email: EmailAddress::new("timeline@test.com".to_string()).unwrap(),
        email_type: EmailType::Personal,
        is_preferred_contact: true,
        can_receive_notifications: true,
        can_receive_marketing: false,
    }));
    
    let component_event = ComponentDataUpdated {
        person_id,
        component_data: email_data,
        updated_at: Utc::now(),
    };
    
    projection.handle_event(&PersonEvent::ComponentDataUpdated(component_event)).await.unwrap();
    
    // Get timeline
    let timeline = projection.get_timeline(&person_id, None).await;
    assert_eq!(timeline.len(), 3); // Created, Updated, Component Added
    
    // Verify timeline order (most recent first)
    assert!(timeline[0].timestamp > timeline[1].timestamp);
    assert!(timeline[1].timestamp > timeline[2].timestamp);
}

#[tokio::test]
async fn test_query_service_integration() {
    // Create projections
    let summary_proj = PersonSummaryProjection::new();
    let search_proj = PersonSearchProjection::new();
    let skills_proj = PersonSkillsProjection::new();
    let network_proj = PersonNetworkProjection::new();
    let timeline_proj = PersonTimelineProjection::new();
    
    // Create query service
    let query_service = PersonQueryService::new(
        summary_proj.clone(),
        search_proj.clone(),
        skills_proj.clone(),
        network_proj.clone(),
        timeline_proj.clone(),
    );
    
    // Create a person
    let person_id = PersonId::new();
    let name = PersonName::new("Test".to_string(), "User".to_string());
    let created_event = PersonCreated {
        person_id,
        name: name.clone(),
        created_at: Utc::now(),
        source: "test".to_string(),
    };
    
    // Process event in all projections
    summary_proj.handle_event(&PersonEvent::PersonCreated(created_event.clone())).await.unwrap();
    search_proj.handle_event(&PersonEvent::PersonCreated(created_event.clone())).await.unwrap();
    skills_proj.handle_event(&PersonEvent::PersonCreated(created_event.clone())).await.unwrap();
    network_proj.handle_event(&PersonEvent::PersonCreated(created_event.clone())).await.unwrap();
    timeline_proj.handle_event(&PersonEvent::PersonCreated(created_event.clone())).await.unwrap();
    
    // Test query service methods
    assert!(query_service.get_summary(person_id).await.is_some());
    assert!(!query_service.search("Test", 10).await.is_empty());
    assert!(query_service.get_person_skills(person_id).await.is_empty()); // No skills yet
    assert!(query_service.get_connections(person_id).await.is_empty()); // No connections yet
    assert!(!query_service.get_timeline(person_id, None).await.is_empty());
} 