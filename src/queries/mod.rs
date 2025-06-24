//! Person domain queries

use cim_domain::EntityId;
use serde::{Deserialize, Serialize};

use crate::aggregate::PersonMarker;

/// Person ID type alias
pub type PersonId = EntityId<PersonMarker>;

/// Queries for the Person domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonQuery {
    /// Get person by ID
    GetById {
        person_id: PersonId,
    },
    
    /// Find persons by name
    FindByName {
        given_name: Option<String>,
        family_name: Option<String>,
    },
    
    /// Find persons by email
    FindByEmail {
        email: String,
    },
    
    /// Find persons by phone
    FindByPhone {
        phone: String,
    },
    
    /// List all active persons
    ListActive {
        offset: usize,
        limit: usize,
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

    // CRM-specific queries

    /// Get customer view for a person
    GetCustomerView {
        person_id: PersonId
    },

    /// Get partner view for a person
    GetPartnerView {
        person_id: PersonId
    },

    /// Find customers by segment
    FindCustomersBySegment {
        segment: String,
        sub_segment: Option<String>,
    },

    /// Find customers by preferences
    FindCustomersByPreferences {
        preference_category: String,
        preference_value: String,
    },

    /// Find customers by behavioral pattern
    FindCustomersByBehavior {
        pattern: String,
        threshold: f32,
    },

    /// Find people by relationship
    FindPeopleByRelationship {
        relationship_type: String,
        related_person_id: Option<PersonId>,
    },

    /// Find people by interest
    FindPeopleByInterest {
        interest_category: String,
        interest_name: Option<String>,
    },

    /// Get people with birthdays in date range
    GetPeopleWithBirthdays {
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    },

    /// Find people by social media presence
    FindPeopleBySocialMedia {
        platform: String,
        has_verified: Option<bool>,
    },

    /// Get component history for a person
    GetComponentHistory {
        person_id: PersonId,
        component_type: Option<String>,
        since: Option<chrono::DateTime<chrono::Utc>>,
    },

    /// Find people with specific components
    FindPeopleWithComponents {
        component_types: Vec<String>,
        match_all: bool,
    },

    /// Get full person profile
    GetFullProfile {
        person_id: PersonId,
        include_history: bool,
    },

    /// Search people by multiple criteria
    SearchPeople {
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        organization: Option<String>,
        skills: Option<Vec<String>>,
        segments: Option<Vec<String>>,
        limit: usize,
        offset: usize,
    },
}
