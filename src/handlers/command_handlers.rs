//! Command handlers for the Person domain

use crate::aggregate::{Person, PersonId};
use crate::commands::*;
use crate::events::*;
use crate::value_objects::*;
use cim_domain::{DomainError, DomainResult};

/// Create a new person
pub fn handle_create_person(
    person_id: PersonId,
    name: PersonName,
    source: String,
) -> DomainResult<(Person, Vec<PersonEvent>)> {
    let person = Person::new(person_id, name.clone());
    
    let event = PersonEvent::PersonCreated(PersonCreated {
        person_id,
        name,
        source,
        created_at: chrono::Utc::now(),
    });
    
    Ok((person, vec![event]))
}

/// Handle a command for an existing person
pub fn handle_person_command(
    person: &mut Person,
    command: PersonCommand,
) -> DomainResult<Vec<PersonEvent>> {
    person.handle_command(command)
        .map_err(DomainError::ValidationError)
}
