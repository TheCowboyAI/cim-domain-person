//! Person view service - Pure identity views
//!
//! Provides read-only views of person identity.
//! Cross-domain data (employment, skills, etc.) should be queried from their respective domains.

use crate::aggregate::Person;
use cim_domain::DomainResult;
use serde::{Deserialize, Serialize};

/// Basic person identity view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonIdentityView {
    /// The person's ID
    pub person_id: uuid::Uuid,

    /// Given name
    pub given_name: String,

    /// Family name
    pub family_name: String,

    /// Display name
    pub display_name: String,

    /// Birth date (if set)
    pub birth_date: Option<chrono::NaiveDate>,

    /// Date of death (if recorded)
    pub date_of_death: Option<chrono::NaiveDate>,

    /// Current lifecycle status
    pub status: LifecycleStatus,
}

/// Simplified lifecycle status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStatus {
    Active,
    Suspended,
    Deceased,
    Merged,
}

/// Service for creating person views
pub struct PersonViewService;

impl Default for PersonViewService {
    fn default() -> Self {
        Self::new()
    }
}

impl PersonViewService {
    /// Create a new view service
    pub fn new() -> Self {
        Self
    }

    /// Create an identity view from a person
    pub fn create_identity_view(person: &Person) -> DomainResult<PersonIdentityView> {
        let status = match &person.lifecycle {
            crate::aggregate::PersonLifecycle::Active => LifecycleStatus::Active,
            crate::aggregate::PersonLifecycle::Deactivated { .. } => LifecycleStatus::Suspended,
            crate::aggregate::PersonLifecycle::Deceased { .. } => LifecycleStatus::Deceased,
            crate::aggregate::PersonLifecycle::MergedInto { .. } => LifecycleStatus::Merged,
        };

        // Extract first given name and family name for simple view
        let given_name = person.core_identity.legal_name.components.given_names
            .first()
            .cloned()
            .unwrap_or_default();
        let family_name = person.core_identity.legal_name.components.family_names
            .first()
            .cloned()
            .unwrap_or_default();

        Ok(PersonIdentityView {
            person_id: person.id.into(),
            given_name,
            family_name,
            display_name: person.core_identity.legal_name.display_name(),
            birth_date: person.core_identity.birth_date,
            date_of_death: person.core_identity.death_date,
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::PersonName;
    use crate::aggregate::PersonId;

    #[test]
    fn test_create_identity_view() {
        let person_id = PersonId::new();
        let name = PersonName::new("John".to_string(), "Doe".to_string());
        let person = Person::new(person_id, name);

        let view = PersonViewService::create_identity_view(&person).unwrap();

        assert_eq!(view.given_name, "John");
        assert_eq!(view.family_name, "Doe");
        // display_name uses informal format (first given name only)
        assert_eq!(view.display_name, "John");
        assert_eq!(view.status, LifecycleStatus::Active);
    }
}
