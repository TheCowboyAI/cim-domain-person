//! Example demonstrating CRM-focused person composition
//!
//! This example shows how to compose person entities with various components
//! to create different views and concepts for Customer Relationship Management.

use cim_domain::AggregateRoot;
use cim_domain_person::{
    PersonCompositionService, NameComponent, ContactComponent, EmailAddress,
    PhoneNumber, PreferencesComponent, CommunicationPreferences, ContactChannel,
    ChannelSettings, ContactTimePreference, FrequencyPreference, ContentPreferences,
    ContentType, ContentFormat, ComplexityLevel, PrivacyPreferences,
    PhysicalAttributesComponent, VisionCorrection, RelationshipComponent,
    Relationship, RelationshipType, RelationshipStatus, SocialMediaComponent,
    SocialMediaProfile, SocialPlatform, PrivacySetting, InterestsComponent,
    InterestCategory, Interest, BehavioralComponent, PurchaseBehavior,
    EngagementPatterns, InteractionSummary, PredictiveScores,
    EmployeeViewBuilder, CustomerViewBuilder, PartnerViewBuilder,
};
use std::collections::HashMap;

fn main() {
    println!("=== CRM Person Composition Example ===\n");

    // Example 1: Create a comprehensive customer profile
    create_customer_example();
    
    println!("\n{}\n", "=".repeat(50));
    
    // Example 2: Create an employee with full details
    create_employee_example();
    
    println!("\n{}\n", "=".repeat(50));
    
    // Example 3: Create a business partner
    create_partner_example();
    
    println!("\n{}\n", "=".repeat(50));
    
    // Example 4: Demonstrate complex naming
    demonstrate_complex_names();
}

fn create_customer_example() {
    println!("Creating a comprehensive customer profile...\n");
    
    // Create a customer with complex name
    let name = NameComponent {
        title: Some("Dr.".to_string()),
        honorific: Some("Ms.".to_string()),
        given_names: vec!["Sarah".to_string()],
        middle_names: vec!["Elizabeth".to_string()],
        family_names: vec!["Johnson".to_string()],
        maternal_family_name: None,
        generational_suffix: None,
        professional_suffix: Some("PhD".to_string()),
        preferred_name: Some("Sarah".to_string()),
        name_order: cim_domain_person::NameOrder::GivenFirst,
        cultural_context: Some("American".to_string()),
    };
    
    println!("Customer Name: {}", name.formatted_name());
    println!("Preferred Name: {}", name.display_name());
    
    // Create contact information
    let contact = ContactComponent {
        emails: vec![
            EmailAddress {
                email: "sarah.johnson@email.com".to_string(),
                email_type: "personal".to_string(),
                is_primary: true,
                is_verified: true,
            },
            EmailAddress {
                email: "dr.johnson@university.edu".to_string(),
                email_type: "work".to_string(),
                is_primary: false,
                is_verified: true,
            },
        ],
        phones: vec![
            PhoneNumber {
                number: "+1-555-123-4567".to_string(),
                phone_type: "mobile".to_string(),
                is_primary: true,
                sms_capable: true,
            },
        ],
        addresses: vec![], // Would reference Location aggregates
    };
    
    // Create communication preferences
    let mut channel_settings = HashMap::new();
    channel_settings.insert(
        ContactChannel::Email,
        ChannelSettings {
            opted_in: true,
            marketing_allowed: true,
            transactional_allowed: true,
            last_consent_date: Some(chrono::Local::now().naive_local().date()),
        },
    );
    channel_settings.insert(
        ContactChannel::SMS,
        ChannelSettings {
            opted_in: true,
            marketing_allowed: false,
            transactional_allowed: true,
            last_consent_date: Some(chrono::Local::now().naive_local().date()),
        },
    );
    
    let preferences = PreferencesComponent {
        communication: CommunicationPreferences {
            preferred_channel: ContactChannel::Email,
            channel_settings,
            contact_time_preference: ContactTimePreference {
                preferred_days: vec![
                    chrono::Weekday::Tue,
                    chrono::Weekday::Thu,
                    chrono::Weekday::Sat,
                ],
                preferred_hours: Some((10, 18)), // 10 AM to 6 PM
                timezone: "America/New_York".to_string(),
            },
            preferred_language: "en-US".to_string(),
            frequency_preference: FrequencyPreference::Weekly,
        },
        product_preferences: vec![],
        content_preferences: ContentPreferences {
            content_types: vec![
                ContentType::Educational,
                ContentType::CaseStudies,
                ContentType::HowTo,
            ],
            topics: vec![
                "Technology".to_string(),
                "Innovation".to_string(),
                "Research".to_string(),
            ],
            format_preference: ContentFormat::Mixed,
            complexity_preference: ComplexityLevel::Advanced,
        },
        privacy_preferences: PrivacyPreferences {
            data_sharing_allowed: false,
            analytics_allowed: true,
            personalization_allowed: true,
            third_party_sharing_allowed: false,
            regulatory_preferences: {
                let mut prefs = HashMap::new();
                prefs.insert("GDPR".to_string(), true);
                prefs.insert("CCPA".to_string(), true);
                prefs
            },
        },
    };
    
    // Create the customer
    let mut person = PersonCompositionService::create_customer(
        name,
        contact,
        preferences,
    ).unwrap();
    
    println!("\nCustomer created with ID: {:?}", person.id());
    
    // Add behavioral data
    let behavioral = BehavioralComponent {
        purchase_behavior: PurchaseBehavior {
            average_order_value: Some(285.50),
            purchase_frequency: Some(12.0), // 12 orders per year
            payment_methods: vec!["Credit Card".to_string(), "PayPal".to_string()],
            seasonal_patterns: {
                let mut patterns = HashMap::new();
                patterns.insert("Q1".to_string(), 0.2);
                patterns.insert("Q2".to_string(), 0.25);
                patterns.insert("Q3".to_string(), 0.3);
                patterns.insert("Q4".to_string(), 0.25);
                patterns
            },
            category_distribution: {
                let mut dist = HashMap::new();
                dist.insert("Electronics".to_string(), 0.45);
                dist.insert("Books".to_string(), 0.30);
                dist.insert("Software".to_string(), 0.25);
                dist
            },
            typical_price_range: Some((50.0, 500.0)),
            discount_sensitivity: Some(0.3), // Low sensitivity
        },
        engagement_patterns: EngagementPatterns {
            email_open_rate: Some(0.65),
            click_through_rate: Some(0.22),
            visit_frequency: Some(4.5), // visits per month
            avg_session_duration: Some(420), // 7 minutes
            device_usage: {
                let mut devices = HashMap::new();
                devices.insert("Desktop".to_string(), 0.7);
                devices.insert("Mobile".to_string(), 0.3);
                devices
            },
            active_hours: vec![0.1; 24], // Simplified
        },
        interaction_summary: InteractionSummary {
            total_interactions: 156,
            last_interaction: Some(chrono::Utc::now()),
            support_tickets: 3,
            avg_satisfaction: Some(4.8),
            channels_used: vec![ContactChannel::Email, ContactChannel::Phone],
        },
        predictive_scores: PredictiveScores {
            churn_risk: Some(0.05), // Very low
            predicted_ltv: Some(8500.0),
            purchase_probability: Some(0.85),
            upsell_potential: Some(0.7),
            referral_likelihood: Some(0.9),
        },
    };
    
    PersonCompositionService::add_behavioral_data(
        &mut person,
        behavioral,
        "analytics_system",
    ).unwrap();
    
    // Add interests
    let mut interests_map = HashMap::new();
    interests_map.insert(
        InterestCategory::Technology,
        vec![
            Interest {
                name: "Artificial Intelligence".to_string(),
                level: 10,
                duration_years: Some(5.0),
                specifics: vec![
                    "Machine Learning".to_string(),
                    "Neural Networks".to_string(),
                    "NLP".to_string(),
                ],
                activities: vec![
                    "Research".to_string(),
                    "Conference attendance".to_string(),
                    "Publishing papers".to_string(),
                ],
            },
        ],
    );
    interests_map.insert(
        InterestCategory::Reading,
        vec![
            Interest {
                name: "Science Fiction".to_string(),
                level: 8,
                duration_years: Some(15.0),
                specifics: vec![
                    "Hard sci-fi".to_string(),
                    "Cyberpunk".to_string(),
                ],
                activities: vec![
                    "Book club".to_string(),
                    "Writing reviews".to_string(),
                ],
            },
        ],
    );
    
    let interests = InterestsComponent {
        interests: interests_map,
        interest_profile: Some(cim_domain_person::InterestProfile {
            primary_category: InterestCategory::Technology,
            diversity_score: 0.8,
            activity_level: 0.9,
            social_preference: 0.6,
        }),
    };
    
    person.add_component(interests, "crm_system", Some("Interest profiling".to_string())).unwrap();
    
    // Build customer view
    if CustomerViewBuilder::can_build(&person) {
        let view = CustomerViewBuilder::build(&person).unwrap();
        println!("\nCustomer View created with {} components", view.components.len());
        println!("Components: {:?}", view.components);
    }
}

fn create_employee_example() {
    println!("Creating an employee profile...\n");
    
    // Create employee with formal name
    let name = NameComponent {
        title: None,
        honorific: Some("Mr.".to_string()),
        given_names: vec!["Robert".to_string()],
        middle_names: vec!["James".to_string()],
        family_names: vec!["Smith".to_string()],
        maternal_family_name: None,
        generational_suffix: Some("Jr.".to_string()),
        professional_suffix: Some("MBA".to_string()),
        preferred_name: Some("Bob".to_string()),
        name_order: cim_domain_person::NameOrder::GivenFirst,
        cultural_context: None,
    };
    
    println!("Employee Name: {}", name.formatted_name());
    println!("Goes by: {}", name.display_name());
    
    let employment = cim_domain_person::EmploymentComponent {
        organization_id: uuid::Uuid::new_v4(),
        employee_id: "EMP-2024-001".to_string(),
        title: "Senior Software Engineer".to_string(),
        department: Some("Engineering".to_string()),
        manager_id: Some(uuid::Uuid::new_v4()),
        status: "active".to_string(),
        start_date: chrono::NaiveDate::from_ymd_opt(2022, 3, 15).unwrap(),
        end_date: None,
    };
    
    let contact = ContactComponent {
        emails: vec![
            EmailAddress {
                email: "bob.smith@company.com".to_string(),
                email_type: "work".to_string(),
                is_primary: true,
                is_verified: true,
            },
        ],
        phones: vec![
            PhoneNumber {
                number: "+1-555-0100".to_string(),
                phone_type: "work".to_string(),
                is_primary: true,
                sms_capable: false,
            },
        ],
        addresses: vec![],
    };
    
    let mut person = PersonCompositionService::create_employee(
        name,
        employment,
        contact,
    ).unwrap();
    
    // Add physical attributes for ID badge
    let physical = PhysicalAttributesComponent {
        height_cm: Some(178.0),
        weight_kg: None, // Not needed for ID
        build: None,
        hair_color: Some("Brown".to_string()),
        hair_style: None,
        eye_color: Some("Green".to_string()),
        skin_tone: None,
        facial_hair: Some("Beard".to_string()),
        vision_correction: Some(VisionCorrection::Glasses),
        appearance_notes: Some("Usually wears glasses".to_string()),
    };
    
    PersonCompositionService::add_physical_attributes(
        &mut person,
        physical,
        "hr_system",
        Some("ID badge photo update".to_string()),
    ).unwrap();
    
    // Build employee view
    if EmployeeViewBuilder::can_build(&person) {
        let view = EmployeeViewBuilder::build(&person).unwrap();
        println!("\nEmployee View created with {} components", view.components.len());
    }
}

fn create_partner_example() {
    println!("Creating a business partner profile...\n");
    
    let name = NameComponent::simple("Alexandra".to_string(), "Chen".to_string());
    let mut person = PersonCompositionService::create_basic_person(name).unwrap();
    
    // Add contact
    let contact = ContactComponent {
        emails: vec![
            EmailAddress {
                email: "alexandra@techpartners.com".to_string(),
                email_type: "business".to_string(),
                is_primary: true,
                is_verified: true,
            },
        ],
        phones: vec![],
        addresses: vec![],
    };
    person.add_component(contact, "partner_system", None).unwrap();
    
    // Add business relationships
    let relationships = RelationshipComponent {
        relationships: vec![
            Relationship {
                person_id: uuid::Uuid::new_v4(),
                relationship_type: RelationshipType::BusinessPartner,
                reciprocal_type: RelationshipType::BusinessPartner,
                start_date: Some(chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
                end_date: None,
                status: RelationshipStatus::Active,
                notes: Some("Strategic technology partner".to_string()),
            },
        ],
    };
    PersonCompositionService::add_relationships(
        &mut person,
        relationships,
        "partner_system",
    ).unwrap();
    
    // Add social media for professional networking
    let social = SocialMediaComponent {
        profiles: vec![
            SocialMediaProfile {
                platform: SocialPlatform::LinkedIn,
                username: "alexandra-chen".to_string(),
                profile_url: Some("https://linkedin.com/in/alexandra-chen".to_string()),
                verified: true,
                privacy: PrivacySetting::Professional,
                last_active: Some(chrono::Utc::now()),
                follower_count: Some(5000),
            },
        ],
        metrics: None,
    };
    PersonCompositionService::add_social_media(
        &mut person,
        social,
        "partner_system",
    ).unwrap();
    
    // Build partner view
    if PartnerViewBuilder::can_build(&person) {
        let view = PartnerViewBuilder::build(&person).unwrap();
        println!("Partner View created with {} components", view.components.len());
        println!("Partner ID: {:?}", person.id());
    }
}

fn demonstrate_complex_names() {
    println!("Demonstrating complex naming scenarios...\n");
    
    // Spanish naming convention
    let spanish_name = NameComponent {
        title: Some("Ing.".to_string()), // Ingeniero
        honorific: Some("Sr.".to_string()),
        given_names: vec!["José".to_string(), "María".to_string()],
        middle_names: vec![],
        family_names: vec!["García".to_string()],
        maternal_family_name: Some("Rodríguez".to_string()),
        generational_suffix: None,
        professional_suffix: None,
        preferred_name: Some("José María".to_string()),
        name_order: cim_domain_person::NameOrder::GivenFirst,
        cultural_context: Some("Spanish".to_string()),
    };
    
    println!("Spanish Name: {}", spanish_name.formatted_name());
    
    // Japanese naming convention
    let japanese_name = NameComponent {
        title: None,
        honorific: None, // Would use -san, -sama, etc. in context
        given_names: vec!["Hiroshi".to_string()],
        middle_names: vec![],
        family_names: vec!["Tanaka".to_string()],
        maternal_family_name: None,
        generational_suffix: None,
        professional_suffix: None,
        preferred_name: None,
        name_order: cim_domain_person::NameOrder::FamilyFirst,
        cultural_context: Some("Japanese".to_string()),
    };
    
    println!("Japanese Name: {}", japanese_name.formatted_name());
    
    // Complex Western name with multiple titles
    let complex_name = NameComponent {
        title: Some("Prof. Dr.".to_string()),
        honorific: Some("Sir".to_string()),
        given_names: vec!["William".to_string()],
        middle_names: vec!["Alexander".to_string(), "James".to_string(), "Theodore".to_string()],
        family_names: vec!["Montgomery-Smith".to_string()],
        maternal_family_name: None,
        generational_suffix: Some("III".to_string()),
        professional_suffix: Some("MD, PhD, FRCS".to_string()),
        preferred_name: Some("Bill".to_string()),
        name_order: cim_domain_person::NameOrder::GivenFirst,
        cultural_context: Some("British".to_string()),
    };
    
    println!("Complex Name: {}", complex_name.formatted_name());
    println!("But prefers to be called: {}", complex_name.display_name());
} 