//! Person view service for creating different views of person data
//!
//! This service provides methods for creating view-specific representations
//! of person entities based on their registered components.

use crate::aggregate::{Person, ComponentType};
use cim_domain::DomainResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// View representation of a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonView {
    /// The person's ID
    pub person_id: uuid::Uuid,
    
    /// The view type
    pub view_type: ViewType,
    
    /// Components registered for this person
    pub registered_components: Vec<String>,
    
    /// Basic person data
    pub data: PersonViewData,
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

/// Basic person view data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonViewData {
    pub given_name: String,
    pub family_name: String,
    pub display_name: String,
    pub is_active: bool,
    pub has_email: bool,
    pub has_phone: bool,
    pub has_address: bool,
    pub has_employment: bool,
    pub has_skills: bool,
    pub has_preferences: bool,
    pub has_relationships: bool,
}

/// Employee view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeView {
    pub person_id: uuid::Uuid,
    pub display_name: String,
    pub has_email: bool,
    pub has_employment: bool,
    pub has_skills: bool,
    pub is_active: bool,
}

impl EmployeeView {
    pub fn from_person(person: &Person) -> Self {
        Self {
            person_id: person.id.into(),
            display_name: person.core_identity.legal_name.display_name(),
            has_email: person.has_component(&ComponentType::EmailAddress),
            has_employment: person.has_component(&ComponentType::Employment),
            has_skills: person.has_component(&ComponentType::Skill),
            is_active: person.is_active(),
        }
    }
}

/// Customer view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerView {
    pub person_id: uuid::Uuid,
    pub display_name: String,
    pub has_email: bool,
    pub has_phone: bool,
    pub has_preferences: bool,
    pub has_segmentation: bool,
    pub has_behavioral_data: bool,
    pub is_active: bool,
}

impl CustomerView {
    pub fn from_person(person: &Person) -> Self {
        Self {
            person_id: person.id.into(),
            display_name: person.core_identity.legal_name.display_name(),
            has_email: person.has_component(&ComponentType::EmailAddress),
            has_phone: person.has_component(&ComponentType::PhoneNumber),
            has_preferences: person.has_component(&ComponentType::CommunicationPreferences),
            has_segmentation: person.has_component(&ComponentType::CustomerSegment),
            has_behavioral_data: person.has_component(&ComponentType::BehavioralData),
            is_active: person.is_active(),
        }
    }
}

/// Partner view for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerView {
    pub person_id: uuid::Uuid,
    pub display_name: String,
    pub has_email: bool,
    pub has_employment: bool,
    pub has_relationships: bool,
    pub has_social_profile: bool,
    pub is_active: bool,
}

impl PartnerView {
    pub fn from_person(person: &Person) -> Self {
        Self {
            person_id: person.id.into(),
            display_name: person.core_identity.legal_name.display_name(),
            has_email: person.has_component(&ComponentType::EmailAddress),
            has_employment: person.has_component(&ComponentType::Employment),
            has_relationships: person.has_component(&ComponentType::Relationship),
            has_social_profile: person.has_component(&ComponentType::SocialProfile),
            is_active: person.is_active(),
        }
    }
}

/// Builder for employee views
pub struct EmployeeViewBuilder;

impl EmployeeViewBuilder {
    /// Build an employee view from a person
    pub fn build(person: &Person) -> DomainResult<PersonView> {
        let registered_components: Vec<String> = person.components
            .iter()
            .map(|c| c.to_string())
            .collect();
        
        let data = PersonViewData {
            given_name: person.core_identity.legal_name.given_name.clone(),
            family_name: person.core_identity.legal_name.family_name.clone(),
            display_name: person.core_identity.legal_name.display_name(),
            is_active: person.is_active(),
            has_email: person.has_component(&ComponentType::EmailAddress),
            has_phone: person.has_component(&ComponentType::PhoneNumber),
            has_address: person.has_component(&ComponentType::Address),
            has_employment: person.has_component(&ComponentType::Employment),
            has_skills: person.has_component(&ComponentType::Skill),
            has_preferences: person.has_component(&ComponentType::GeneralPreferences),
            has_relationships: person.has_component(&ComponentType::Relationship),
        };
        
        Ok(PersonView {
            person_id: person.id.into(),
            view_type: ViewType::Employee,
            registered_components,
            data,
        })
    }
    
    /// Check if a person can be viewed as an employee
    pub fn can_build(person: &Person) -> bool {
        person.has_component(&ComponentType::Employment)
            && person.has_component(&ComponentType::EmailAddress)
    }
}

/// Builder for customer views
pub struct CustomerViewBuilder;

impl CustomerViewBuilder {
    /// Build a customer view from a person
    pub fn build(person: &Person) -> DomainResult<PersonView> {
        let registered_components: Vec<String> = person.components
            .iter()
            .map(|c| c.to_string())
            .collect();
        
        let data = PersonViewData {
            given_name: person.core_identity.legal_name.given_name.clone(),
            family_name: person.core_identity.legal_name.family_name.clone(),
            display_name: person.core_identity.legal_name.display_name(),
            is_active: person.is_active(),
            has_email: person.has_component(&ComponentType::EmailAddress),
            has_phone: person.has_component(&ComponentType::PhoneNumber),
            has_address: person.has_component(&ComponentType::Address),
            has_employment: person.has_component(&ComponentType::Employment),
            has_skills: person.has_component(&ComponentType::Skill),
            has_preferences: person.has_component(&ComponentType::CommunicationPreferences),
            has_relationships: person.has_component(&ComponentType::Relationship),
        };
        
        Ok(PersonView {
            person_id: person.id.into(),
            view_type: ViewType::Customer,
            registered_components,
            data,
        })
    }
    
    /// Check if a person can be viewed as a customer
    pub fn can_build(person: &Person) -> bool {
        person.has_component(&ComponentType::EmailAddress)
            && (person.has_component(&ComponentType::CommunicationPreferences)
                || person.has_component(&ComponentType::CustomerSegment))
    }
}

/// Builder for partner views
pub struct PartnerViewBuilder;

impl PartnerViewBuilder {
    /// Build a partner view from a person
    pub fn build(person: &Person) -> DomainResult<PersonView> {
        let registered_components: Vec<String> = person.components
            .iter()
            .map(|c| c.to_string())
            .collect();
        
        let data = PersonViewData {
            given_name: person.core_identity.legal_name.given_name.clone(),
            family_name: person.core_identity.legal_name.family_name.clone(),
            display_name: person.core_identity.legal_name.display_name(),
            is_active: person.is_active(),
            has_email: person.has_component(&ComponentType::EmailAddress),
            has_phone: person.has_component(&ComponentType::PhoneNumber),
            has_address: person.has_component(&ComponentType::Address),
            has_employment: person.has_component(&ComponentType::Employment),
            has_skills: person.has_component(&ComponentType::Skill),
            has_preferences: person.has_component(&ComponentType::GeneralPreferences),
            has_relationships: person.has_component(&ComponentType::Relationship),
        };
        
        Ok(PersonView {
            person_id: person.id.into(),
            view_type: ViewType::Partner,
            registered_components,
            data,
        })
    }
    
    /// Check if a person can be viewed as a partner
    pub fn can_build(person: &Person) -> bool {
        person.has_component(&ComponentType::EmailAddress)
            && person.has_component(&ComponentType::Relationship)
    }
}

/// Service for managing person views
pub struct PersonViewService;

impl PersonViewService {
    /// Create a new service instance
    pub fn new() -> Self {
        Self
    }
    
    /// Get all applicable views for a person
    pub fn get_applicable_views(&self, person: &Person) -> Vec<ViewType> {
        let mut views = Vec::new();
        
        if EmployeeViewBuilder::can_build(person) {
            views.push(ViewType::Employee);
        }
        
        if CustomerViewBuilder::can_build(person) {
            views.push(ViewType::Customer);
        }
        
        if PartnerViewBuilder::can_build(person) {
            views.push(ViewType::Partner);
        }
        
        // Everyone can be a contact
        views.push(ViewType::Contact);
        
        views
    }
    
    /// Build a specific view for a person
    pub fn build_view(&self, person: &Person, view_type: ViewType) -> DomainResult<PersonView> {
        match view_type {
            ViewType::Employee => EmployeeViewBuilder::build(person),
            ViewType::Customer => CustomerViewBuilder::build(person),
            ViewType::Partner => PartnerViewBuilder::build(person),
            _ => {
                // Default contact view
                Ok(PersonView {
                    person_id: person.id.into(),
                    view_type,
                    registered_components: person.components.iter().map(|c| c.to_string()).collect(),
                    data: PersonViewData {
                        given_name: person.core_identity.legal_name.given_name.clone(),
                        family_name: person.core_identity.legal_name.family_name.clone(),
                        display_name: person.core_identity.legal_name.display_name(),
                        is_active: person.is_active(),
                        has_email: person.has_component(&ComponentType::EmailAddress),
                        has_phone: person.has_component(&ComponentType::PhoneNumber),
                        has_address: person.has_component(&ComponentType::Address),
                        has_employment: false,
                        has_skills: false,
                        has_preferences: false,
                        has_relationships: false,
                    },
                })
            }
        }
    }
    
    /// Get a summary of registered components
    pub fn get_component_summary(&self, person: &Person) -> HashMap<String, bool> {
        let mut summary = HashMap::new();
        
        summary.insert("email".to_string(), person.has_component(&ComponentType::EmailAddress));
        summary.insert("phone".to_string(), person.has_component(&ComponentType::PhoneNumber));
        summary.insert("address".to_string(), person.has_component(&ComponentType::Address));
        summary.insert("employment".to_string(), person.has_component(&ComponentType::Employment));
        summary.insert("skills".to_string(), person.has_component(&ComponentType::Skill));
        summary.insert("certifications".to_string(), person.has_component(&ComponentType::Certification));
        summary.insert("education".to_string(), person.has_component(&ComponentType::Education));
        summary.insert("social_profiles".to_string(), person.has_component(&ComponentType::SocialProfile));
        summary.insert("relationships".to_string(), person.has_component(&ComponentType::Relationship));
        summary.insert("preferences".to_string(), person.has_component(&ComponentType::GeneralPreferences));
        summary.insert("customer_segment".to_string(), person.has_component(&ComponentType::CustomerSegment));
        summary.insert("behavioral_data".to_string(), person.has_component(&ComponentType::BehavioralData));
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::PersonId;
    use crate::value_objects::PersonName;

    #[test]
    fn test_employee_view() {
        // Create an employee
        let person_id = PersonId::new();
        let name = PersonName::new("John".to_string(), "Doe".to_string());
        let mut person = Person::new(person_id, name);
        
        // Register employee components
        person.register_component(ComponentType::Employment).unwrap();
        person.register_component(ComponentType::EmailAddress).unwrap();
        
        // Build employee view
        assert!(EmployeeViewBuilder::can_build(&person));
        let view = EmployeeViewBuilder::build(&person).unwrap();
        
        assert_eq!(view.view_type, ViewType::Employee);
        assert!(view.registered_components.contains(&"Employment".to_string()));
        assert!(view.registered_components.contains(&"EmailAddress".to_string()));
        assert!(view.data.has_employment);
        assert!(view.data.has_email);
    }

    #[test]
    fn test_customer_view() {
        // Create a customer
        let person_id = PersonId::new();
        let name = PersonName::new("Jane".to_string(), "Smith".to_string());
        let mut person = Person::new(person_id, name);
        
        // Register customer components
        person.register_component(ComponentType::EmailAddress).unwrap();
        person.register_component(ComponentType::CommunicationPreferences).unwrap();
        person.register_component(ComponentType::CustomerSegment).unwrap();
        
        // Build customer view
        assert!(CustomerViewBuilder::can_build(&person));
        let view = CustomerViewBuilder::build(&person).unwrap();
        
        assert_eq!(view.view_type, ViewType::Customer);
        assert!(view.data.has_email);
        assert!(view.data.has_preferences);
    }

    #[test]
    fn test_partner_view() {
        // Create a partner
        let person_id = PersonId::new();
        let name = PersonName::new("Partner".to_string(), "Company".to_string());
        let mut person = Person::new(person_id, name);
        
        // Register partner components
        person.register_component(ComponentType::EmailAddress).unwrap();
        person.register_component(ComponentType::Relationship).unwrap();
        
        // Build partner view
        assert!(PartnerViewBuilder::can_build(&person));
        let view = PartnerViewBuilder::build(&person).unwrap();
        
        assert_eq!(view.view_type, ViewType::Partner);
        assert!(view.data.has_email);
        assert!(view.data.has_relationships);
    }
    
    #[test]
    fn test_view_service() {
        let service = PersonViewService::new();
        
        // Create a person with multiple component types
        let person_id = PersonId::new();
        let name = PersonName::new("Multi".to_string(), "Role".to_string());
        let mut person = Person::new(person_id, name);
        
        // Register components for multiple views
        person.register_component(ComponentType::EmailAddress).unwrap();
        person.register_component(ComponentType::Employment).unwrap();
        person.register_component(ComponentType::CommunicationPreferences).unwrap();
        person.register_component(ComponentType::Relationship).unwrap();
        
        // Check applicable views
        let views = service.get_applicable_views(&person);
        assert!(views.contains(&ViewType::Employee));
        assert!(views.contains(&ViewType::Customer));
        assert!(views.contains(&ViewType::Partner));
        assert!(views.contains(&ViewType::Contact));
        
        // Get component summary
        let summary = service.get_component_summary(&person);
        assert_eq!(summary.get("email"), Some(&true));
        assert_eq!(summary.get("employment"), Some(&true));
        assert_eq!(summary.get("skills"), Some(&false));
    }
}