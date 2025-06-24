//! Standalone test for person merge functionality
//! 
//! This tests the merge functionality added to the Person domain
//! which allows merging two person entities that are found to be the same person.

use cim_domain::{EntityId};
use cim_domain_person::{
    aggregate::{Person, PersonMarker, MergeStatus},
    commands::{MergeReason},
    value_objects::{NameComponent, EmailComponent, PhoneComponent, EmailType, PhoneType},
};

/// Test that a person can be marked as merged into another
/// 
/// ```mermaid
/// graph LR
///     A[Person A - Active] -->|mark_as_merged_into| B[Person B]
///     A -->|Status| C[MergedInto B]
/// ```
#[test]
fn test_mark_person_as_merged() {
    let person_a_id = EntityId::<PersonMarker>::new();
    let person_b_id = EntityId::<PersonMarker>::new();

    let mut person_a = Person::new(
        person_a_id,
        NameComponent {
            given_name: "John".to_string(),
            family_name: "Doe".to_string(),
            middle_names: vec![],
            display_name: "John Doe".to_string(),
            name_order: cim_domain_person::value_objects::NameOrder::WesternOrder,
        },
        None,
        None,
    );

    // Mark person A as merged into person B
    let result = person_a.mark_as_merged_into(person_b_id, MergeReason::DuplicateIdentity);
    assert!(result.is_ok());

    // Check merge status
    match &person_a.merge_status {
        MergeStatus::MergedInto { target_person_id, reason, .. } => {
            assert_eq!(*target_person_id, person_b_id);
            assert_eq!(*reason, MergeReason::DuplicateIdentity);
        }
        _ => panic!("Expected MergedInto status"),
    }
}

/// Test that a merged person cannot be modified
/// 
/// ```mermaid
/// graph LR
///     A[Person - Merged] -->|add_email| B[Error: Cannot modify]
///     A -->|add_phone| C[Error: Cannot modify]
/// ```
#[test]
fn test_merged_person_cannot_be_modified() {
    let person_a_id = EntityId::<PersonMarker>::new();
    let person_b_id = EntityId::<PersonMarker>::new();

    let mut person = Person::new(
        person_a_id,
        NameComponent {
            given_name: "Jane".to_string(),
            family_name: "Smith".to_string(),
            middle_names: vec![],
            display_name: "Jane Smith".to_string(),
            name_order: cim_domain_person::value_objects::NameOrder::WesternOrder,
        },
        None,
        None,
    );

    // Mark as merged
    person.mark_as_merged_into(person_b_id, MergeReason::DuplicateIdentity).unwrap();

    // Try to add email - should fail
    let email = EmailComponent {
        email: "jane@example.com".to_string(),
        email_type: EmailType::Personal,
        verified: false,
        primary: true,
    };
    let result = person.add_email(email);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot modify merged person"));

    // Try to add phone - should fail
    let phone = PhoneComponent {
        number: "+1234567890".to_string(),
        phone_type: PhoneType::Mobile,
        verified: false,
        primary: true,
    };
    let result = person.add_phone(phone);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot modify merged person"));
}

/// Test transferable components collection
/// 
/// ```mermaid
/// graph LR
///     A[Person with Data] -->|get_transferable_components| B[Components Bundle]
///     B -->|emails| C[Email Set]
///     B -->|phones| D[Phone Set]
///     B -->|addresses| E[Address Set]
/// ```
#[test]
fn test_get_transferable_components() {
    let person_id = EntityId::<PersonMarker>::new();
    
    let mut person = Person::new(
        person_id,
        NameComponent {
            given_name: "Bob".to_string(),
            family_name: "Johnson".to_string(),
            middle_names: vec![],
            display_name: "Bob Johnson".to_string(),
            name_order: cim_domain_person::value_objects::NameOrder::WesternOrder,
        },
        Some(EmailComponent {
            email: "bob@example.com".to_string(),
            email_type: EmailType::Work,
            verified: true,
            primary: true,
        }),
        Some(PhoneComponent {
            number: "+1234567890".to_string(),
            phone_type: PhoneType::Work,
            verified: true,
            primary: true,
        }),
    );

    // Add additional email
    person.add_email(EmailComponent {
        email: "bob.personal@example.com".to_string(),
        email_type: EmailType::Personal,
        verified: false,
        primary: false,
    }).unwrap();

    // Get transferable components
    let components = person.get_transferable_components();

    // Verify components
    assert_eq!(components.emails.len(), 2);
    assert_eq!(components.phones.len(), 1);
    assert!(components.emails.iter().any(|e| e.email == "bob@example.com"));
    assert!(components.emails.iter().any(|e| e.email == "bob.personal@example.com"));
    assert!(components.phones.iter().any(|p| p.number == "+1234567890"));
}

/// Test applying transferred components
/// 
/// ```mermaid
/// graph LR
///     A[Components Bundle] -->|apply_transferred_components| B[Target Person]
///     B -->|Result| C[Person + New Components]
/// ```
#[test]
fn test_apply_transferred_components() {
    let target_id = EntityId::<PersonMarker>::new();
    
    let mut target = Person::new(
        target_id,
        NameComponent {
            given_name: "Robert".to_string(),
            family_name: "Johnson".to_string(),
            middle_names: vec![],
            display_name: "Robert Johnson".to_string(),
            name_order: cim_domain_person::value_objects::NameOrder::WesternOrder,
        },
        Some(EmailComponent {
            email: "robert@company.com".to_string(),
            email_type: EmailType::Work,
            verified: true,
            primary: true,
        }),
        None,
    );

    // Create components to transfer
    let transferred = cim_domain_person::events::TransferredComponents {
        emails: vec![
            EmailComponent {
                email: "bob@oldcompany.com".to_string(),
                email_type: EmailType::Work,
                verified: true,
                primary: false,
            },
            EmailComponent {
                email: "bob.personal@example.com".to_string(),
                email_type: EmailType::Personal,
                verified: false,
                primary: false,
            },
        ],
        phones: vec![
            PhoneComponent {
                number: "+9876543210".to_string(),
                phone_type: PhoneType::Mobile,
                verified: true,
                primary: true,
            },
        ],
        addresses: vec![],
        social_profiles: vec![],
        preferences: vec![],
    };

    // Apply transferred components
    target.apply_transferred_components(transferred);

    // Verify merge
    assert_eq!(target.emails.len(), 3); // Original + 2 transferred
    assert_eq!(target.phones.len(), 1); // 1 transferred
    assert!(target.emails.iter().any(|e| e.email == "robert@company.com"));
    assert!(target.emails.iter().any(|e| e.email == "bob@oldcompany.com"));
    assert!(target.emails.iter().any(|e| e.email == "bob.personal@example.com"));
    assert!(target.phones.iter().any(|p| p.number == "+9876543210"));
}

/// Test that a person cannot be merged into itself
/// 
/// ```mermaid
/// graph LR
///     A[Person] -->|merge_into_self| B[Error: Cannot merge into itself]
/// ```
#[test]
fn test_cannot_merge_into_self() {
    let person_id = EntityId::<PersonMarker>::new();
    
    let mut person = Person::new(
        person_id,
        NameComponent {
            given_name: "Self".to_string(),
            family_name: "Merge".to_string(),
            middle_names: vec![],
            display_name: "Self Merge".to_string(),
            name_order: cim_domain_person::value_objects::NameOrder::WesternOrder,
        },
        None,
        None,
    );

    // Try to merge into self
    let result = person.mark_as_merged_into(person_id, MergeReason::DuplicateIdentity);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot merge person into itself"));
}

/// Test that an archived person cannot be merged
/// 
/// ```mermaid
/// graph LR
///     A[Person - Archived] -->|merge| B[Error: Cannot merge archived]
/// ```
#[test]
fn test_archived_person_cannot_be_merged() {
    let person_a_id = EntityId::<PersonMarker>::new();
    let person_b_id = EntityId::<PersonMarker>::new();
    
    let mut person = Person::new(
        person_a_id,
        NameComponent {
            given_name: "Archived".to_string(),
            family_name: "Person".to_string(),
            middle_names: vec![],
            display_name: "Archived Person".to_string(),
            name_order: cim_domain_person::value_objects::NameOrder::WesternOrder,
        },
        None,
        None,
    );

    // Archive the person
    person.archive().unwrap();

    // Try to merge - should fail
    let result = person.mark_as_merged_into(person_b_id, MergeReason::DuplicateIdentity);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot merge archived person"));
}

/// Test that a person already merged cannot be merged again
/// 
/// ```mermaid
/// graph LR
///     A[Person - Already Merged] -->|merge_again| B[Error: Already merged]
/// ```
#[test]
fn test_already_merged_person_cannot_be_merged_again() {
    let person_a_id = EntityId::<PersonMarker>::new();
    let person_b_id = EntityId::<PersonMarker>::new();
    let person_c_id = EntityId::<PersonMarker>::new();
    
    let mut person = Person::new(
        person_a_id,
        NameComponent {
            given_name: "Already".to_string(),
            family_name: "Merged".to_string(),
            middle_names: vec![],
            display_name: "Already Merged".to_string(),
            name_order: cim_domain_person::value_objects::NameOrder::WesternOrder,
        },
        None,
        None,
    );

    // First merge
    person.mark_as_merged_into(person_b_id, MergeReason::DuplicateIdentity).unwrap();

    // Try to merge again - should fail
    let result = person.mark_as_merged_into(person_c_id, MergeReason::DuplicateIdentity);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Person already merged"));
} 