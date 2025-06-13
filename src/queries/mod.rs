//! Query definitions for the person domain

use crate::aggregate::PersonId;
use serde::{Deserialize, Serialize};

/// Queries that can be executed against the person domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonQuery {
    /// Get a person by their ID
    GetPersonById {
        person_id: PersonId
    },

    /// Find a person by email address
    FindPersonByEmail {
        email: String
    },

    /// List all active people with pagination
    ListActivePeople {
        limit: usize,
        offset: usize
    },

    /// Search people by name pattern
    SearchPeopleByName {
        name_pattern: String
    },

    /// Get employee view for a person
    GetEmployeeView {
        person_id: PersonId
    },

    /// Get LDAP projection for a person
    GetLdapProjection {
        person_id: PersonId,
        base_dn: String,
    },

    /// Find people by organization
    FindPeopleByOrganization {
        organization_id: uuid::Uuid,
        include_inactive: bool,
    },

    /// Find people by skill
    FindPeopleBySkill {
        skill_name: String,
        min_proficiency: Option<String>,
    },

    /// Find people by role
    FindPeopleByRole {
        role: String,
    },
}
