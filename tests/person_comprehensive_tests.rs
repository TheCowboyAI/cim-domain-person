//! Comprehensive tests for the Person domain

use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::*,
    events::*,
    value_objects::*,
};
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

#[test]
fn test_person_creation_and_basic_info() {
    let person_id = PersonId::new();
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    
    let mut person = Person::new(person_id, name.clone(), "Test".to_string());
    
    assert_eq!(person.id, person_id);
    assert_eq!(person.name.given_name, "John");
    assert_eq!(person.name.family_name, "Doe");
    assert!(person.is_active);
}

#[test]
fn test_contact_management() {
    let person_id = PersonId::new();
    let name = PersonName::new("Jane".to_string(), "Smith".to_string());
    let mut person = Person::new(person_id, name, "Test".to_string());
    
    // Add email
    let email = EmailAddress::new("jane@example.com".to_string());
    let add_email = AddEmail {
        person_id,
        email: email.clone(),
        primary: true,
    };
    
    let events = person.handle_command(PersonCommand::AddEmail(add_email)).unwrap();
    assert_eq!(events.len(), 1);
    
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.emails.len(), 1);
    assert_eq!(person.primary_email, Some("jane@example.com".to_string()));
    
    // Add phone
    let phone = PhoneNumber::with_country("555-1234".to_string(), "1".to_string());
    let add_phone = AddPhone {
        person_id,
        phone: phone.clone(),
        primary: true,
    };
    
    let events = person.handle_command(PersonCommand::AddPhone(add_phone)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.phones.len(), 1);
    assert_eq!(person.primary_phone, Some("555-1234".to_string()));
    
    // Add address
    let address = PhysicalAddress::new(
        "123 Main St".to_string(),
        "Springfield".to_string(),
        "USA".to_string(),
    );
    let add_address = AddAddress {
        person_id,
        address: address.clone(),
        address_type: AddressType::Home,
    };
    
    let events = person.handle_command(PersonCommand::AddAddress(add_address)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.addresses.len(), 1);
    assert!(person.addresses.contains_key(&AddressType::Home));
}

#[test]
fn test_employment_lifecycle() {
    let person_id = PersonId::new();
    let name = PersonName::new("Bob".to_string(), "Johnson".to_string());
    let mut person = Person::new(person_id, name, "Test".to_string());
    
    let org_id = Uuid::new_v4();
    let employment = Employment {
        organization_id: org_id,
        organization_name: "TechCorp".to_string(),
        department: Some("Engineering".to_string()),
        position: "Software Engineer".to_string(),
        employment_type: EmploymentType::FullTime,
        start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        end_date: None,
        manager_id: None,
    };
    
    // Add employment
    let add_employment = AddEmployment {
        person_id,
        employment: employment.clone(),
    };
    
    let events = person.handle_command(PersonCommand::AddEmployment(add_employment)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.employments.len(), 1);
    assert_eq!(person.current_employment, Some(org_id));
    
    // Update employment
    let update_employment = UpdateEmployment {
        person_id,
        organization_id: org_id,
        updates: EmploymentUpdate {
            department: Some("R&D".to_string()),
            position: Some("Senior Software Engineer".to_string()),
            manager_id: None,
        },
    };
    
    let events = person.handle_command(PersonCommand::UpdateEmployment(update_employment)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    let updated_employment = person.employments.get(&org_id).unwrap();
    assert_eq!(updated_employment.department, Some("R&D".to_string()));
    assert_eq!(updated_employment.position, "Senior Software Engineer");
    
    // End employment
    let end_employment = EndEmployment {
        person_id,
        organization_id: org_id,
        end_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        reason: Some("Career change".to_string()),
    };
    
    let events = person.handle_command(PersonCommand::EndEmployment(end_employment)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    let ended_employment = person.employments.get(&org_id).unwrap();
    assert!(ended_employment.end_date.is_some());
    assert_eq!(person.current_employment, None);
}

#[test]
fn test_skills_and_certifications() {
    let person_id = PersonId::new();
    let name = PersonName::new("Alice".to_string(), "Developer".to_string());
    let mut person = Person::new(person_id, name, "Test".to_string());
    
    // Add skill
    let skill = Skill {
        name: "Rust".to_string(),
        category: "Programming".to_string(),
        proficiency: ProficiencyLevel::Expert,
        years_experience: Some(5.0),
        last_used: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        certifications: vec![],
    };
    
    let add_skill = AddSkill {
        person_id,
        skill: skill.clone(),
    };
    
    let events = person.handle_command(PersonCommand::AddSkill(add_skill)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.skills.len(), 1);
    assert!(person.skills.contains_key("Rust"));
    
    // Update skill
    let update_skill = UpdateSkill {
        person_id,
        skill_name: "Rust".to_string(),
        proficiency: ProficiencyLevel::Expert,
        last_used: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
    };
    
    let events = person.handle_command(PersonCommand::UpdateSkill(update_skill)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    // Add certification
    let certification = Certification {
        name: "AWS Certified Developer".to_string(),
        issuer: "Amazon Web Services".to_string(),
        issue_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
        expiry_date: Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        credential_id: Some("AWS-123456".to_string()),
        verification_url: Some("https://aws.amazon.com/verify/123456".to_string()),
    };
    
    let add_cert = AddCertification {
        person_id,
        certification: certification.clone(),
    };
    
    let events = person.handle_command(PersonCommand::AddCertification(add_cert)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.certifications.len(), 1);
}

#[test]
fn test_relationships() {
    let person_id = PersonId::new();
    let name = PersonName::new("Manager".to_string(), "Smith".to_string());
    let mut person = Person::new(person_id, name, "Test".to_string());
    
    let other_person_id = Uuid::new_v4();
    let relationship = Relationship {
        person_id: other_person_id,
        relationship_type: RelationshipType::DirectReport,
        start_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        end_date: None,
        status: RelationshipStatus::Active,
        notes: Some("Team lead".to_string()),
    };
    
    let add_relationship = AddRelationship {
        person_id,
        relationship: relationship.clone(),
    };
    
    let events = person.handle_command(PersonCommand::AddRelationship(add_relationship)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.relationships.len(), 1);
    assert!(person.relationships.contains_key(&other_person_id));
}

#[test]
fn test_social_media_profiles() {
    let person_id = PersonId::new();
    let name = PersonName::new("Social".to_string(), "User".to_string());
    let mut person = Person::new(person_id, name, "Test".to_string());
    
    let profile = SocialProfile {
        platform: SocialPlatform::LinkedIn,
        username: "socialuser".to_string(),
        profile_url: Some("https://linkedin.com/in/socialuser".to_string()),
        verified: true,
        follower_count: Some(500),
        following_count: Some(200),
        engagement_rate: Some(0.05),
        last_active: Some(Utc::now()),
    };
    
    let add_profile = AddSocialProfile {
        person_id,
        profile: profile.clone(),
    };
    
    let events = person.handle_command(PersonCommand::AddSocialProfile(add_profile)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.social_profiles.len(), 1);
    assert!(person.social_profiles.contains_key(&SocialPlatform::LinkedIn));
    
    // Update profile
    let update_profile = UpdateSocialProfile {
        person_id,
        platform: SocialPlatform::LinkedIn,
        updates: SocialProfileUpdate {
            username: None,
            verified: Some(true),
            follower_count: Some(600),
            engagement_rate: Some(0.06),
        },
    };
    
    let events = person.handle_command(PersonCommand::UpdateSocialProfile(update_profile)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    let updated_profile = person.social_profiles.get(&SocialPlatform::LinkedIn).unwrap();
    assert_eq!(updated_profile.follower_count, Some(600));
    assert_eq!(updated_profile.engagement_rate, Some(0.06));
}

#[test]
fn test_customer_segmentation() {
    let person_id = PersonId::new();
    let name = PersonName::new("VIP".to_string(), "Customer".to_string());
    let mut person = Person::new(person_id, name, "Test".to_string());
    
    let segment = CustomerSegment {
        segment_type: SegmentType::VIP,
        value_tier: ValueTier::Platinum,
        lifecycle_stage: LifecycleStage::Advocacy,
        persona: Some("High-Value Tech Buyer".to_string()),
        attributes: Default::default(),
    };
    
    let set_segment = SetCustomerSegment {
        person_id,
        segment: segment.clone(),
    };
    
    let events = person.handle_command(PersonCommand::SetCustomerSegment(set_segment)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert!(person.customer_segment.is_some());
    let customer_segment = person.customer_segment.unwrap();
    assert_eq!(customer_segment.value_tier, ValueTier::Platinum);
}

#[test]
fn test_tags_and_custom_attributes() {
    let person_id = PersonId::new();
    let name = PersonName::new("Tagged".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name, "Test".to_string());
    
    let tag = Tag {
        name: "Important".to_string(),
        category: "Priority".to_string(),
        added_by: Uuid::new_v4(),
        added_at: Utc::now(),
    };
    
    let add_tag = AddTag {
        person_id,
        tag: tag.clone(),
    };
    
    let events = person.handle_command(PersonCommand::AddTag(add_tag)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.tags.len(), 1);
    
    // Custom attribute
    let attribute = CustomAttribute {
        name: "CustomerID".to_string(),
        value: "CUST-12345".to_string(),
        data_type: "string".to_string(),
        source: "CRM".to_string(),
        updated_at: Utc::now(),
    };
    
    let set_attribute = SetCustomAttribute {
        person_id,
        attribute: attribute.clone(),
    };
    
    let events = person.handle_command(PersonCommand::SetCustomAttribute(set_attribute)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert_eq!(person.custom_attributes.len(), 1);
    assert!(person.custom_attributes.contains_key("CustomerID"));
}

#[test]
fn test_person_lifecycle() {
    let person_id = PersonId::new();
    let name = PersonName::new("Active".to_string(), "Person".to_string());
    let mut person = Person::new(person_id, name, "Test".to_string());
    
    // Deactivate
    let deactivate = DeactivatePerson {
        person_id,
        reason: "Data cleanup".to_string(),
    };
    
    let events = person.handle_command(PersonCommand::DeactivatePerson(deactivate)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert!(!person.is_active);
    
    // Try to update inactive person
    let update_name = UpdateName {
        person_id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: None,
    };
    
    let result = person.handle_command(PersonCommand::UpdateName(update_name));
    assert!(result.is_err());
    
    // Reactivate
    let reactivate = ReactivatePerson {
        person_id,
        reason: "Customer returned".to_string(),
    };
    
    let events = person.handle_command(PersonCommand::ReactivatePerson(reactivate)).unwrap();
    for event in &events {
        person.apply_event(event);
    }
    
    assert!(person.is_active);
}

#[test]
fn test_person_merge() {
    let source_id = PersonId::new();
    let target_id = PersonId::new();
    let name = PersonName::new("Duplicate".to_string(), "Person".to_string());
    let mut source_person = Person::new(source_id, name, "Test".to_string());
    
    let merge = MergePersons {
        source_person_id: source_id,
        target_person_id: target_id,
        merge_reason: MergeReason::DuplicateIdentity,
    };
    
    let events = source_person.handle_command(PersonCommand::MergePersons(merge)).unwrap();
    assert_eq!(events.len(), 2); // PersonMergedInto and PersonsMerged
    
    for event in &events {
        source_person.apply_event(event);
    }
    
    assert_eq!(source_person.merged_into, Some(target_id));
    
    // Try to modify merged person
    let update_name = UpdateName {
        person_id: source_id,
        name: PersonName::new("New".to_string(), "Name".to_string()),
        reason: None,
    };
    
    let result = source_person.handle_command(PersonCommand::UpdateName(update_name));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Cannot modify a merged person");
} 