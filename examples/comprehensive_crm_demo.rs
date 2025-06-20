//! Comprehensive CRM Demo for the Enhanced Person Domain
//!
//! This demo showcases the full capabilities of the Person domain
//! including customer segmentation, behavioral tracking, and multi-cultural support.

use cim_domain_person::{
    aggregate::PersonId,
    services::{
        PersonCompositionService,
        views::{CustomerView, EmployeeView, PartnerView},
    },
    value_objects::{
        // Name components
        NameComponent, NameOrder,
        // Behavioral components
        PreferencesComponent, CommunicationPreferences, ContactChannel,
        ContactTimePreference, FrequencyPreference,
        BehavioralComponent, PurchaseBehavior, EngagementPatterns,
        InteractionSummary, PredictiveScores,
        SegmentationComponent, CustomerSegment, LifecycleStage, ValueTier,
        // Social components
        RelationshipComponent, Relationship, RelationshipType, RelationshipStatus,
        SocialMediaComponent, SocialMediaProfile, SocialPlatform, PrivacySetting, SocialMetrics,
    },
};

use chrono::{Utc, NaiveDate};
use std::collections::HashMap;
use uuid::Uuid;

fn main() {
    println!("=== Comprehensive CRM Demo ===\n");

    // Demo 1: Multi-cultural Customer with Complex Name
    demo_multicultural_customer();
    println!("\n{}\n", "=".repeat(50));

    // Demo 2: High-Value Customer with Predictive Analytics
    demo_vip_customer();
    println!("\n{}\n", "=".repeat(50));

    // Demo 3: Employee to Partner Transition
    demo_employee_partner_transition();
    println!("\n{}\n", "=".repeat(50));

    // Demo 4: Social Media Influencer
    demo_social_influencer();

    println!("\n✅ All demos completed successfully!");
}

fn demo_multicultural_customer() {
    println!("📌 Demo 1: Multi-cultural Customer Support\n");

    let service = PersonCompositionService::new();
    let mut customer = service.create_customer(
        PersonId::new(),
        "María García López",
        Some("maria@empresa.es"),
        Some("+34-555-123456"),
    );

    // Add Spanish naming convention
    let spanish_name = NameComponent {
        title: Some("Dra.".to_string()),
        honorific: Some("Sra.".to_string()),
        given_names: vec!["María".to_string(), "Isabel".to_string()],
        middle_names: vec![],
        family_names: vec!["García".to_string()],
        maternal_family_name: Some("López".to_string()),
        generational_suffix: None,
        professional_suffix: Some("PhD".to_string()),
        preferred_name: Some("Maribel".to_string()),
        name_order: NameOrder::GivenFirst,
        cultural_context: Some("Spanish".to_string()),
    };

    customer.add_component(
        spanish_name,
        "CRM",
        Some("Cultural name support".to_string())
    ).unwrap();

    // Add language preferences
    let preferences = PreferencesComponent {
        communication: CommunicationPreferences {
            preferred_channel: ContactChannel::Email,
            channel_settings: HashMap::new(),
            contact_time_preference: ContactTimePreference {
                preferred_days: vec![],
                preferred_hours: Some((10, 18)),
                timezone: "Europe/Madrid".to_string(),
            },
            preferred_language: "es-ES".to_string(),
            frequency_preference: FrequencyPreference::Weekly,
        },
        product_preferences: vec![],
        content_preferences: cim_domain_person::value_objects::ContentPreferences {
            content_types: vec![],
            topics: vec!["Tecnología".to_string(), "Innovación".to_string()],
            format_preference: cim_domain_person::value_objects::ContentFormat::Text,
            complexity_preference: cim_domain_person::value_objects::ComplexityLevel::Advanced,
        },
        privacy_preferences: cim_domain_person::value_objects::PrivacyPreferences {
            data_sharing_allowed: true,
            analytics_allowed: true,
            personalization_allowed: true,
            third_party_sharing_allowed: false,
            regulatory_preferences: {
                let mut prefs = HashMap::new();
                prefs.insert("GDPR".to_string(), true);
                prefs
            },
        },
    };

    customer.add_component(preferences, "CRM", Some("Spanish customer preferences".to_string())).unwrap();

    let view = CustomerView::from_person(&customer);
    println!("Customer: {}", view.name);
    println!("Preferred name: Maribel");
    println!("Full Spanish name: Dra. Sra. María Isabel García López PhD");
    println!("Language: es-ES");
    println!("Timezone: Europe/Madrid");
    println!("GDPR compliant: ✓");
}

fn demo_vip_customer() {
    println!("📌 Demo 2: VIP Customer with Predictive Analytics\n");

    let service = PersonCompositionService::new();
    let mut customer = service.create_customer(
        PersonId::new(),
        "James Chen",
        Some("james@techcorp.com"),
        Some("+1-555-9876"),
    );

    // Add high-value behavioral data
    let behavioral = BehavioralComponent {
        purchase_behavior: PurchaseBehavior {
            average_order_value: Some(2500.0),
            purchase_frequency: Some(24.0), // Twice monthly
            payment_methods: vec!["Premium Card".to_string()],
            seasonal_patterns: {
                let mut patterns = HashMap::new();
                patterns.insert("Q4".to_string(), 1.5); // 50% increase in Q4
                patterns
            },
            category_distribution: {
                let mut dist = HashMap::new();
                dist.insert("Electronics".to_string(), 0.6);
                dist.insert("Software".to_string(), 0.4);
                dist
            },
            typical_price_range: Some((1000.0, 5000.0)),
            discount_sensitivity: Some(0.1), // Low sensitivity
        },
        engagement_patterns: EngagementPatterns {
            email_open_rate: Some(0.95),
            click_through_rate: Some(0.80),
            visit_frequency: Some(15.0), // 15 visits/month
            avg_session_duration: Some(600), // 10 minutes
            device_usage: {
                let mut usage = HashMap::new();
                usage.insert("Desktop".to_string(), 0.7);
                usage.insert("Mobile".to_string(), 0.3);
                usage
            },
            active_hours: {
                let mut hours = vec![0.0; 24];
                // Peak activity 9-11 AM and 2-4 PM
                hours[9] = 0.9;
                hours[10] = 1.0;
                hours[11] = 0.8;
                hours[14] = 0.8;
                hours[15] = 0.9;
                hours[16] = 0.7;
                hours
            },
        },
        interaction_summary: InteractionSummary {
            total_interactions: 500,
            last_interaction: Some(Utc::now()),
            support_tickets: 2,
            avg_satisfaction: Some(4.9),
            channels_used: vec![
                ContactChannel::Email,
                ContactChannel::Phone,
                ContactChannel::InApp,
            ],
        },
        predictive_scores: PredictiveScores {
            churn_risk: Some(0.02), // 2% - very low
            predicted_ltv: Some(75000.0),
            purchase_probability: Some(0.98),
            upsell_potential: Some(0.85),
            referral_likelihood: Some(0.92),
        },
    };

    customer.add_component(behavioral, "Analytics", Some("VIP behavioral profile".to_string())).unwrap();

    // Add VIP segmentation
    let segmentation = SegmentationComponent {
        primary_segment: CustomerSegment::VIPCustomer,
        secondary_segments: vec![
            CustomerSegment::LoyalCustomer,
            CustomerSegment::Custom("Early Adopter".to_string()),
        ],
        lifecycle_stage: LifecycleStage::Advocacy,
        value_tier: ValueTier::Platinum,
        persona: Some("Tech Executive".to_string()),
        custom_segments: {
            let mut segments = HashMap::new();
            segments.insert("Industry".to_string(), "Technology".to_string());
            segments.insert("Company Size".to_string(), "Enterprise".to_string());
            segments
        },
    };

    customer.add_component(segmentation, "Analytics", Some("VIP segmentation".to_string())).unwrap();

    let view = CustomerView::from_person(&customer);
    println!("VIP Customer: {}", view.name);
    println!("Segment: {:?}", view.segment);
    println!("Lifetime Value: ${:?}", view.lifetime_value.unwrap_or(0.0));
    println!("Churn Risk: {}%", (0.02 * 100.0));
    println!("Purchase Probability: {}%", (0.98 * 100.0));
    println!("Engagement Score: {}%", (view.engagement_score.unwrap_or(0.0) * 100.0));
    println!("Preferred Device: Desktop (70%)");
    println!("Peak Activity: 10 AM and 3 PM");
}

fn demo_employee_partner_transition() {
    println!("📌 Demo 3: Employee to Partner Transition\n");

    let service = PersonCompositionService::new();
    let person_id = PersonId::new();

    // Start as employee
    let mut person = service.create_employee(
        person_id,
        "Sarah Williams",
        "Engineering",
        Some("Senior Developer"),
        Some(PersonId::new()),
    );

    println!("Initial: Employee");
    let employee_view = EmployeeView::from_person(&person);
    println!("  Name: {}", employee_view.name);
    println!("  Department: {:?}", employee_view.department);
    println!("  Position: {:?}", employee_view.position);

    // Transition to partner
    println!("\nTransition: Employee → Partner");

    // Update employment status instead of adding new component
    // In a real system, this would be done through commands/events
    // For demo purposes, we'll create a new person as partner
    let partner_person = service.create_partner(
        PersonId::new(),
        "Sarah Williams",
        "TechCorp Partners",
        Some("Technical Partner"),
    );

    // Add business relationships
    let relationships = RelationshipComponent {
        relationships: vec![
            Relationship {
                person_id: Uuid::new_v4(),
                relationship_type: RelationshipType::BusinessPartner,
                reciprocal_type: RelationshipType::BusinessPartner,
                start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                end_date: None,
                status: RelationshipStatus::Active,
                notes: Some("Strategic technology partner".to_string()),
            },
        ],
    };

    let mut partner = partner_person;
    partner.add_component(relationships, "Partnerships", Some("Partner network".to_string())).unwrap();

    let partner_view = PartnerView::from_person(&partner);
    println!("\nFinal: Partner");
    println!("  Name: {}", partner_view.name);
    println!("  Partnership Type: Technical Partner");
    println!("  Status: Active Strategic Partner");
    println!("  Note: In production, this would be the same person entity with updated components");
}

fn demo_social_influencer() {
    println!("📌 Demo 4: Social Media Influencer Customer\n");

    let service = PersonCompositionService::new();
    let mut customer = service.create_customer(
        PersonId::new(),
        "Alex Rivera",
        Some("alex@influencer.com"),
        Some("+1-555-2468"),
    );

    // Add social media presence
    let social = SocialMediaComponent {
        profiles: vec![
            SocialMediaProfile {
                platform: SocialPlatform::Twitter,
                username: "alextech".to_string(),
                profile_url: Some("https://twitter.com/alextech".to_string()),
                verified: true,
                privacy: PrivacySetting::Public,
                last_active: Some(Utc::now()),
                follower_count: Some(50000),
            },
            SocialMediaProfile {
                platform: SocialPlatform::LinkedIn,
                username: "alexrivera".to_string(),
                profile_url: Some("https://linkedin.com/in/alexrivera".to_string()),
                verified: true,
                privacy: PrivacySetting::Professional,
                last_active: Some(Utc::now()),
                follower_count: Some(15000),
            },
            SocialMediaProfile {
                platform: SocialPlatform::Instagram,
                username: "alex.tech.life".to_string(),
                profile_url: Some("https://instagram.com/alex.tech.life".to_string()),
                verified: false,
                privacy: PrivacySetting::Public,
                last_active: Some(Utc::now()),
                follower_count: Some(25000),
            },
        ],
        metrics: Some(SocialMetrics {
            total_followers: 90000,
            engagement_rate: Some(0.065), // 6.5% - excellent
            primary_platform: Some(SocialPlatform::Twitter),
            influence_score: Some(92.0),
        }),
    };

    customer.add_component(social, "Marketing", Some("Influencer profile".to_string())).unwrap();

    // Add influencer behavioral patterns
    let behavioral = BehavioralComponent {
        purchase_behavior: PurchaseBehavior {
            average_order_value: Some(800.0),
            purchase_frequency: Some(18.0),
            payment_methods: vec!["Influencer Account".to_string()],
            seasonal_patterns: HashMap::new(),
            category_distribution: {
                let mut dist = HashMap::new();
                dist.insert("Tech Gadgets".to_string(), 0.7);
                dist.insert("Content Creation Tools".to_string(), 0.3);
                dist
            },
            typical_price_range: Some((200.0, 2000.0)),
            discount_sensitivity: Some(0.3),
        },
        engagement_patterns: EngagementPatterns {
            email_open_rate: Some(0.88),
            click_through_rate: Some(0.65),
            visit_frequency: Some(20.0),
            avg_session_duration: Some(480),
            device_usage: {
                let mut usage = HashMap::new();
                usage.insert("Mobile".to_string(), 0.6);
                usage.insert("Desktop".to_string(), 0.4);
                usage
            },
            active_hours: vec![0.5; 24], // Active throughout the day
        },
        interaction_summary: InteractionSummary {
            total_interactions: 300,
            last_interaction: Some(Utc::now()),
            support_tickets: 5,
            avg_satisfaction: Some(4.7),
            channels_used: vec![ContactChannel::InApp, ContactChannel::Email],
        },
        predictive_scores: PredictiveScores {
            churn_risk: Some(0.08),
            predicted_ltv: Some(25000.0),
            purchase_probability: Some(0.92),
            upsell_potential: Some(0.78),
            referral_likelihood: Some(0.95), // Very high
        },
    };

    customer.add_component(behavioral, "Analytics", Some("Influencer behavior".to_string())).unwrap();

    // Special influencer segment
    let segmentation = SegmentationComponent {
        primary_segment: CustomerSegment::Custom("Influencer".to_string()),
        secondary_segments: vec![CustomerSegment::ActiveCustomer],
        lifecycle_stage: LifecycleStage::Advocacy,
        value_tier: ValueTier::Gold,
        persona: Some("Tech Influencer".to_string()),
        custom_segments: {
            let mut segments = HashMap::new();
            segments.insert("Influence Level".to_string(), "Macro".to_string());
            segments.insert("Content Focus".to_string(), "Technology".to_string());
            segments
        },
    };

    customer.add_component(segmentation, "Marketing", Some("Influencer segmentation".to_string())).unwrap();

    let view = CustomerView::from_person(&customer);
    println!("Influencer: {}", view.name);
    println!("Total Followers: 90,000");
    println!("Engagement Rate: 6.5% (Excellent)");
    println!("Influence Score: 92/100");
    println!("Primary Platform: Twitter (50K followers, verified ✓)");
    println!("Referral Likelihood: 95%");
    println!("Customer Value: Gold Tier");
    println!("Special Status: Macro Influencer - Technology");
} 