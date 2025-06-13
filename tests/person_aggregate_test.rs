use cim_domain_person::{
    Person, PersonId, PersonCommand, PersonEvent,
    IdentityComponent, ContactComponent, EmailAddress,
};
use cim_core_domain::{EntityId, AggregateRoot};
use uuid::Uuid;

#[test]
fn test_person_creation() {
    let person_id = PersonId::from_uuid(Uuid::new_v4());
    let identity = IdentityComponent {
        legal_name: "John Doe".to_string(),
        preferred_name: Some("Johnny".to_string()),
        date_of_birth: None,
        government_id: None,
    };

    let person = Person::new(person_id, identity.clone());

    assert_eq!(person.id(), person_id);
    assert_eq!(person.version(), 0);
    assert!(person.has_component::<IdentityComponent>());
}

#[test]
fn test_add_contact_component() {
    let person_id = PersonId::from_uuid(Uuid::new_v4());
    let identity = IdentityComponent {
        legal_name: "Jane Doe".to_string(),
        preferred_name: None,
        date_of_birth: None,
        government_id: None,
    };

    let mut person = Person::new(person_id, identity);

    let contact = ContactComponent {
        emails: vec![EmailAddress {
            email: "jane@example.com".to_string(),
            email_type: "work".to_string(),
            is_primary: true,
            is_verified: false,
        }],
        phones: vec![],
        addresses: vec![],
    };

    let result = person.add_component(contact.clone(), "test", Some("Adding contact".to_string()));
    assert!(result.is_ok());
    assert!(person.has_component::<ContactComponent>());
    assert_eq!(person.version(), 1);
}
