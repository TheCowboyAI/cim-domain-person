//! Working demonstration of the Person domain functionality

use cim_domain_person::{
    aggregate::{Person, PersonId},
    services::PersonCompositionService,
    value_objects::*,
};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Person Domain Working Demo ===\n");

    // Create service
    let service = PersonCompositionService::new();
    let person_id = PersonId::from_uuid(Uuid::new_v4());

    // Create a customer
    println!("Creating customer...");
    let mut customer = service.create_customer(
        person_id,
        "Alice Johnson",
        Some("alice@example.com"),
        Some("+1-555-0123"),
    );

    println!("✓ Customer created: Alice Johnson");
    println!("  Email: alice@example.com");
    println!("  Phone: +1-555-0123");
    println!("  Components: {}", customer.component_count());

    // Add preferences
    println!("\nAdding preferences...");
    let preferences = PreferencesComponent {
        communication: CommunicationPreferences {
            preferred_channel: ContactChannel::Email,
            channel_settings: Default::default(),
            contact_time_preference: ContactTimePreference {
                preferred_days: vec![],
                preferred_hours: Some((9, 17)),
                timezone: "America/New_York".to_string(),
            },
            preferred_language: "en-US".to_string(),
            frequency_preference: FrequencyPreference::Weekly,
        },
        product_preferences: vec![
            ProductPreference {
                category: "Electronics".to_string(),
                preference_level: 8,
                preferred_brands: vec!["Apple".to_string(), "Samsung".to_string()],
                price_sensitivity: 5,
                quality_preference: 9,
            },
        ],
        content_preferences: ContentPreferences {
            content_types: vec![ContentType::Educational, ContentType::News],
            topics: vec!["Technology".to_string(), "Science".to_string()],
            format_preference: ContentFormat::Video,
            complexity_preference: ComplexityLevel::Intermediate,
        },
        privacy_preferences: PrivacyPreferences {
            data_sharing_allowed: false,
            analytics_allowed: true,
            personalization_allowed: true,
            third_party_sharing_allowed: false,
            regulatory_preferences: Default::default(),
        },
    };

    customer.add_component(
        preferences,
        "System",
        Some("Initial preferences setup".to_string()),
    )?;

    println!("✓ Preferences added");
    println!("  Preferred channel: Email");
    println!("  Language: en-US");
    println!("  Frequency: Weekly");

    // Add behavioral data
    println!("\nAdding behavioral data...");
    let behavioral = BehavioralComponent {
        purchase_behavior: PurchaseBehavior {
            average_order_value: Some(150.0),
            purchase_frequency: Some(6.0), // 6 times per year
            payment_methods: vec!["Credit Card".to_string()],
            seasonal_patterns: Default::default(),
            category_distribution: Default::default(),
            typical_price_range: Some((50.0, 300.0)),
            discount_sensitivity: Some(0.7),
        },
        engagement_patterns: EngagementPatterns {
            email_open_rate: Some(0.65),
            click_through_rate: Some(0.15),
            visit_frequency: Some(2.5),
            avg_session_duration: Some(180),
            device_usage: Default::default(),
            active_hours: vec![0.1; 24],
        },
        interaction_summary: InteractionSummary {
            total_interactions: 25,
            last_interaction: Some(chrono::Utc::now()),
            support_tickets: 1,
            avg_satisfaction: Some(4.2),
            channels_used: vec![ContactChannel::Email, ContactChannel::Phone],
        },
        predictive_scores: PredictiveScores {
            churn_risk: Some(0.15),
            predicted_ltv: Some(3000.0),
            purchase_probability: Some(0.8),
            upsell_potential: Some(0.6),
            referral_likelihood: Some(0.7),
        },
    };

    customer.add_component(
        behavioral,
        "Analytics",
        Some("Q1 behavioral analysis".to_string()),
    )?;

    println!("✓ Behavioral data added");
    println!("  Average order value: $150");
    println!("  Purchase frequency: 6/year");
    println!("  Churn risk: 15%");
    println!("  Predicted LTV: $3,000");

    // Add segmentation
    println!("\nAdding segmentation...");
    let segmentation = SegmentationComponent {
        primary_segment: CustomerSegment::ActiveCustomer,
        secondary_segments: vec![CustomerSegment::Lead],
        lifecycle_stage: LifecycleStage::Purchase,
        value_tier: ValueTier::Silver,
        persona: Some("Tech Enthusiast".to_string()),
        custom_segments: Default::default(),
    };

    customer.add_component(
        segmentation,
        "Marketing",
        Some("Monthly segmentation update".to_string()),
    )?;

    println!("✓ Segmentation added");
    println!("  Primary segment: Active Customer");
    println!("  Lifecycle stage: Purchase");
    println!("  Value tier: Silver");
    println!("  Persona: Tech Enthusiast");

    // Final summary
    println!("\n=== Final Customer Profile ===");
    println!("Name: Alice Johnson");
    println!("Total components: {}", customer.component_count());
    println!("Components:");
    for component_type in customer.component_types() {
        println!("  - {}", component_type);
    }

    println!("\n✅ Demo completed successfully!");

    Ok(())
} 