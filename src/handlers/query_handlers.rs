//! Query handlers for person domain

use crate::{
    aggregate::PersonId,
    projections::{PersonProjection, EmployeeView, LdapProjection},
    queries::PersonQuery,
};
use cim_domain::DomainResult;
use std::collections::HashMap;

/// Person read model for queries
pub struct PersonReadModel {
    projections: HashMap<PersonId, PersonProjection>,
}

impl PersonReadModel {
    pub fn new() -> Self {
        Self {
            projections: HashMap::new(),
        }
    }

    /// Handle a person query
    pub async fn handle_query(&self, query: PersonQuery) -> DomainResult<PersonQueryResult> {
        match query {
            PersonQuery::GetPersonById { person_id } => {
                if let Some(projection) = self.projections.get(&person_id) {
                    Ok(PersonQueryResult::Person(projection.clone()))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::FindPersonByEmail { email } => {
                let found: Vec<_> = self.projections.values()
                    .filter(|p| p.emails.iter().any(|e| e.email == email))
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(found))
            }

            PersonQuery::ListActivePeople { limit, offset } => {
                let people: Vec<_> = self.projections.values()
                    .filter(|p| p.is_active)
                    .skip(offset)
                    .take(limit)
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(people))
            }

            PersonQuery::SearchPeopleByName { name_pattern } => {
                let pattern = name_pattern.to_lowercase();
                let found: Vec<_> = self.projections.values()
                    .filter(|p| p.name.to_lowercase().contains(&pattern))
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(found))
            }

            PersonQuery::GetEmployeeView { person_id } => {
                if let Some(projection) = self.projections.get(&person_id) {
                    // Create employee view from person projection
                    let employee_view = EmployeeView {
                        person_id: projection.person_id,
                        name: projection.name.clone(),
                        email: projection.emails.first().map(|e| e.email.clone()),
                        department: projection.department.clone(),
                        position: projection.position.clone(),
                        manager_id: projection.manager_id,
                        direct_reports: projection.direct_reports.clone(),
                        is_active: projection.is_active,
                    };
                    Ok(PersonQueryResult::EmployeeView(employee_view))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::GetLdapProjection { person_id, base_dn } => {
                if let Some(projection) = self.projections.get(&person_id) {
                    // Create LDAP projection from person projection
                    let ldap_projection = LdapProjection {
                        dn: format!("uid={},ou=people,{}", projection.person_id, base_dn),
                        uid: projection.person_id.to_string(),
                        cn: projection.name.clone(),
                        sn: projection.name.split_whitespace().last().unwrap_or("").to_string(),
                        given_name: projection.name.split_whitespace().next().unwrap_or("").to_string(),
                        mail: projection.emails.iter().map(|e| e.email.clone()).collect(),
                        telephone_number: projection.phones.iter().map(|p| p.number.clone()).collect(),
                        object_class: vec!["inetOrgPerson".to_string(), "person".to_string()],
                    };
                    Ok(PersonQueryResult::LdapProjection(ldap_projection))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::FindPeopleByOrganization { organization_id, include_inactive } => {
                let found: Vec<_> = self.projections.values()
                    .filter(|p| {
                        p.organization_id == Some(organization_id) &&
                        (include_inactive || p.is_active)
                    })
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(found))
            }

            PersonQuery::FindPeopleBySkill { skill_name, min_proficiency } => {
                let found: Vec<_> = self.projections.values()
                    .filter(|p| {
                        p.skills.iter().any(|s| {
                            let skill_matches = s.skill.to_lowercase() == skill_name.to_lowercase();
                            let proficiency_matches = min_proficiency.as_ref()
                                .map(|min| s.level >= *min)
                                .unwrap_or(true);
                            skill_matches && proficiency_matches
                        })
                    })
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(found))
            }

            PersonQuery::FindPeopleByRole { role } => {
                let found: Vec<_> = self.projections.values()
                    .filter(|p| {
                        p.roles.iter().any(|r| r.to_lowercase() == role.to_lowercase())
                    })
                    .cloned()
                    .collect();

                Ok(PersonQueryResult::People(found))
            }
        }
    }
}

/// Result types for person queries
#[derive(Debug, Clone)]
pub enum PersonQueryResult {
    Person(PersonProjection),
    People(Vec<PersonProjection>),
    EmployeeView(EmployeeView),
    LdapProjection(LdapProjection),
    NotFound,
}
