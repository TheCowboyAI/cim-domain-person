//! Debug test for lifecycle transitions

use cim_domain_person::{
    aggregate::{Person, PersonId, PersonLifecycle},
    commands::{PersonCommand, DeactivatePerson, ReactivatePerson},
    value_objects::PersonName,
};

#[test]
fn test_reactivation_flow() {
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Test".to_string(), "User".to_string()),
    );
    
    // Step 1: Deactivate
    let deactivate_cmd = PersonCommand::DeactivatePerson(DeactivatePerson {
        person_id: person.id,
        reason: "Test".to_string(),
    });
    
    let result = person.handle_command(deactivate_cmd);
    assert!(result.is_ok(), "Deactivation should succeed, but got: {:?}", result);
    
    // Check the lifecycle after deactivation
    println!("Lifecycle after deactivation: {:?}", person.lifecycle);
    assert!(matches!(person.lifecycle, PersonLifecycle::Deactivated { .. }), 
            "Should be deactivated but is: {:?}", person.lifecycle);
    
    // Step 2: Reactivate
    let reactivate_cmd = PersonCommand::ReactivatePerson(ReactivatePerson {
        person_id: person.id,
        reason: "Test reactivation".to_string(),
    });
    
    let result = person.handle_command(reactivate_cmd);
    assert!(result.is_ok(), "Reactivation should succeed, but got: {:?}", result);
    
    // Check the lifecycle after reactivation
    println!("Lifecycle after reactivation: {:?}", person.lifecycle);
    assert!(matches!(person.lifecycle, PersonLifecycle::Active),
            "Should be active but is: {:?}", person.lifecycle);
}