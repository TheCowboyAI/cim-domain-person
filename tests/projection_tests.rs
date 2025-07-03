//! Tests for person projections

use chrono::{Utc, NaiveDate};
use cim_domain_person::{
    aggregate::PersonId,
    components::data::{
        ComponentData, ContactData, EmailComponentData, EmailType, ProfessionalData,
        ProficiencyLevel, RelationshipData, Skill, SkillCategory, SkillsData, SocialData,
    },
    events::*,
    projections::*,
    queries::PersonQueryService,
    value_objects::{EmailAddress, PersonName},
};
use uuid::Uuid;

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
    projection
        .handle_event(&PersonEvent::PersonCreated(created_event))
        .await
        .unwrap();

    // Get summary
    let summary = projection.get_summary(&person_id).await.unwrap();
    assert_eq!(summary.person_id, person_id);
    assert_eq!(summary.name, "John Doe");
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
    projection
        .handle_event(&PersonEvent::PersonCreated(person1_created))
        .await
        .unwrap();
    projection
        .handle_event(&PersonEvent::PersonCreated(person2_created))
        .await
        .unwrap();

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
    projection
        .handle_event(&PersonEvent::PersonCreated(created_event))
        .await
        .unwrap();

    // Add skills via component data
    let skills_data = ComponentData::Professional(ProfessionalData::Skills(SkillsData {
        skills: vec![
            Skill {
                name: "Rust".to_string(),
                category: "Technical".to_string(),
                proficiency: "Expert".to_string(),
                years_experience: Some(5.0),
                last_used: Some(Utc::now()),
                endorsement_count: Some(0),
                certifications: vec![],
            },
            Skill {
                name: "Event Sourcing".to_string(),
                category: "Technical".to_string(),
                proficiency: "Advanced".to_string(),
                years_experience: Some(3.0),
                last_used: Some(Utc::now()),
                endorsement_count: Some(0),
                certifications: vec![],
            },
        ],
        specializations: vec![],
    }));

    let component_event = ComponentDataUpdated {
        person_id,
        component_id: Uuid::new_v4(),
        data: skills_data,
        updated_at: Utc::now(),
    };

    projection
        .handle_event(&PersonEvent::ComponentDataUpdated(component_event))
        .await
        .unwrap();

    // Get skills
    let skills = projection.get_person_skills(&person_id).await;
    assert_eq!(skills.len(), 2);

    // Get skill recommendations
    let recommendations = projection.get_skill_recommendations(&person_id, 5).await;
    assert!(recommendations.is_empty());
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
    projection
        .handle_event(&PersonEvent::PersonCreated(person1_created))
        .await
        .unwrap();
    projection
        .handle_event(&PersonEvent::PersonCreated(person2_created))
        .await
        .unwrap();

    // Add relationship via component data
    let rel_data = ComponentData::Social(SocialData::Relationship(RelationshipData {
        other_person_id: person2_id,
        relationship_type: "colleague".to_string(),
        start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: None,
        notes: Some("Works in same department".to_string()),
    }));

    let component_event = ComponentDataUpdated {
        person_id: person1_id,
        component_id: Uuid::new_v4(),
        data: rel_data,
        updated_at: Utc::now(),
    };

    projection
        .handle_event(&PersonEvent::ComponentDataUpdated(component_event))
        .await
        .unwrap();

    // Add reverse relationship
    let rel_data2 = ComponentData::Social(SocialData::Relationship(RelationshipData {
        other_person_id: person1_id,
        relationship_type: "colleague".to_string(),
        start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        end_date: None,
        notes: Some("Works in same department".to_string()),
    }));

    let component_event2 = ComponentDataUpdated {
        person_id: person2_id,
        component_id: Uuid::new_v4(),
        data: rel_data2,
        updated_at: Utc::now(),
    };

    projection
        .handle_event(&PersonEvent::ComponentDataUpdated(component_event2))
        .await
        .unwrap();

    // Get connections
    let connections = projection.get_connections(&person1_id).await;
    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0].to_person, person2_id);

    // Check network analysis
    let analysis = projection.get_network_stats(&person1_id).await;
    assert_eq!(analysis.total_connections, 2);
    assert!(analysis.connections_by_type.values().any(|&count| count == 2));
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
    projection
        .handle_event(&PersonEvent::PersonCreated(created_event))
        .await
        .unwrap();

    // Add name update event
    let name_updated_event = NameUpdated {
        person_id,
        old_name: name.clone(),
        new_name: PersonName::new("Timeline".to_string(), "Updated".to_string()),
        reason: Some("Test update".to_string()),
        updated_at: Utc::now(),
    };

    projection
        .handle_event(&PersonEvent::NameUpdated(name_updated_event))
        .await
        .unwrap();

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
        component_id: Uuid::new_v4(),
        data: email_data,
        updated_at: Utc::now(),
    };

    projection
        .handle_event(&PersonEvent::ComponentDataUpdated(component_event))
        .await
        .unwrap();

    // Get timeline
    let timeline = projection.get_timeline(&person_id, None).await;
    assert_eq!(timeline.len(), 3); // Created, Updated, Component Added

    // Verify timeline order (chronological order when no limit specified)
    assert!(timeline[0].timestamp < timeline[1].timestamp);
    assert!(timeline[1].timestamp < timeline[2].timestamp);
}

#[tokio::test]
async fn test_query_service_integration() {
    // Create projections
    let summary_proj = std::sync::Arc::new(PersonSummaryProjection::new());
    let search_proj = std::sync::Arc::new(PersonSearchProjection::new());
    let skills_proj = std::sync::Arc::new(PersonSkillsProjection::new());
    let network_proj = std::sync::Arc::new(PersonNetworkProjection::new());
    let timeline_proj = std::sync::Arc::new(PersonTimelineProjection::new());

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
    summary_proj
        .handle_event(&PersonEvent::PersonCreated(created_event.clone()))
        .await
        .unwrap();
    search_proj
        .handle_event(&PersonEvent::PersonCreated(created_event.clone()))
        .await
        .unwrap();
    skills_proj
        .handle_event(&PersonEvent::PersonCreated(created_event.clone()))
        .await
        .unwrap();
    network_proj
        .handle_event(&PersonEvent::PersonCreated(created_event.clone()))
        .await
        .unwrap();
    timeline_proj
        .handle_event(&PersonEvent::PersonCreated(created_event.clone()))
        .await
        .unwrap();

    // Test query service methods
    assert!(query_service.get_person_summary(&person_id).await.is_some());
    assert!(!query_service.search_persons("Test", 10).await.is_empty());
    assert!(query_service.get_person_skills(&person_id).await.is_empty()); // No skills yet
    assert!(query_service.get_person_connections(&person_id).await.is_empty()); // No connections yet
    assert!(!query_service.get_person_timeline(&person_id, None).await.is_empty());
}
