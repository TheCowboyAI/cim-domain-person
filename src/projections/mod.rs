//! Read model projections for the person domain

use crate::{
    aggregate::{Person, PersonId},
    events::PersonEvent,
    value_objects::*,
};
use cim_domain::{DomainResult, DomainError, AggregateRoot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Person projection for read models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonProjection {
    pub person_id: PersonId,
    pub name: String,
    pub emails: Vec<EmailAddress>,
    pub phones: Vec<PhoneNumber>,
    pub is_active: bool,
    pub employment: Option<EmploymentComponent>,
    pub position: Option<PositionComponent>,
    pub skills: Option<SkillsComponent>,
    pub access: Option<AccessComponent>,
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

            PersonEvent::ContactUpdated { new_contact, .. } => {
                self.emails = new_contact.emails.clone();
                self.phones = new_contact.phones.clone();
            }

            PersonEvent::EmploymentAdded { employment, .. } => {
                self.employment = Some(employment.clone());
                self.is_active = employment.status == "active";
            }

            PersonEvent::EmploymentStatusChanged { new_status, .. } => {
                if let Some(ref mut emp) = self.employment {
                    emp.status = new_status.clone();
                    self.is_active = new_status == "active";
                }
            }

            PersonEvent::PositionAdded { position, .. } => {
                self.position = Some(position.clone());
            }

            PersonEvent::SkillsUpdated { new_skills, .. } => {
                self.skills = Some(new_skills.clone());
            }

            PersonEvent::AccessGranted { access, .. } => {
                self.access = Some(access.clone());
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
    /// Identity information (name, DOB, etc.)
    pub identity: IdentityComponent,
    /// Contact information (email, phone, address)
    pub contact: ContactComponent,
    /// Employment details (organization, title, department)
    pub employment: EmploymentComponent,
    /// Current position information if available
    pub position: Option<PositionComponent>,
    /// Skills and certifications if available
    pub skills: Option<SkillsComponent>,
}

impl EmployeeView {
    /// Create employee view from person
    pub fn from_person(person: &Person) -> DomainResult<Self> {
        let identity = person.get_component::<IdentityComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Person missing identity component".to_string()
            ))?
            .clone();

        let contact = person.get_component::<ContactComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Employee missing contact component".to_string()
            ))?
            .clone();

        let employment = person.get_component::<EmploymentComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Employee missing employment component".to_string()
            ))?
            .clone();

        let position = person.get_component::<PositionComponent>().cloned();
        let skills = person.get_component::<SkillsComponent>().cloned();

        Ok(Self {
            person_id: person.id(),
            identity,
            contact,
            employment,
            position,
            skills,
        })
    }
}

/// LDAP projection for directory services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapProjection {
    /// Distinguished Name (full LDAP path)
    pub dn: String,
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
    /// Job title if employed
    pub title: Option<String>,
    /// Department if employed
    pub department: Option<String>,
    /// Manager's DN if applicable
    pub manager: Option<String>,
}

impl LdapProjection {
    /// Create LDAP projection from person
    pub fn from_person(person: &Person, base_dn: &str) -> DomainResult<Self> {
        let identity = person.get_component::<IdentityComponent>()
            .ok_or_else(|| DomainError::ValidationError(
                "Person missing identity component".to_string()
            ))?;

        let contact = person.get_component::<ContactComponent>();
        let employment = person.get_component::<EmploymentComponent>();

        // Parse name (simple split for now)
        let name_parts: Vec<&str> = identity.legal_name.split_whitespace().collect();
        let given_name = name_parts.first().unwrap_or(&"").to_string();
        let sn = name_parts.last().unwrap_or(&"").to_string();

        let cn = identity.preferred_name.as_ref()
            .unwrap_or(&identity.legal_name)
            .clone();

        let dn = format!("cn={},ou=people,{}", cn, base_dn);

        let mail = contact.map(|c| c.emails.iter()
            .map(|e| e.email.clone())
            .collect())
            .unwrap_or_default();

        let telephone_number = contact.map(|c| c.phones.iter()
            .map(|p| p.number.clone())
            .collect())
            .unwrap_or_default();

        let (title, department) = employment.map(|e| (Some(e.title.clone()), e.department.clone()))
            .unwrap_or((None, None));

        Ok(Self {
            dn,
            cn,
            sn,
            given_name,
            mail,
            telephone_number,
            title,
            department,
            manager: None, // Would need to resolve manager's DN
        })
    }
}
