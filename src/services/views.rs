//! View builders for creating different person concepts
//!
//! These builders demonstrate how to compose person entities with
//! specific sets of components to represent different business concepts.

use crate::aggregate::Person;
use crate::value_objects::*;
use cim_domain::DomainResult;
use serde::{Deserialize, Serialize};

/// A view of a person with specific components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonView {
    /// The person's ID
    pub person_id: uuid::Uuid,
    
    /// The view type
    pub view_type: ViewType,
    
    /// Components included in this view
    pub components: Vec<String>,
    
    /// View-specific data
    pub data: serde_json::Value,
}

/// Types of person views
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewType {
    Employee,
    Customer,
    Partner,
    Candidate,
    Contact,
    Custom(String),
}

/// Employee view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeView {
    pub name: String,
    pub email: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub manager_id: Option<uuid::Uuid>,
    pub skills: Vec<String>,
}

impl EmployeeView {
    pub fn from_person(person: &Person) -> Self {
        let identity = person.get_component::<IdentityComponent>();
        let name = person.get_component::<NameComponent>()
            .map(|n| n.display_name())
            .or_else(|| identity.map(|i| i.legal_name.clone()))
            .unwrap_or_else(|| "Unknown".to_string());
        
        let contact = person.get_component::<ContactComponent>();
        let email = contact.and_then(|c| c.emails.first().map(|e| e.email.clone()));
        
        let employment = person.get_component::<EmploymentComponent>();
        let department = employment.and_then(|e| e.department.clone());
        let position = person.get_component::<PositionComponent>()
            .map(|p| p.title.clone())
            .or_else(|| employment.map(|e| e.title.clone()));
        let manager_id = employment.and_then(|e| e.manager_id);
        
        let skills = person.get_component::<SkillsComponent>()
            .map(|s| s.skills.keys().cloned().collect())
            .unwrap_or_default();
        
        Self {
            name,
            email,
            department,
            position,
            manager_id,
            skills,
        }
    }
}

/// Customer view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerView {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub segment: Option<String>,
    pub lifetime_value: Option<f64>,
    pub engagement_score: Option<f32>,
    pub preferred_channel: Option<String>,
    pub language_preference: Option<String>,
}

impl CustomerView {
    pub fn from_person(person: &Person) -> Self {
        let identity = person.get_component::<IdentityComponent>();
        let name = person.get_component::<NameComponent>()
            .map(|n| n.display_name())
            .or_else(|| identity.map(|i| i.legal_name.clone()))
            .unwrap_or_else(|| "Unknown".to_string());
        
        let contact = person.get_component::<ContactComponent>();
        let email = contact.and_then(|c| c.emails.first().map(|e| e.email.clone()));
        let phone = contact.and_then(|c| c.phones.first().map(|p| p.number.clone()));
        
        let segmentation = person.get_component::<SegmentationComponent>();
        let segment = segmentation.map(|s| format!("{:?}", s.primary_segment));
        
        let behavioral = person.get_component::<BehavioralComponent>();
        let lifetime_value = behavioral.and_then(|b| b.predictive_scores.predicted_ltv);
        let engagement_score = behavioral.and_then(|b| b.engagement_patterns.email_open_rate);
        
        let preferences = person.get_component::<PreferencesComponent>();
        let preferred_channel = preferences.map(|p| format!("{:?}", p.communication.preferred_channel));
        let language_preference = preferences.map(|p| p.communication.preferred_language.clone());
        
        Self {
            name,
            email,
            phone,
            segment,
            lifetime_value,
            engagement_score,
            preferred_channel,
            language_preference,
        }
    }
}

/// Partner view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerView {
    pub name: String,
    pub email: Option<String>,
    pub organization: Option<String>,
    pub partnership_type: Option<String>,
    pub social_profiles: Vec<String>,
    pub influence_score: Option<f32>,
}

impl PartnerView {
    pub fn from_person(person: &Person) -> Self {
        let identity = person.get_component::<IdentityComponent>();
        let name = person.get_component::<NameComponent>()
            .map(|n| n.display_name())
            .or_else(|| identity.map(|i| i.legal_name.clone()))
            .unwrap_or_else(|| "Unknown".to_string());
        
        let contact = person.get_component::<ContactComponent>();
        let email = contact.and_then(|c| c.emails.first().map(|e| e.email.clone()));
        
        let employment = person.get_component::<EmploymentComponent>();
        let organization = employment.and_then(|e| e.department.clone());
        let partnership_type = employment.map(|e| e.title.clone());
        
        let social = person.get_component::<SocialMediaComponent>();
        let social_profiles = social
            .map(|s| s.profiles.iter().map(|p| format!("{:?}: {}", p.platform, p.username)).collect())
            .unwrap_or_default();
        let influence_score = social.and_then(|s| s.metrics.as_ref().and_then(|m| m.influence_score));
        
        Self {
            name,
            email,
            organization,
            partnership_type,
            social_profiles,
            influence_score,
        }
    }
}

/// Builder for employee views
pub struct EmployeeViewBuilder;

impl EmployeeViewBuilder {
    /// Build an employee view from a person
    pub fn build(person: &Person) -> DomainResult<PersonView> {
        let mut data = serde_json::Map::new();
        let mut components = Vec::new();
        
        // Extract name
        if let Some(name) = person.get_component::<NameComponent>() {
            data.insert("name".to_string(), serde_json::to_value(name)?);
            components.push("NameComponent".to_string());
        }
        
        // Extract employment
        if let Some(employment) = person.get_component::<EmploymentComponent>() {
            data.insert("employment".to_string(), serde_json::to_value(employment)?);
            components.push("EmploymentComponent".to_string());
        }
        
        // Extract contact
        if let Some(contact) = person.get_component::<ContactComponent>() {
            data.insert("contact".to_string(), serde_json::to_value(contact)?);
            components.push("ContactComponent".to_string());
        }
        
        // Extract skills
        if let Some(skills) = person.get_component::<SkillsComponent>() {
            data.insert("skills".to_string(), serde_json::to_value(skills)?);
            components.push("SkillsComponent".to_string());
        }
        
        // Extract access
        if let Some(access) = person.get_component::<AccessComponent>() {
            data.insert("access".to_string(), serde_json::to_value(access)?);
            components.push("AccessComponent".to_string());
        }
        
        // Extract position
        if let Some(position) = person.get_component::<PositionComponent>() {
            data.insert("position".to_string(), serde_json::to_value(position)?);
            components.push("PositionComponent".to_string());
        }
        
        Ok(PersonView {
            person_id: person.id().into(),
            view_type: ViewType::Employee,
            components,
            data: serde_json::Value::Object(data),
        })
    }
    
    /// Check if a person can be viewed as an employee
    pub fn can_build(person: &Person) -> bool {
        person.has_component::<EmploymentComponent>()
            && person.has_component::<ContactComponent>()
    }
}

/// Builder for customer views
pub struct CustomerViewBuilder;

impl CustomerViewBuilder {
    /// Build a customer view from a person
    pub fn build(person: &Person) -> DomainResult<PersonView> {
        let mut data = serde_json::Map::new();
        let mut components = Vec::new();
        
        // Extract name
        if let Some(name) = person.get_component::<NameComponent>() {
            data.insert("name".to_string(), serde_json::to_value(name)?);
            components.push("NameComponent".to_string());
        }
        
        // Extract contact
        if let Some(contact) = person.get_component::<ContactComponent>() {
            data.insert("contact".to_string(), serde_json::to_value(contact)?);
            components.push("ContactComponent".to_string());
        }
        
        // Extract preferences
        if let Some(preferences) = person.get_component::<PreferencesComponent>() {
            data.insert("preferences".to_string(), serde_json::to_value(preferences)?);
            components.push("PreferencesComponent".to_string());
        }
        
        // Extract behavioral data
        if let Some(behavioral) = person.get_component::<BehavioralComponent>() {
            data.insert("behavioral".to_string(), serde_json::to_value(behavioral)?);
            components.push("BehavioralComponent".to_string());
        }
        
        // Extract segmentation
        if let Some(segmentation) = person.get_component::<SegmentationComponent>() {
            data.insert("segmentation".to_string(), serde_json::to_value(segmentation)?);
            components.push("SegmentationComponent".to_string());
        }
        
        // Extract social media
        if let Some(social) = person.get_component::<SocialMediaComponent>() {
            data.insert("social_media".to_string(), serde_json::to_value(social)?);
            components.push("SocialMediaComponent".to_string());
        }
        
        // Extract interests
        if let Some(interests) = person.get_component::<InterestsComponent>() {
            data.insert("interests".to_string(), serde_json::to_value(interests)?);
            components.push("InterestsComponent".to_string());
        }
        
        Ok(PersonView {
            person_id: person.id().into(),
            view_type: ViewType::Customer,
            components,
            data: serde_json::Value::Object(data),
        })
    }
    
    /// Check if a person can be viewed as a customer
    pub fn can_build(person: &Person) -> bool {
        person.has_component::<ContactComponent>()
            && (person.has_component::<PreferencesComponent>()
                || person.has_component::<SegmentationComponent>())
    }
}

/// Builder for partner views
pub struct PartnerViewBuilder;

impl PartnerViewBuilder {
    /// Build a partner view from a person
    pub fn build(person: &Person) -> DomainResult<PersonView> {
        let mut data = serde_json::Map::new();
        let mut components = Vec::new();
        
        // Extract name
        if let Some(name) = person.get_component::<NameComponent>() {
            data.insert("name".to_string(), serde_json::to_value(name)?);
            components.push("NameComponent".to_string());
        }
        
        // Extract contact
        if let Some(contact) = person.get_component::<ContactComponent>() {
            data.insert("contact".to_string(), serde_json::to_value(contact)?);
            components.push("ContactComponent".to_string());
        }
        
        // Extract relationships
        if let Some(relationships) = person.get_component::<RelationshipComponent>() {
            data.insert("relationships".to_string(), serde_json::to_value(relationships)?);
            components.push("RelationshipComponent".to_string());
        }
        
        // Extract employment (for business partners)
        if let Some(employment) = person.get_component::<EmploymentComponent>() {
            data.insert("employment".to_string(), serde_json::to_value(employment)?);
            components.push("EmploymentComponent".to_string());
        }
        
        // Extract skills (for technical partners)
        if let Some(skills) = person.get_component::<SkillsComponent>() {
            data.insert("skills".to_string(), serde_json::to_value(skills)?);
            components.push("SkillsComponent".to_string());
        }
        
        // Extract social media (for influencer partners)
        if let Some(social) = person.get_component::<SocialMediaComponent>() {
            data.insert("social_media".to_string(), serde_json::to_value(social)?);
            components.push("SocialMediaComponent".to_string());
        }
        
        Ok(PersonView {
            person_id: person.id().into(),
            view_type: ViewType::Partner,
            components,
            data: serde_json::Value::Object(data),
        })
    }
    
    /// Check if a person can be viewed as a partner
    pub fn can_build(person: &Person) -> bool {
        person.has_component::<ContactComponent>()
            && person.has_component::<RelationshipComponent>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::PersonCompositionService;

    #[test]
    fn test_employee_view() {
        // Create an employee
        let name = NameComponent {
            title: None,
            honorific: None,
            given_names: vec!["John".to_string()],
            middle_names: vec![],
            family_names: vec!["Doe".to_string()],
            maternal_family_name: None,
            generational_suffix: None,
            professional_suffix: None,
            preferred_name: None,
            name_order: NameOrder::GivenFirst,
            cultural_context: None,
        };
        let employment = EmploymentComponent {
            organization_id: uuid::Uuid::new_v4(),
            employee_id: "EMP001".to_string(),
            title: "Manager".to_string(),
            department: Some("Sales".to_string()),
            manager_id: None,
            status: "active".to_string(),
            start_date: chrono::Local::now().naive_local().date(),
            end_date: None,
        };
        let contact = ContactComponent {
            emails: vec![],
            phones: vec![],
            addresses: vec![],
        };
        
        let person = PersonCompositionService::create_employee_with_components(
            name,
            employment,
            contact,
        ).unwrap();
        
        // Build employee view
        assert!(EmployeeViewBuilder::can_build(&person));
        let view = EmployeeViewBuilder::build(&person).unwrap();
        
        assert_eq!(view.view_type, ViewType::Employee);
        assert!(view.components.contains(&"EmploymentComponent".to_string()));
        assert!(view.components.contains(&"ContactComponent".to_string()));
    }

    #[test]
    fn test_customer_view() {
        // Create a customer
        let name = NameComponent {
            title: None,
            honorific: None,
            given_names: vec!["Jane".to_string()],
            middle_names: vec![],
            family_names: vec!["Smith".to_string()],
            maternal_family_name: None,
            generational_suffix: None,
            professional_suffix: None,
            preferred_name: None,
            name_order: NameOrder::GivenFirst,
            cultural_context: None,
        };
        let contact = ContactComponent {
            emails: vec![],
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
                frequency_preference: FrequencyPreference::Monthly,
            },
            product_preferences: vec![],
            content_preferences: ContentPreferences {
                content_types: vec![],
                topics: vec![],
                format_preference: ContentFormat::Mixed,
                complexity_preference: ComplexityLevel::Beginner,
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
        
        // Build customer view
        assert!(CustomerViewBuilder::can_build(&person));
        let view = CustomerViewBuilder::build(&person).unwrap();
        
        assert_eq!(view.view_type, ViewType::Customer);
        assert!(view.components.contains(&"PreferencesComponent".to_string()));
        assert!(view.components.contains(&"ContactComponent".to_string()));
    }

    #[test]
    fn test_partner_view() {
        // Create a basic person and add partner components
        let name = NameComponent {
            title: None,
            honorific: None,
            given_names: vec!["Partner".to_string()],
            middle_names: vec![],
            family_names: vec!["Company".to_string()],
            maternal_family_name: None,
            generational_suffix: None,
            professional_suffix: None,
            preferred_name: None,
            name_order: NameOrder::GivenFirst,
            cultural_context: None,
        };
        let mut person = PersonCompositionService::create_basic_person_with_name(name).unwrap();
        
        // Add contact
        let contact = ContactComponent {
            emails: vec![],
            phones: vec![],
            addresses: vec![],
        };
        person.add_component(contact, "system", None).unwrap();
        
        // Add relationships
        let relationships = RelationshipComponent {
            relationships: vec![Relationship {
                person_id: uuid::Uuid::new_v4(),
                relationship_type: RelationshipType::BusinessPartner,
                reciprocal_type: RelationshipType::BusinessPartner,
                start_date: Some(chrono::Local::now().naive_local().date()),
                end_date: None,
                status: RelationshipStatus::Active,
                notes: Some("Strategic partner".to_string()),
            }],
        };
        person.add_component(relationships, "system", None).unwrap();
        
        // Build partner view
        assert!(PartnerViewBuilder::can_build(&person));
        let view = PartnerViewBuilder::build(&person).unwrap();
        
        assert_eq!(view.view_type, ViewType::Partner);
        assert!(view.components.contains(&"RelationshipComponent".to_string()));
    }
} 