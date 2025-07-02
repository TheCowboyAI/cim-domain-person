//! Person composition service for building person entities with components
//!
//! This service provides methods for creating and composing person entities
//! with various components to represent different concepts.

use crate::aggregate::{Person, PersonId, ComponentType};
use crate::value_objects::*;
use cim_domain::DomainResult;

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
        let person_name = PersonName::new(
            name.split_whitespace().next().unwrap_or("").to_string(),
            name.split_whitespace().last().unwrap_or("").to_string(),
        );
        
        Person::new(person_id, person_name)
    }
    
    /// Create a basic person with name
    pub fn create_basic_person_with_name(
        person_id: PersonId,
        name: PersonName,
    ) -> DomainResult<Person> {
        Ok(Person::new(person_id, name))
    }
    
    /// Create an employee with basic info
    pub fn create_employee(
        &self,
        person_id: PersonId,
        name: &str,
        _department: &str,
        _position: Option<&str>,
        _manager_id: Option<PersonId>,
    ) -> DomainResult<Person> {
        let person_name = PersonName::new(
            name.split_whitespace().next().unwrap_or("").to_string(),
            name.split_whitespace().last().unwrap_or("").to_string(),
        );
        
        let mut person = Person::new(person_id, person_name);
        
        // Register that this person has employment component
        person.register_component(ComponentType::Employment)?;
        // Also register email as expected by has_employee_components
        person.register_component(ComponentType::EmailAddress)?;
        
        Ok(person)
    }
    
    /// Create an employee with standard components
    pub fn create_employee_with_components(
        person_id: PersonId,
        name: PersonName,
    ) -> DomainResult<Person> {
        let mut person = Person::new(person_id, name);
        
        // Register employment-related components
        person.register_component(ComponentType::Employment)?;
        person.register_component(ComponentType::EmailAddress)?;
        person.register_component(ComponentType::PhoneNumber)?;
        
        Ok(person)
    }
    
    /// Create a customer with basic info
    pub fn create_customer(
        &self,
        person_id: PersonId,
        name: &str,
        email: Option<&str>,
        phone: Option<&str>,
    ) -> DomainResult<Person> {
        let person_name = PersonName::new(
            name.split_whitespace().next().unwrap_or("").to_string(),
            name.split_whitespace().last().unwrap_or("").to_string(),
        );
        
        let mut person = Person::new(person_id, person_name);
        
        // Register contact components if provided
        if email.is_some() {
            person.register_component(ComponentType::EmailAddress)?;
        }
        
        if phone.is_some() {
            person.register_component(ComponentType::PhoneNumber)?;
        }
        
        Ok(person)
    }
    
    /// Create a customer with CRM components
    pub fn create_customer_with_components(
        person_id: PersonId,
        name: PersonName,
    ) -> DomainResult<Person> {
        let mut person = Person::new(person_id, name);
        
        // Register customer-related components
        person.register_component(ComponentType::EmailAddress)?;
        person.register_component(ComponentType::PhoneNumber)?;
        person.register_component(ComponentType::CommunicationPreferences)?;
        person.register_component(ComponentType::CustomerSegment)?;
        
        Ok(person)
    }
    
    /// Create a partner with basic info
    pub fn create_partner(
        &self,
        person_id: PersonId,
        name: &str,
        _organization: &str,
        _partnership_type: Option<&str>,
    ) -> DomainResult<Person> {
        let person_name = PersonName::new(
            name.split_whitespace().next().unwrap_or("").to_string(),
            name.split_whitespace().last().unwrap_or("").to_string(),
        );
        
        let mut person = Person::new(person_id, person_name);
        
        // Register partner-related components
        person.register_component(ComponentType::Employment)?;
        person.register_component(ComponentType::Relationship)?;
        
        Ok(person)
    }
    
    /// Register physical attributes component
    pub fn register_physical_attributes(
        person: &mut Person,
    ) -> DomainResult<()> {
        person.register_component(ComponentType::CustomAttribute)
    }
    
    /// Register social media component
    pub fn register_social_media(
        person: &mut Person,
    ) -> DomainResult<()> {
        person.register_component(ComponentType::SocialProfile)
    }
    
    /// Register behavioral data component
    pub fn register_behavioral_data(
        person: &mut Person,
    ) -> DomainResult<()> {
        person.register_component(ComponentType::BehavioralData)
    }
    
    /// Register relationships component
    pub fn register_relationships(
        person: &mut Person,
    ) -> DomainResult<()> {
        person.register_component(ComponentType::Relationship)
    }
    
    /// Check if a person has the required components for a specific view
    pub fn has_employee_components(person: &Person) -> bool {
        person.has_component(&ComponentType::Employment)
            && person.has_component(&ComponentType::EmailAddress)
    }
    
    /// Check if a person has customer components
    pub fn has_customer_components(person: &Person) -> bool {
        person.has_component(&ComponentType::EmailAddress)
            && person.has_component(&ComponentType::CommunicationPreferences)
            && person.has_component(&ComponentType::CustomerSegment)
    }
    
    /// Check if a person has partner components
    pub fn has_partner_components(person: &Person) -> bool {
        person.has_component(&ComponentType::EmailAddress)
            && person.has_component(&ComponentType::Relationship)
            && person.has_component(&ComponentType::Employment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_basic_person() {
        let person_id = PersonId::new();
        let name = PersonName::new("John".to_string(), "Doe".to_string());
        let person = PersonCompositionService::create_basic_person_with_name(person_id, name).unwrap();
        
        assert_eq!(person.id, person_id);
        assert_eq!(person.core_identity.legal_name.given_name, "John");
        assert_eq!(person.core_identity.legal_name.family_name, "Doe");
    }

    #[test]
    fn test_create_employee() {
        let service = PersonCompositionService::new();
        let person_id = PersonId::new();
        
        let person = service.create_employee(
            person_id,
            "Jane Smith",
            "Engineering",
            Some("Software Engineer"),
            None,
        ).unwrap();
        
        assert!(PersonCompositionService::has_employee_components(&person));
        assert!(person.has_component(&ComponentType::Employment));
    }

    #[test]
    fn test_create_customer() {
        let service = PersonCompositionService::new();
        let person_id = PersonId::new();
        
        let person = service.create_customer(
            person_id,
            "Alice Johnson",
            Some("alice@example.com"),
            Some("555-1234"),
        ).unwrap();
        
        assert!(person.has_component(&ComponentType::EmailAddress));
        assert!(person.has_component(&ComponentType::PhoneNumber));
    }

    #[test]
    fn test_register_components() {
        let person_id = PersonId::new();
        let name = PersonName::new("Bob".to_string(), "Wilson".to_string());
        let mut person = Person::new(person_id, name);
        
        // Register social media
        PersonCompositionService::register_social_media(&mut person).unwrap();
        assert!(person.has_component(&ComponentType::SocialProfile));
        
        // Register behavioral data
        PersonCompositionService::register_behavioral_data(&mut person).unwrap();
        assert!(person.has_component(&ComponentType::BehavioralData));
    }
} 