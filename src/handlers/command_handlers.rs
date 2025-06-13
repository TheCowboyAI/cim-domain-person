//! Command handlers for person domain

use crate::{
    aggregate::{Person, PersonId, PersonMarker},
    commands::PersonCommand,
    events::PersonEvent,
    value_objects::*,
};
use cim_core_domain::{AggregateRoot, EntityId, DomainResult};
use chrono::Utc;

/// Handle person commands
pub async fn handle_person_command(
    aggregate: &mut Person,
    command: PersonCommand,
) -> DomainResult<Vec<PersonEvent>> {
    match command {
        PersonCommand::RegisterPerson { person_id, identity, contact } => {
            // This would typically be handled at aggregate creation
            Ok(vec![PersonEvent::PersonRegistered {
                person_id,
                identity,
                contact,
                registered_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateContact { person_id, contact } => {
            let old_contact = aggregate.get_component::<ContactComponent>().cloned();
            aggregate.remove_component::<ContactComponent>().ok();
            aggregate.add_component(contact.clone(), "system", Some("Contact update".to_string()))?;

            Ok(vec![PersonEvent::ContactUpdated {
                person_id,
                old_contact,
                new_contact: contact,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::AddEmployment { person_id, employment } => {
            aggregate.add_component(employment.clone(), "system", Some("Employment added".to_string()))?;

            Ok(vec![PersonEvent::EmploymentAdded {
                person_id,
                employment,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateEmploymentStatus { person_id, organization_id, status, end_date } => {
            if let Some(mut employment) = aggregate.get_component::<EmploymentComponent>().cloned() {
                let old_status = employment.status.clone();
                employment.status = status.clone();
                employment.end_date = end_date;

                aggregate.remove_component::<EmploymentComponent>()?;
                aggregate.add_component(employment, "system", Some("Status update".to_string()))?;

                Ok(vec![PersonEvent::EmploymentStatusChanged {
                    person_id,
                    organization_id,
                    old_status,
                    new_status: status,
                    end_date,
                    changed_at: Utc::now(),
                }])
            } else {
                Err(cim_core_domain::DomainError::ValidationError(
                    "No employment found for organization".to_string()
                ))
            }
        }

        PersonCommand::AddPosition { person_id, position } => {
            aggregate.add_component(position.clone(), "system", Some("Position added".to_string()))?;

            Ok(vec![PersonEvent::PositionAdded {
                person_id,
                position,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateSkills { person_id, skills } => {
            let old_skills = aggregate.get_component::<SkillsComponent>().cloned();
            aggregate.remove_component::<SkillsComponent>().ok();
            aggregate.add_component(skills.clone(), "system", Some("Skills update".to_string()))?;

            Ok(vec![PersonEvent::SkillsUpdated {
                person_id,
                old_skills,
                new_skills: skills,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::GrantAccess { person_id, access } => {
            aggregate.add_component(access.clone(), "system", Some("Access granted".to_string()))?;

            Ok(vec![PersonEvent::AccessGranted {
                person_id,
                access,
                granted_at: Utc::now(),
            }])
        }

        PersonCommand::AddExternalIdentifier { person_id, system, identifier } => {
            let mut external_ids = aggregate.get_component::<ExternalIdentifiersComponent>()
                .cloned()
                .unwrap_or_else(|| ExternalIdentifiersComponent {
                    ldap_dn: None,
                    ad_sid: None,
                    oauth_subjects: HashMap::new(),
                    external_ids: HashMap::new(),
                });

            external_ids.external_ids.insert(system.clone(), identifier.clone());

            aggregate.remove_component::<ExternalIdentifiersComponent>().ok();
            aggregate.add_component(external_ids, "system", Some("External ID added".to_string()))?;

            Ok(vec![PersonEvent::ExternalIdentifierAdded {
                person_id,
                system,
                identifier,
                added_at: Utc::now(),
            }])
        }
    }
}
