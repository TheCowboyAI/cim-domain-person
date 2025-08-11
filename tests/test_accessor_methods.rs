//! Test accessor methods for Person aggregate

use cim_domain_person::{
    aggregate::{Person, PersonId},
    value_objects::PersonName,
};

#[test]
fn test_person_accessor_methods() {
    let person = Person::new(
        PersonId::new(),
        PersonName::new("Test".to_string(), "User".to_string()),
    );

    // Test that accessor methods exist and work
    let identity = person.core_identity();
    assert_eq!(identity.legal_name, PersonName::new("Test".to_string(), "User".to_string()));
    
    let lifecycle = person.lifecycle();
    assert!(matches!(lifecycle, cim_domain_person::aggregate::PersonLifecycle::Active));
    
    let count = person.component_count();
    assert_eq!(count, 0);
    
    let types = person.component_types();
    assert!(types.is_empty());
    
    println!("✓ All accessor methods work correctly");
}