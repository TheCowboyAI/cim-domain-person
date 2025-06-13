//! Query handlers for person domain

use crate::{
    aggregate::{Person, PersonId},
    projections::{PersonProjection, EmployeeView, LdapProjection},
    queries::PersonQuery,
};
use cim_core_domain::DomainResult;
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
