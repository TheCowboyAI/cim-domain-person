//! Basic tests for the Person domain

use cim_domain_person::{
    commands::PersonCommand,
    events::PersonEvent,
    handlers::{handle_create_person, handle_person_command},
    value_objects::{PersonName, EmailAddress, PhoneNumber, PhysicalAddress},
};

/// Test creating a person
///
/// ```mermaid
/// graph LR
///     A[Create Person] --> B[PersonCreated Event]
///     B --> C[Person Aggregate]
/// ```
#[tokio::test]
async fn test_create_person() {
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let email = EmailAddress::new("john@example.com".to_string());
    
    let (person, events) = handle_create_person(name.clone(), Some(email.clone()))
        .await
        .unwrap();
    
    // Verify person state
    assert_eq!(person.name.given_name, "John");
    assert_eq!(person.name.family_name, "Doe");
    assert!(person.is_active);
    assert_eq!(person.emails.len(), 1);
    assert_eq!(person.emails.get("primary").unwrap().address, "john@example.com");
    
    // Verify events
    assert_eq!(events.len(), 2);
    match &events[0] {
        PersonEvent::PersonCreated { name: event_name, .. } => {
            assert_eq!(event_name.given_name, "John");
            assert_eq!(event_name.family_name, "Doe");
        }
        _ => panic!("Expected PersonCreated event"),
    }
    match &events[1] {
        PersonEvent::EmailAdded { label, email: event_email, .. } => {
            assert_eq!(label, "primary");
            assert_eq!(event_email.address, "john@example.com");
        }
        _ => panic!("Expected EmailAdded event"),
    }
}

/// Test updating a person's name
///
/// ```mermaid
/// graph LR
///     A[Update Name Command] --> B[NameUpdated Event]
///     B --> C[Updated Person]
/// ```
#[tokio::test]
async fn test_update_name() {
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let (mut person, _) = handle_create_person(name, None)
        .await
        .unwrap();
    
    // Update name
    let new_name = PersonName::new("Jane".to_string(), "Smith".to_string());
    let events = handle_person_command(
        &mut person,
        PersonCommand::UpdateName { name: new_name.clone() }
    )
    .await
    .unwrap();
    
    // Verify state
    assert_eq!(person.name.given_name, "Jane");
    assert_eq!(person.name.family_name, "Smith");
    
    // Verify event
    assert_eq!(events.len(), 1);
    match &events[0] {
        PersonEvent::NameUpdated { old_name, new_name: event_name, .. } => {
            assert_eq!(old_name.given_name, "John");
            assert_eq!(event_name.given_name, "Jane");
        }
        _ => panic!("Expected NameUpdated event"),
    }
}

/// Test adding and removing emails
///
/// ```mermaid
/// graph LR
///     A[Add Email] --> B[EmailAdded Event]
///     B --> C[Remove Email]
///     C --> D[EmailRemoved Event]
/// ```
#[tokio::test]
async fn test_email_management() {
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let (mut person, _) = handle_create_person(name, None)
        .await
        .unwrap();
    
    // Add work email
    let work_email = EmailAddress::new("john@work.com".to_string());
    let events = handle_person_command(
        &mut person,
        PersonCommand::AddEmail {
            label: "work".to_string(),
            email: work_email.clone(),
        }
    )
    .await
    .unwrap();
    
    assert_eq!(events.len(), 1);
    assert!(person.emails.contains_key("work"));
    
    // Remove work email
    let events = handle_person_command(
        &mut person,
        PersonCommand::RemoveEmail {
            label: "work".to_string(),
        }
    )
    .await
    .unwrap();
    
    assert_eq!(events.len(), 1);
    assert!(!person.emails.contains_key("work"));
}

/// Test deactivating and reactivating a person
///
/// ```mermaid
/// graph LR
///     A[Active Person] --> B[Deactivate]
///     B --> C[Inactive Person]
///     C --> D[Reactivate]
///     D --> E[Active Person]
/// ```
#[tokio::test]
async fn test_person_activation() {
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let (mut person, _) = handle_create_person(name, None)
        .await
        .unwrap();
    
    assert!(person.is_active);
    
    // Deactivate
    let events = handle_person_command(&mut person, PersonCommand::Deactivate)
        .await
        .unwrap();
    
    assert_eq!(events.len(), 1);
    assert!(!person.is_active);
    
    // Try to update inactive person (should fail)
    let new_name = PersonName::new("Jane".to_string(), "Doe".to_string());
    let result = handle_person_command(
        &mut person,
        PersonCommand::UpdateName { name: new_name }
    )
    .await;
    
    assert!(result.is_err());
    
    // Reactivate
    let events = handle_person_command(&mut person, PersonCommand::Reactivate)
        .await
        .unwrap();
    
    assert_eq!(events.len(), 1);
    assert!(person.is_active);
}

/// Test adding phone numbers and addresses
///
/// ```mermaid
/// graph LR
///     A[Add Phone] --> B[PhoneAdded Event]
///     B --> C[Add Address]
///     C --> D[AddressAdded Event]
/// ```
#[tokio::test]
async fn test_contact_information() {
    let name = PersonName::new("John".to_string(), "Doe".to_string());
    let (mut person, _) = handle_create_person(name, None)
        .await
        .unwrap();
    
    // Add phone
    let phone = PhoneNumber::with_country("555-1234".to_string(), "1".to_string());
    let events = handle_person_command(
        &mut person,
        PersonCommand::AddPhone {
            label: "mobile".to_string(),
            phone: phone.clone(),
        }
    )
    .await
    .unwrap();
    
    assert_eq!(events.len(), 1);
    assert_eq!(person.phones.get("mobile").unwrap().number, "555-1234");
    
    // Add address
    let address = PhysicalAddress::new(
        "123 Main St".to_string(),
        "Springfield".to_string(),
        "USA".to_string(),
    );
    let events = handle_person_command(
        &mut person,
        PersonCommand::AddAddress {
            label: "home".to_string(),
            address: address.clone(),
        }
    )
    .await
    .unwrap();
    
    assert_eq!(events.len(), 1);
    assert_eq!(person.addresses.get("home").unwrap().city, "Springfield");
} 