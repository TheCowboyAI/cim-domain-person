//! Read model projections for the person domain

use crate::{
    aggregate::{Person, PersonId},
    events::PersonEvent,
    value_objects::*,
};
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};

/// Person projection for read models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonProjection {
    pub person_id: PersonId,
    pub name: String,
    pub emails: Vec<EmailAddress>,
    pub phones: Vec<PhoneNumber>,
    pub is_active: bool,
    pub employment: Option<EmploymentComponent>,
    pub position: Option<String>,
    pub department: Option<String>,
    pub skills: Vec<SkillProficiency>,
    pub access: Option<AccessComponent>,
    pub organization_id: Option<uuid::Uuid>,
    pub manager_id: Option<PersonId>,
    pub direct_reports: Vec<PersonId>,
    pub roles: Vec<String>,
}

impl PersonProjection {
    /// Apply an event to update the projection
    pub fn apply_event(&mut self, event: &PersonEvent) {
        match event {
            PersonEvent::PersonRegistered { person_id, identity, contact, .. } => {
                self.person_id = PersonId::from_uuid(*person_id);
                self.name = identity.preferred_name.as_ref()
                    .unwrap_or(&identity.legal_name)
                    .clone();
                if let Some(contact) = contact {
                    self.emails = contact.emails.clone();
                    self.phones = contact.phones.clone();
                }
                self.is_active = true;
            }

            PersonEvent::ContactRemoved { .. } => {
                self.emails.clear();
                self.phones.clear();
            }

            PersonEvent::ContactAdded { new_contact, .. } => {
                self.emails = new_contact.emails.clone();
                self.phones = new_contact.phones.clone();
            }

            PersonEvent::EmploymentAdded { employment, .. } => {
                self.employment = Some(employment.clone());
                self.is_active = employment.status == "active";
                self.organization_id = Some(employment.organization_id);
                self.position = Some(employment.title.clone());
                self.department = employment.department.clone();
            }

            PersonEvent::EmploymentStatusChanged { new_status, .. } => {
                if let Some(ref mut emp) = self.employment {
                    emp.status = new_status.clone();
                    self.is_active = new_status == "active";
                }
            }

            PersonEvent::PositionAdded { position, .. } => {
                self.position = Some(position.title.clone());
                // Department is on employment, not position
                // Manager relationship would need to be tracked separately
            }

            PersonEvent::SkillsRemoved { .. } => {
                self.skills.clear();
            }

            PersonEvent::SkillsAdded { new_skills, .. } => {
                self.skills = new_skills.skills.values().cloned().collect();
            }

            PersonEvent::AccessGranted { access, .. } => {
                self.access = Some(access.clone());
                self.roles = access.roles.clone();
            }

            _ => {}
        }
    }
}

/// Employee view of a person
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeView {
    /// The person's unique identifier
    pub person_id: PersonId,
    /// Employee name
    pub name: String,
    /// Primary email
    pub email: Option<String>,
    /// Department
    pub department: Option<String>,
    /// Position/title
    pub position: Option<String>,
    /// Manager ID
    pub manager_id: Option<PersonId>,
    /// Direct reports
    pub direct_reports: Vec<PersonId>,
    /// Active status
    pub is_active: bool,
}

impl EmployeeView {
    /// Create employee view from person
    pub fn from_person(person: &Person) -> DomainResult<Self> {
        let identity = person.get_component::<IdentityComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Person missing identity component".to_string()
            ))?;

        let contact = person.get_component::<ContactComponent>();
        let employment = person.get_component::<EmploymentComponent>();
        let position = person.get_component::<PositionComponent>();

        Ok(Self {
            person_id: person.id().into(),
            name: identity.preferred_name.as_ref()
                .unwrap_or(&identity.legal_name)
                .clone(),
            email: contact.and_then(|c| c.emails.first().map(|e| e.email.clone())),
            department: employment.and_then(|e| e.department.clone()),
            position: position.map(|p| p.title.clone())
                .or_else(|| employment.map(|e| e.title.clone())),
            manager_id: employment.and_then(|e| e.manager_id.map(PersonId::from_uuid)),
            direct_reports: Vec::new(), // Would need to be populated from a query
            is_active: employment.map(|e| e.status == "active").unwrap_or(false),
        })
    }
}

/// LDAP projection for directory services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapProjection {
    /// Distinguished Name (full LDAP path)
    pub dn: String,
    /// User ID
    pub uid: String,
    /// Common Name (typically the preferred name)
    pub cn: String,
    /// Surname (last name)
    pub sn: String,
    /// Given name (first name)
    pub given_name: String,
    /// Email addresses
    pub mail: Vec<String>,
    /// Phone numbers
    pub telephone_number: Vec<String>,
    /// Object classes
    pub object_class: Vec<String>,
}

impl LdapProjection {
    /// Create LDAP projection from PersonProjection
    pub fn from_projection(projection: &PersonProjection, base_dn: &str) -> Self {
        // Parse name (simple split for now)
        let name_parts: Vec<&str> = projection.name.split_whitespace().collect();
        let given_name = name_parts.first().unwrap_or(&"").to_string();
        let sn = name_parts.last().unwrap_or(&"").to_string();

        let cn = projection.name.clone();
        let dn = format!("cn={cn},ou=people,{base_dn}");

        let mail = projection.emails.iter()
            .map(|e| e.email.clone())
            .collect();

        let telephone_number = projection.phones.iter()
            .map(|p| p.number.clone())
            .collect();

        Self {
            dn,
            uid: projection.person_id.to_string(),
            cn,
            sn,
            given_name,
            mail,
            telephone_number,
            object_class: vec!["inetOrgPerson".to_string(), "person".to_string()],
        }
    }
    /// Create LDAP projection from person
    pub fn from_person(person: &Person, base_dn: &str) -> DomainResult<Self> {
        let identity = person.get_component::<IdentityComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Person missing identity component".to_string()
            ))?;

        let contact = person.get_component::<ContactComponent>();

        // Parse name (simple split for now)
        let name_parts: Vec<&str> = identity.legal_name.split_whitespace().collect();
        let given_name = name_parts.first().unwrap_or(&"").to_string();
        let sn = name_parts.last().unwrap_or(&"").to_string();

        let cn = identity.preferred_name.as_ref()
            .unwrap_or(&identity.legal_name)
            .clone();

        let dn = format!("cn={cn},ou=people,{base_dn}");

        let mail = contact.map(|c| c.emails.iter()
            .map(|e| e.email.clone())
            .collect())
            .unwrap_or_default();

        let telephone_number = contact.map(|c| c.phones.iter()
            .map(|p| p.number.clone())
            .collect())
            .unwrap_or_default();

        Ok(Self {
            dn,
            uid: person.id().to_string(),
            cn,
            sn,
            given_name,
            mail,
            telephone_number,
            object_class: vec!["inetOrgPerson".to_string(), "person".to_string()],
        })
    }
}
