//! Test component registration with user tracking

use cim_domain_person::{
    aggregate::{Person, PersonId, ComponentType},
    value_objects::PersonName,
};

#[test]
fn test_register_component_with_user_tracking() {
    let mut person = Person::new(
        PersonId::new(),
        PersonName::new("Test".to_string(), "User".to_string()),
    );

    // Register component with tracking
    let result = person.register_component_with_source(
        ComponentType::EmailAddress, 
        "hr_system".to_string()
    );
    
    assert!(result.is_ok());
    assert!(person.has_component(&ComponentType::EmailAddress));
    
    println!("✓ Component registration with user tracking works");
}