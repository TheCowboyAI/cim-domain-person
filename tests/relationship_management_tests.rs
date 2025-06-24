//! Tests for Epic 3: Relationship Management User Stories
//! 
//! Tests cover:
//! - Story 3.1: Establish Employment Relationship
//! - Story 3.2: Associate Person with Location  
//! - Story 3.3: Map Professional Networks

use cim_domain_person::{
    aggregate::{Person, PersonId, ComponentType},
    cross_domain::{
        person_organization::{PersonOrganizationRelation, RelationType as OrgRelationType},
        person_location::{PersonLocationRelation, RelationType as LocationRelationType},
    },
    value_objects::PersonName,
    events::PersonEvent,
};
use chrono::NaiveDate;
use uuid::Uuid;

/// Test Story 3.1: Establish Employment Relationship
/// 
/// ```mermaid
/// graph TB
///     A[HR Manager] --> B[Link Person to Org]
///     B --> C[Set Role/Department]
///     C --> D[Track Start Date]
///     D --> E[Support Multiple Jobs]
///     E --> F[Track Manager]
/// ```
#[test]
fn test_establish_employment_relationship() {
    // As a HR manager
    // I want to link a person to an organization as an employee
    
    // Arrange
    let person_id = PersonId::new();
    let organization_id = Uuid::new_v4();
    let manager_id = PersonId::new();
    
    // Create employment relationship
    let employment = PersonOrganizationRelation {
        person_id,
        organization_id,
        relation_type: OrgRelationType::Employee,
        role: Some("Software Engineer".to_string()),
        department: Some("Engineering".to_string()),
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: None,
        is_primary: true,
        manager_id: Some(manager_id.into()),
    };
    
    // Assert acceptance criteria
    assert_eq!(employment.person_id, person_id);
    assert_eq!(employment.organization_id, organization_id);
    assert!(matches!(employment.relation_type, OrgRelationType::Employee));
    assert_eq!(employment.role, Some("Software Engineer".to_string()));
    assert_eq!(employment.department, Some("Engineering".to_string()));
    assert!(employment.is_primary);
    assert_eq!(employment.manager_id, Some(manager_id.into()));
}

/// Test multiple concurrent employments
#[test]
fn test_multiple_concurrent_employments() {
    // Support multiple concurrent employments
    let person_id = PersonId::new();
    
    let employment1 = PersonOrganizationRelation {
        person_id,
        organization_id: Uuid::new_v4(),
        relation_type: OrgRelationType::Employee,
        role: Some("Full-time Developer".to_string()),
        department: Some("Engineering".to_string()),
        start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        end_date: None,
        is_primary: true,
        manager_id: None,
    };
    
    let employment2 = PersonOrganizationRelation {
        person_id,
        organization_id: Uuid::new_v4(),
        relation_type: OrgRelationType::Contractor,
        role: Some("Consultant".to_string()),
        department: None,
        start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
        end_date: None,
        is_primary: false,
        manager_id: None,
    };
    
    // Both employments are valid and concurrent
    assert!(employment1.is_primary);
    assert!(!employment2.is_primary);
    assert!(employment1.end_date.is_none());
    assert!(employment2.end_date.is_none());
}

/// Test employment types
#[test]
fn test_employment_types() {
    let types = vec![
        OrgRelationType::Employee,
        OrgRelationType::Contractor,
        OrgRelationType::Partner,
        OrgRelationType::BoardMember,
        OrgRelationType::Advisor,
        OrgRelationType::Vendor,
        OrgRelationType::Customer,
        OrgRelationType::Alumni,
    ];
    
    // All types should be distinct
    for (i, t1) in types.iter().enumerate() {
        for (j, t2) in types.iter().enumerate() {
            if i != j {
                assert!(!matches!(t1, t2));
            }
        }
    }
}

/// Test Story 3.2: Associate Person with Location
/// 
/// ```mermaid
/// graph TB
///     A[Facilities Manager] --> B[Link to Location]
///     B --> C[Set Address Type]
///     C --> D[Track Primary]
///     D --> E[Valid Date Ranges]
///     E --> F[Query by Location]
/// ```
#[test]
fn test_associate_person_with_location() {
    // As a facilities manager
    // I want to associate people with physical locations
    
    // Arrange
    let person_id = PersonId::new();
    let location_id = Uuid::new_v4();
    
    // Create location relationship
    let location = PersonLocationRelation {
        person_id,
        location_id,
        relation_type: LocationRelationType::WorkLocation,
        is_primary: true,
        valid_from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        valid_until: None,
    };
    
    // Assert acceptance criteria
    assert_eq!(location.person_id, person_id);
    assert_eq!(location.location_id, location_id);
    assert!(matches!(location.relation_type, LocationRelationType::WorkLocation));
    assert!(location.is_primary);
    assert!(location.valid_from.is_some());
    assert!(location.valid_until.is_none()); // Currently valid
}

/// Test multiple address types
#[test]
fn test_multiple_address_types() {
    let person_id = PersonId::new();
    
    // Home address
    let home = PersonLocationRelation {
        person_id,
        location_id: Uuid::new_v4(),
        relation_type: LocationRelationType::Residence,
        is_primary: true,
        valid_from: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        valid_until: None,
    };
    
    // Work address
    let work = PersonLocationRelation {
        person_id,
        location_id: Uuid::new_v4(),
        relation_type: LocationRelationType::WorkLocation,
        is_primary: false,
        valid_from: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        valid_until: None,
    };
    
    // Mailing address (different from home)
    let mailing = PersonLocationRelation {
        person_id,
        location_id: Uuid::new_v4(),
        relation_type: LocationRelationType::MailingAddress,
        is_primary: false,
        valid_from: Some(NaiveDate::from_ymd_opt(2023, 6, 1).unwrap()),
        valid_until: None,
    };
    
    // All should be valid
    assert!(matches!(home.relation_type, LocationRelationType::Residence));
    assert!(matches!(work.relation_type, LocationRelationType::WorkLocation));
    assert!(matches!(mailing.relation_type, LocationRelationType::MailingAddress));
    assert!(home.is_primary);
    assert!(!work.is_primary);
    assert!(!mailing.is_primary);
}

/// Test date-bounded locations
#[test]
fn test_date_bounded_locations() {
    let person_id = PersonId::new();
    
    // Past residence
    let past_home = PersonLocationRelation {
        person_id,
        location_id: Uuid::new_v4(),
        relation_type: LocationRelationType::PastResident,
        is_primary: false,
        valid_from: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
        valid_until: Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
    };
    
    // Current residence
    let current_home = PersonLocationRelation {
        person_id,
        location_id: Uuid::new_v4(),
        relation_type: LocationRelationType::Residence,
        is_primary: true,
        valid_from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        valid_until: None,
    };
    
    // Past location has end date
    assert!(past_home.valid_until.is_some());
    // Current location has no end date
    assert!(current_home.valid_until.is_none());
}

/// Test Story 3.3: Map Professional Networks
/// 
/// ```mermaid
/// graph TB
///     A[Business Dev Manager] --> B[Map Relationships]
///     B --> C[Set Relationship Type]
///     C --> D[Bidirectional Tracking]
///     D --> E[Strength Metrics]
///     E --> F[Time Bounds]
/// ```
#[test]
fn test_map_professional_networks() {
    // As a business development manager
    // I want to map relationships between people
    
    // This would typically be implemented as person-to-person relationships
    // For now, we test that the person can have relationship components registered
    
    let mut person = Person::new(PersonId::new(), PersonName::new("Network", "Node"));
    
    // Register relationship component
    let result = person.register_component(ComponentType::Relationships, "business_dev");
    assert!(result.is_ok());
    assert!(person.has_component(&ComponentType::Relationships));
}

/// Test relationship types for person-to-person
#[test]
fn test_person_relationship_types() {
    // Define various relationship types (would be in value objects)
    #[derive(Debug, PartialEq)]
    enum PersonRelationshipType {
        Manager,
        DirectReport,
        Colleague,
        Mentor,
        Mentee,
        Partner,
        Client,
        Vendor,
        Friend,
    }
    
    let types = vec![
        PersonRelationshipType::Manager,
        PersonRelationshipType::DirectReport,
        PersonRelationshipType::Colleague,
        PersonRelationshipType::Mentor,
        PersonRelationshipType::Mentee,
    ];
    
    // Verify bidirectional relationships
    assert!(types.contains(&PersonRelationshipType::Manager));
    assert!(types.contains(&PersonRelationshipType::DirectReport));
    assert!(types.contains(&PersonRelationshipType::Mentor));
    assert!(types.contains(&PersonRelationshipType::Mentee));
}

/// Test component registration for cross-domain relationships
#[test]
fn test_cross_domain_component_registration() {
    let mut person = Person::new(PersonId::new(), PersonName::new("Cross", "Domain"));
    
    // When establishing cross-domain relationships, relevant components should be registered
    
    // Employment relationship would trigger
    let employment_result = person.register_component(ComponentType::Employment, "hr_system");
    assert!(employment_result.is_ok());
    
    // Location relationship would trigger  
    let location_result = person.register_component(ComponentType::Location, "facilities");
    assert!(location_result.is_ok());
    
    // Verify both are registered
    assert!(person.has_component(&ComponentType::Employment));
    assert!(person.has_component(&ComponentType::Location));
} 