//! Simple example demonstrating the ECS-oriented Person domain
//! 
//! This example shows basic usage of the Person domain with components.

use cim_domain_person::{
    aggregate::{Person, PersonId, PersonLifecycle, ComponentType},
    value_objects::{PersonName, EmailAddress, PhoneNumber},
    components::{
        contact::{EmailComponent, PhoneComponent, ContactContext},
        skills::{SkillComponent, SkillCategory, ProficiencyLevel},
        ComponentMetadata,
    },
};
use chrono::{Utc, NaiveDate};

fn main() {
    println!("=== CIM Person Domain - Simple ECS Demo ===\n");
    
    // Create a new person with just core identity
    let person_id = PersonId::new();
    let name = PersonName::new("Alice".to_string(), "Johnson".to_string());
    let mut person = Person::new(person_id, name);
    
    println!("Created person: {person.core_identity.legal_name} ({person_id})");
    println!("Initial state: {:?}\n", person.lifecycle);
    
    // Register components
    demo_component_registration(&mut person);
    
    // Show component data
    demo_component_data();
    
    // Demonstrate lifecycle
    demo_lifecycle(&mut person);
}

fn demo_component_registration(person: &mut Person) {
    println!("--- Component Registration ---");
    
    // Register email component
    match person.register_component(ComponentType::EmailAddress) {
        Ok(_) => println!("✓ Registered EmailAddress component"),
        Err(e) => println!("✗ Failed to register EmailAddress: {e}"),
    }
    
    // Register phone component
    match person.register_component(ComponentType::PhoneNumber) {
        Ok(_) => println!("✓ Registered PhoneNumber component"),
        Err(e) => println!("✗ Failed to register PhoneNumber: {e}"),
    }
    
    // Register skill component
    match person.register_component(ComponentType::Skill) {
        Ok(_) => println!("✓ Registered Skill component"),
        Err(e) => println!("✗ Failed to register Skill: {e}"),
    }
    
    println!("\nTotal components registered: {}", person.components.len());
    println!();
}

fn demo_component_data() {
    println!("--- Component Data Examples ---");
    
    // Create metadata
    let metadata = ComponentMetadata {
        attached_at: Utc::now(),
        updated_at: Utc::now(),
        source: "demo".to_string(),
        version: 1,
    };
    
    // Email component
    let email = EmailComponent {
        email: EmailAddress {
            address: "alice@example.com".to_string(),
            verified: true,
        },
        is_primary: true,
        context: ContactContext::Personal,
        metadata: metadata.clone(),
    };
    
    println!("Email Component:");
    println!("  Address: {email.email.address}");
    println!("  Verified: {email.email.verified}");
    println!("  Primary: {email.is_primary}");
    println!("  Context: {:?}", email.context);
    
    // Phone component
    let phone = PhoneComponent {
        phone: PhoneNumber {
            number: "+1-555-123-4567".to_string(),
            country_code: Some("1".to_string()),
            extension: None,
            sms_capable: true,
        },
        is_primary: true,
        context: ContactContext::Work,
        metadata: metadata.clone(),
    };
    
    println!("\nPhone Component:");
    println!("  Number: {phone.phone.number}");
    println!("  Country Code: {:?}", phone.phone.country_code);
    println!("  SMS Capable: {phone.phone.sms_capable}");
    println!("  Primary: {phone.is_primary}");
    println!("  Context: {:?}", phone.context);
    
    // Skill component
    let skill = SkillComponent {
        skill_id: "rust-programming".to_string(),
        name: "Rust Programming".to_string(),
        category: SkillCategory::Technical,
        proficiency: ProficiencyLevel::Advanced,
        years_experience: Some(3.5),
        last_used: Some(NaiveDate::from_ymd_opt(2024, 1, 20).unwrap()),
        metadata,
    };
    
    println!("\nSkill Component:");
    println!("  Name: {skill.name}");
    println!("  Category: {:?}", skill.category);
    println!("  Proficiency: {:?}", skill.proficiency);
    if let Some(years) = skill.years_experience {
        println!("  Experience: {years} years");
    }
    println!();
}

fn demo_lifecycle(person: &mut Person) {
    println!("--- Lifecycle Management ---");
    
    // Set birth date
    let birth_date = NaiveDate::from_ymd_opt(1985, 3, 15).unwrap();
    person.core_identity.birth_date = Some(birth_date);
    println!("Set birth date: {birth_date}");
    
    // Deactivate person
    println!("\nDeactivating person...");
    person.lifecycle = PersonLifecycle::Deactivated {
        reason: "Account suspended for review".to_string(),
        since: Utc::now(),
    };
    
    // Try to add component while deactivated
    match person.register_component(ComponentType::Address) {
        Ok(_) => println!("✓ Added component while deactivated"),
        Err(e) => println!("✗ Cannot add component while deactivated: {e}"),
    }
    
    // Reactivate
    println!("\nReactivating person...");
    person.lifecycle = PersonLifecycle::Active;
    println!("Person is now active: {}", person.is_active());
} 