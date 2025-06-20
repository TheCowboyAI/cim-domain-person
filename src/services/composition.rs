//! Person composition service for building person entities with components
//!
//! This service provides methods for creating and composing person entities
//! with various components to represent different concepts.

use crate::aggregate::{Person, PersonId};
use crate::value_objects::*;
use cim_domain::DomainResult;
use uuid::Uuid;

/// Service for composing person entities with components
pub struct PersonCompositionService;

impl PersonCompositionService {
    /// Create a new service instance
    pub fn new() -> Self {
        Self
    }
    
    /// Create a basic person with minimal information from name string
    pub fn create_basic_person(
        &self,
        person_id: PersonId,
        name: &str,
    ) -> Person {
        let identity = IdentityComponent {
            legal_name: name.to_string(),
            preferred_name: None,
            date_of_birth: None,
            government_id: None,
        };
        
        Person::new(person_id, identity)
    }
    
    /// Create a basic person with name component
    pub fn create_basic_person_with_name(
        name: NameComponent,
    ) -> DomainResult<Person> {
        let person_id = PersonId::new();
        
        // Create legacy identity component for compatibility
        let identity = IdentityComponent {
            legal_name: name.display_name(),
            preferred_name: name.preferred_name.clone(),
            date_of_birth: None,
            government_id: None,
        };
        
        let mut person = Person::new(person_id, identity);
        
        // Add the comprehensive name component
        person.add_component(name, "system", Some("Initial creation".to_string()))?;
        
        Ok(person)
    }
    
    /// Create an employee with basic info
    pub fn create_employee(
        &self,
        person_id: PersonId,
        name: &str,
        department: &str,
        position: Option<&str>,
        manager_id: Option<PersonId>,
    ) -> Person {
        let identity = IdentityComponent {
            legal_name: name.to_string(),
            preferred_name: None,
            date_of_birth: None,
            government_id: None,
        };
        
        let mut person = Person::new(person_id, identity);
        
        // Add employment component
        let employment = EmploymentComponent {
            organization_id: Uuid::new_v4(),
            employee_id: format!("EMP{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
            title: position.unwrap_or("Employee").to_string(),
            department: Some(department.to_string()),
            manager_id: manager_id.map(|id| id.into()),
            status: "active".to_string(),
            start_date: chrono::Local::now().naive_local().date(),
            end_date: None,
        };
        
        person.add_component(employment, "hr", Some("Employee creation".to_string())).ok();
        
        person
    }
    
    /// Create an employee with standard components
    pub fn create_employee_with_components(
        name: NameComponent,
        employment: EmploymentComponent,
        contact: ContactComponent,
    ) -> DomainResult<Person> {
        let mut person = Self::create_basic_person_with_name(name)?;
        
        // Add employment components
        person.add_component(employment, "hr_system", Some("Employee onboarding".to_string()))?;
        person.add_component(contact, "hr_system", Some("Employee contact info".to_string()))?;
        
        // Add default access component
        let access = AccessComponent {
            roles: vec!["employee".to_string()],
            permissions: vec![],
            groups: vec![],
            access_level: Some("standard".to_string()),
        };
        person.add_component(access, "hr_system", Some("Default employee access".to_string()))?;
        
        Ok(person)
    }
    
    /// Create a customer with basic info
    pub fn create_customer(
        &self,
        person_id: PersonId,
        name: &str,
        email: Option<&str>,
        phone: Option<&str>,
    ) -> Person {
        let identity = IdentityComponent {
            legal_name: name.to_string(),
            preferred_name: None,
            date_of_birth: None,
            government_id: None,
        };
        
        let mut person = Person::new(person_id, identity);
        
        // Add contact if provided
        if email.is_some() || phone.is_some() {
            let mut emails = vec![];
            if let Some(email_str) = email {
                emails.push(EmailAddress {
                    email: email_str.to_string(),
                    email_type: "personal".to_string(),
                    is_primary: true,
                    is_verified: false,
                });
            }
            
            let mut phones = vec![];
            if let Some(phone_str) = phone {
                phones.push(PhoneNumber {
                    number: phone_str.to_string(),
                    phone_type: "personal".to_string(),
                    is_primary: true,
                    sms_capable: true,
                });
            }
            
            let contact = ContactComponent {
                emails,
                phones,
                addresses: vec![],
            };
            
            person.add_component(contact, "system", Some("Initial contact info".to_string())).ok();
        }
        
        person
    }
    
    /// Create a customer with CRM components
    pub fn create_customer_with_components(
        name: NameComponent,
        contact: ContactComponent,
        preferences: PreferencesComponent,
    ) -> DomainResult<Person> {
        let mut person = Self::create_basic_person_with_name(name)?;
        
        // Add customer components
        person.add_component(contact, "crm_system", Some("Customer contact info".to_string()))?;
        person.add_component(preferences, "crm_system", Some("Customer preferences".to_string()))?;
        
        // Add default segmentation
        let segmentation = SegmentationComponent {
            primary_segment: CustomerSegment::NewCustomer,
            secondary_segments: vec![],
            lifecycle_stage: LifecycleStage::Awareness,
            value_tier: ValueTier::Basic,
            persona: None,
            custom_segments: Default::default(),
        };
        person.add_component(segmentation, "crm_system", Some("Initial segmentation".to_string()))?;
        
        Ok(person)
    }
    
    /// Create a partner with basic info
    pub fn create_partner(
        &self,
        person_id: PersonId,
        name: &str,
        organization: &str,
        partnership_type: Option<&str>,
    ) -> Person {
        let identity = IdentityComponent {
            legal_name: name.to_string(),
            preferred_name: None,
            date_of_birth: None,
            government_id: None,
        };
        
        let mut person = Person::new(person_id, identity);
        
        // Add employment component for organization affiliation
        let employment = EmploymentComponent {
            organization_id: Uuid::new_v4(),
            employee_id: format!("PARTNER{}", Uuid::new_v4().to_string().split('-').next().unwrap()),
            title: partnership_type.unwrap_or("Partner").to_string(),
            department: Some(organization.to_string()),
            manager_id: None,
            status: "active".to_string(),
            start_date: chrono::Local::now().naive_local().date(),
            end_date: None,
        };
        
        person.add_component(employment, "partnerships", Some("Partner creation".to_string())).ok();
        
        person
    }
    
    /// Add physical attributes to a person
    pub fn add_physical_attributes(
        person: &mut Person,
        attributes: PhysicalAttributesComponent,
        added_by: &str,
        reason: Option<String>,
    ) -> DomainResult<()> {
        person.add_component(attributes, added_by, reason)
    }
    
    /// Add social media presence
    pub fn add_social_media(
        person: &mut Person,
        social: SocialMediaComponent,
        added_by: &str,
    ) -> DomainResult<()> {
        person.add_component(
            social,
            added_by,
            Some("Social media profile tracking".to_string()),
        )
    }
    
    /// Add behavioral data for CRM
    pub fn add_behavioral_data(
        person: &mut Person,
        behavioral: BehavioralComponent,
        added_by: &str,
    ) -> DomainResult<()> {
        person.add_component(
            behavioral,
            added_by,
            Some("Behavioral analytics data".to_string()),
        )
    }
    
    /// Add relationships
    pub fn add_relationships(
        person: &mut Person,
        relationships: RelationshipComponent,
        added_by: &str,
    ) -> DomainResult<()> {
        person.add_component(
            relationships,
            added_by,
            Some("Relationship mapping".to_string()),
        )
    }
    
    /// Check if a person has the required components for a specific view
    pub fn has_employee_components(person: &Person) -> bool {
        person.has_component::<EmploymentComponent>()
            && person.has_component::<ContactComponent>()
            && person.has_component::<AccessComponent>()
    }
    
    /// Check if a person has customer components
    pub fn has_customer_components(person: &Person) -> bool {
        person.has_component::<ContactComponent>()
            && person.has_component::<PreferencesComponent>()
            && person.has_component::<SegmentationComponent>()
    }
    
    /// Check if a person has partner components
    pub fn has_partner_components(person: &Person) -> bool {
        person.has_component::<ContactComponent>()
            && person.has_component::<RelationshipComponent>()
            && (person.has_component::<EmploymentComponent>()
                || person.has_component::<SkillsComponent>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_basic_person() {
        let name = NameComponent::simple("John".to_string(), "Doe".to_string());
        let person = PersonCompositionService::create_basic_person_with_name(name).unwrap();
        
        assert!(person.has_component::<NameComponent>());
        assert_eq!(person.component_count(), 2); // IdentityComponent + NameComponent
    }

    #[test]
    fn test_create_employee() {
        let name = NameComponent::simple("Jane".to_string(), "Smith".to_string());
        let employment = EmploymentComponent {
            organization_id: uuid::Uuid::new_v4(),
            employee_id: "EMP001".to_string(),
            title: "Software Engineer".to_string(),
            department: Some("Engineering".to_string()),
            manager_id: None,
            status: "active".to_string(),
            start_date: chrono::Local::now().naive_local().date(),
            end_date: None,
        };
        let contact = ContactComponent {
            emails: vec![EmailAddress {
                email: "jane.smith@company.com".to_string(),
                email_type: "work".to_string(),
                is_primary: true,
                is_verified: true,
            }],
            phones: vec![],
            addresses: vec![],
        };
        
        let person = PersonCompositionService::create_employee_with_components(
            name,
            employment,
            contact,
        ).unwrap();
        
        assert!(PersonCompositionService::has_employee_components(&person));
        assert!(person.has_component::<NameComponent>());
        assert!(person.has_component::<EmploymentComponent>());
        assert!(person.has_component::<ContactComponent>());
        assert!(person.has_component::<AccessComponent>());
    }

    #[test]
    fn test_create_customer() {
        let name = NameComponent::simple("Alice".to_string(), "Johnson".to_string());
        let contact = ContactComponent {
            emails: vec![EmailAddress {
                email: "alice@example.com".to_string(),
                email_type: "personal".to_string(),
                is_primary: true,
                is_verified: true,
            }],
            phones: vec![],
            addresses: vec![],
        };
        let preferences = PreferencesComponent {
            communication: CommunicationPreferences {
                preferred_channel: ContactChannel::Email,
                channel_settings: Default::default(),
                contact_time_preference: ContactTimePreference {
                    preferred_days: vec![],
                    preferred_hours: None,
                    timezone: "UTC".to_string(),
                },
                preferred_language: "en-US".to_string(),
                frequency_preference: FrequencyPreference::Weekly,
            },
            product_preferences: vec![],
            content_preferences: ContentPreferences {
                content_types: vec![ContentType::Educational],
                topics: vec!["Technology".to_string()],
                format_preference: ContentFormat::Text,
                complexity_preference: ComplexityLevel::Intermediate,
            },
            privacy_preferences: PrivacyPreferences {
                data_sharing_allowed: false,
                analytics_allowed: true,
                personalization_allowed: true,
                third_party_sharing_allowed: false,
                regulatory_preferences: Default::default(),
            },
        };
        
        let person = PersonCompositionService::create_customer_with_components(
            name,
            contact,
            preferences,
        ).unwrap();
        
        assert!(PersonCompositionService::has_customer_components(&person));
        assert!(person.has_component::<SegmentationComponent>());
    }

    #[test]
    fn test_add_components() {
        let name = NameComponent::simple("Bob".to_string(), "Wilson".to_string());
        let mut person = PersonCompositionService::create_basic_person_with_name(name).unwrap();
        
        // Add physical attributes
        let attributes = PhysicalAttributesComponent {
            height_cm: Some(180.0),
            weight_kg: Some(75.0),
            build: Some(Build::Athletic),
            hair_color: Some("Brown".to_string()),
            hair_style: None,
            eye_color: Some("Blue".to_string()),
            skin_tone: None,
            facial_hair: None,
            vision_correction: Some(VisionCorrection::None),
            appearance_notes: None,
        };
        
        PersonCompositionService::add_physical_attributes(
            &mut person,
            attributes,
            "medical_system",
            Some("Health screening".to_string()),
        ).unwrap();
        
        assert!(person.has_component::<PhysicalAttributesComponent>());
        
        // Add social media
        let social = SocialMediaComponent {
            profiles: vec![],
            metrics: None,
        };
        
        PersonCompositionService::add_social_media(
            &mut person,
            social,
            "marketing_system",
        ).unwrap();
        
        assert!(person.has_component::<SocialMediaComponent>());
    }
} 