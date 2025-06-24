//! Tests for person merge functionality
//! 
//! This tests the merge functionality added to the Person domain
//! which allows merging two person entities that are found to be the same person.

use cim_domain::{EntityId, AggregateRoot};
use cim_domain_person::{
    aggregate::{Person, PersonMarker, MergeStatus},
    commands::{MergeReason},
    value_objects::{NameComponent, EmailComponent, PhoneComponent, EmailType, PhoneType},
};
use std::collections::HashMap;

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
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            middle_name: None,
            title: None,
            suffix: None,
        },
        Some(EmailComponent {
            email: "john.doe@example.com".to_string(),
            email_type: EmailType::Personal,
            verified: true,
            primary: true,
        }),
        None,
    );

    // Person should start as active
    assert!(person_a.is_active());
    assert_eq!(person_a.get_merge_target(), None);

    // Mark as merged
    let merge_reason = MergeReason::SameEmailVerified {
        email: "john.doe@example.com".to_string(),
    };

    let event = person_a.mark_as_merged_into(person_b_id, merge_reason.clone()).unwrap();

    // Check the person is now marked as merged
    assert!(!person_a.is_active());
    assert_eq!(person_a.get_merge_target(), Some(person_b_id));

    // Check merge status
    match &person_a.merge_status {
        MergeStatus::MergedInto { target_person_id, reason, .. } => {
            assert_eq!(*target_person_id, person_b_id);
            assert_eq!(reason, &merge_reason);
        }
        _ => panic!("Expected MergedInto status"),
    }
}

/// Test that an already merged person cannot be merged again
/// 
/// ```mermaid
/// graph LR
///     A[Person A] -->|Merged Into| B[Person B]
///     A -->|Try Merge Into| C[Person C]
///     A -->|Error| D[Already Merged]
/// ```
#[test]
fn test_cannot_merge_already_merged_person() {
    let person_a_id = EntityId::<PersonMarker>::new();
    let person_b_id = EntityId::<PersonMarker>::new();
    let person_c_id = EntityId::<PersonMarker>::new();

    let mut person_a = Person::new(
        person_a_id,
        NameComponent {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            middle_name: None,
            title: None,
            suffix: None,
        },
        None,
        None,
    );

    // First merge should succeed
    let merge_reason = MergeReason::ManualMerge {
        admin_id: "admin123".to_string(),
        justification: "Duplicate detected".to_string(),
    };

    person_a.mark_as_merged_into(person_b_id, merge_reason.clone()).unwrap();

    // Second merge should fail
    let result = person_a.mark_as_merged_into(person_c_id, merge_reason);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.to_string().contains("already merged"));
}

/// Test that an archived (but not merged) person cannot be merged
/// 
/// ```mermaid
/// graph LR
///     A[Person - Archived] -->|Try Merge| B[Error]
/// ```
#[test]
fn test_cannot_merge_archived_person() {
    let person_a_id = EntityId::<PersonMarker>::new();
    let person_b_id = EntityId::<PersonMarker>::new();

    let mut person_a = Person::new(
        person_a_id,
        NameComponent {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            middle_name: None,
            title: None,
            suffix: None,
        },
        None,
        None,
    );

    // Archive the person
    person_a.archived = true;

    // Try to merge - should fail
    let merge_reason = MergeReason::ManualMerge {
        admin_id: "admin123".to_string(),
        justification: "Test merge".to_string(),
    };

    let result = person_a.mark_as_merged_into(person_b_id, merge_reason);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.to_string().contains("archived"));
}

/// Test getting transferable components
/// 
/// ```mermaid
/// graph TD
///     A[Person with Components] -->|get_transferable_components| B[TransferredComponents]
///     B --> C[Emails]
///     B --> D[Phones]
///     B --> E[Addresses]
///     B --> F[Social Profiles]
/// ```
#[test]
fn test_get_transferable_components() {
    let person_id = EntityId::<PersonMarker>::new();

    let mut person = Person::new(
        person_id,
        NameComponent {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            middle_name: None,
            title: None,
            suffix: None,
        },
        Some(EmailComponent {
            email: "john@example.com".to_string(),
            email_type: EmailType::Personal,
            verified: true,
            primary: true,
        }),
        Some(PhoneComponent {
            number: "+1234567890".to_string(),
            phone_type: PhoneType::Mobile,
            verified: true,
            primary: true,
        }),
    );

    // Add additional email
    person.emails.insert(EmailComponent {
        email: "john.work@example.com".to_string(),
        email_type: EmailType::Work,
        verified: false,
        primary: false,
    });

    let components = person.get_transferable_components();

    assert_eq!(components.emails.len(), 2);
    assert_eq!(components.phones.len(), 1);
    assert_eq!(components.addresses.len(), 0);
    assert_eq!(components.social_profiles.len(), 0);
}

/// Test all merge reasons are valid
/// 
/// ```mermaid
/// graph TD
///     A[MergeReason] --> B[SameEmailVerified]
///     A --> C[SamePhoneVerified]
///     A --> D[GovernmentIdMatch]
///     A --> E[ManualMerge]
///     A --> F[AutoDetectedDuplicate]
///     A --> G[CustomerReported]
///     A --> H[Other]
/// ```
#[test]
fn test_merge_reason_variants() {
    let reasons = vec![
        MergeReason::SameEmailVerified {
            email: "test@example.com".to_string(),
        },
        MergeReason::SamePhoneVerified {
            phone: "+1234567890".to_string(),
        },
        MergeReason::GovernmentIdMatch {
            id_type: "SSN".to_string(),
            masked_id: "***-**-1234".to_string(),
        },
        MergeReason::ManualMerge {
            admin_id: "admin123".to_string(),
            justification: "Customer confirmed duplicate".to_string(),
        },
        MergeReason::AutoDetectedDuplicate {
            confidence: 0.95,
            algorithm: "fuzzy_matching_v2".to_string(),
        },
        MergeReason::CustomerReported {
            ticket_id: "TICKET-12345".to_string(),
        },
        MergeReason::Other {
            description: "Special case merge".to_string(),
        },
    ];

    // Test that all reasons can be used
    for (i, reason) in reasons.into_iter().enumerate() {
        let person_id = EntityId::<PersonMarker>::new();
        let target_id = EntityId::<PersonMarker>::new();

        let mut person = Person::new(
            person_id,
            NameComponent {
                first_name: format!("Test{}", i),
                last_name: "Person".to_string(),
                middle_name: None,
                title: None,
                suffix: None,
            },
            None,
            None,
        );

        let result = person.mark_as_merged_into(target_id, reason.clone());
        assert!(result.is_ok(), "Failed to merge with reason: {:?}", reason);
    }
} 