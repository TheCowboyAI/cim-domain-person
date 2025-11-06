//! Person composition service for creating person entities
//!
//! This service provides methods for creating person entities with core identity.
//!
//! Cross-domain concerns belong in their respective domains:
//! - Employee → cim-domain-hr or cim-domain-employment
//! - Customer → cim-domain-crm
//! - Partner → cim-domain-partners
//! - Contact info → cim-domain-contacts

use crate::aggregate::{Person, PersonId};
use crate::value_objects::*;
use cim_domain::DomainResult;

/// Service for creating person entities
pub struct PersonCompositionService;

impl Default for PersonCompositionService {
    fn default() -> Self {
        Self::new()
    }
}

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
        assert_eq!(person.core_identity.legal_name.components.given_names[0], "John");
        assert_eq!(person.core_identity.legal_name.components.family_names[0], "Doe");
    }

    #[test]
    fn test_create_basic_person_from_string() {
        let service = PersonCompositionService::new();
        let person_id = PersonId::new();

        let person = service.create_basic_person(
            person_id,
            "Jane Smith",
        );

        assert_eq!(person.id, person_id);
        assert_eq!(person.core_identity.legal_name.components.given_names[0], "Jane");
        assert_eq!(person.core_identity.legal_name.components.family_names[0], "Smith");
    }
}
