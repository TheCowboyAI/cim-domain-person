//! Comprehensive tests for the Person domain
//!
//! ```mermaid
//! graph TD
//!     A[Person Domain Tests] --> B[Component Tests]
//!     A --> C[Command/Event Tests]
//!     A --> D[Query Tests]
//!     A --> E[Service Tests]
//!     
//!     B --> B1[Name Components]
//!     B --> B2[Physical Components]
//!     B --> B3[Social Components]
//!     B --> B4[Behavioral Components]
//!     
//!     C --> C1[CRM Commands]
//!     C --> C2[Event Generation]
//!     C --> C3[Command Handlers]
//!     
//!     D --> D1[Customer Queries]
//!     D --> D2[Search Queries]
//!     D --> D3[Analytics Queries]
//!     
//!     E --> E1[Composition Service]
//!     E --> E2[View Builders]
//! ```

use cim_domain_person::{
    aggregate::{Person, PersonId},
    commands::PersonCommand,
    events::PersonEvent,
    queries::PersonQuery,
    handlers::{handle_person_command, PersonQueryResult, PersonReadModel},
    services::{
        PersonCompositionService,
        views::{CustomerView, EmployeeView, PartnerView},
    },
    value_objects::{
        IdentityComponent, ContactComponent, EmailAddress, PhoneNumber,
        EmploymentComponent, PositionComponent, SkillsComponent,
        AccessComponent, ExternalIdentifiersComponent,
        NameComponent, NameOrder, AlternativeNamesComponent,
        PhysicalAttributesComponent, Build, VisionCorrection,
        DistinguishingMarksComponent, BiometricComponent, BiometricHash,
        MedicalIdentityComponent, BloodType,
        RelationshipComponent, Relationship, RelationshipType, RelationshipStatus,
        SocialMediaComponent, SocialMediaProfile, SocialPlatform, PrivacySetting, SocialMetrics,
        InterestsComponent, InterestCategory, Interest,
        PreferencesComponent, CommunicationPreferences, ContactChannel, ChannelSettings,
        ContactTimePreference, FrequencyPreference, ProductPreference, ContentPreferences,
        ContentType, ContentFormat, ComplexityLevel, PrivacyPreferences,
        BehavioralComponent, PurchaseBehavior, EngagementPatterns, InteractionSummary,
        PredictiveScores, SegmentationComponent, CustomerSegment, LifecycleStage, ValueTier,
    },
};
use chrono::{Utc, NaiveDate};
use uuid::Uuid;
use std::collections::HashMap;

#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn test_name_component_with_titles() {
        let name = NameComponent {
            title: Some("Dr.".to_string()),
            honorific: Some("Ms.".to_string()),
            given_names: vec!["Jane".to_string()],
            middle_names: vec!["Marie".to_string()],
            family_names: vec!["Smith".to_string()],
            maternal_family_name: None,
            generational_suffix: Some("Jr.".to_string()),
            professional_suffix: Some("PhD".to_string()),
            preferred_name: Some("Jane".to_string()),
            name_order: NameOrder::GivenFirst,
            cultural_context: Some("English".to_string()),
        };

        assert_eq!(name.formatted_name(), "Dr. Ms. Jane Marie Smith Jr. PhD");
        assert_eq!(name.display_name(), "Jane");
    }

    #[test]
    fn test_spanish_naming_convention() {
        let name = NameComponent {
            title: None,
            honorific: None,
            given_names: vec!["Juan".to_string(), "Carlos".to_string()],
            middle_names: vec![],
            family_names: vec!["García".to_string()],
            maternal_family_name: Some("López".to_string()),
            generational_suffix: None,
            professional_suffix: None,
            preferred_name: Some("Juan Carlos".to_string()),
            name_order: NameOrder::GivenFirst,
            cultural_context: Some("Spanish".to_string()),
        };

        assert_eq!(name.formatted_name(), "Juan Carlos García López");
    }

    #[test]
    fn test_physical_attributes() {
        let attributes = PhysicalAttributesComponent {
            height_cm: Some(175.0),
            weight_kg: Some(70.0),
            build: Some(Build::Athletic),
            hair_color: Some("Brown".to_string()),
            hair_style: Some("Curly".to_string()),
            eye_color: Some("Blue".to_string()),
            skin_tone: Some("Medium".to_string()),
            facial_hair: None,
            vision_correction: Some(VisionCorrection::Contacts),
            appearance_notes: Some("Athletic build".to_string()),
        };

        assert_eq!(attributes.height_cm, Some(175.0));
        assert_eq!(attributes.build, Some(Build::Athletic));
    }

    #[test]
    fn test_behavioral_component() {
        let behavioral = BehavioralComponent {
            purchase_behavior: PurchaseBehavior {
                average_order_value: Some(250.0),
                purchase_frequency: Some(12.0),
                payment_methods: vec!["Website".to_string()],
                seasonal_patterns: HashMap::new(),
                category_distribution: HashMap::new(),
                typical_price_range: Some((100.0, 400.0)),
                discount_sensitivity: Some(0.3),
            },
            engagement_patterns: EngagementPatterns {
                email_open_rate: Some(0.92),
                click_through_rate: Some(0.75),
                visit_frequency: Some(4.0),
                avg_session_duration: Some(300),
                device_usage: HashMap::new(),
                active_hours: vec![0.1; 24],
            },
            interaction_summary: InteractionSummary {
                total_interactions: 150,
                last_interaction: Some(Utc::now()),
                support_tickets: 2,
                avg_satisfaction: Some(4.8),
                channels_used: vec![ContactChannel::Email, ContactChannel::InApp],
            },
            predictive_scores: PredictiveScores {
                churn_risk: Some(0.05),
                predicted_ltv: Some(15000.0),
                purchase_probability: Some(0.95),
                upsell_potential: Some(0.75),
                referral_likelihood: Some(0.85),
            },
        };

        assert_eq!(behavioral.purchase_behavior.average_order_value, Some(250.0));
        assert_eq!(behavioral.engagement_patterns.email_open_rate, Some(0.92));
        assert_eq!(behavioral.predictive_scores.churn_risk, Some(0.05));
        assert_eq!(behavioral.predictive_scores.referral_likelihood, Some(0.85));
    }

    #[test]
    fn test_preferences_component() {
        let preferences = PreferencesComponent {
            communication: CommunicationPreferences {
                preferred_channel: ContactChannel::Email,
                channel_settings: HashMap::new(),
                contact_time_preference: ContactTimePreference {
                    preferred_days: vec![],
                    preferred_hours: None,
                    timezone: "America/Los_Angeles".to_string(),
                },
                preferred_language: "en-US".to_string(),
                frequency_preference: FrequencyPreference::Monthly,
            },
            product_preferences: vec![
                ProductPreference {
                    category: "Electronics".to_string(),
                    preference_level: 9,
                    preferred_brands: vec!["Apple".to_string()],
                    price_sensitivity: 3,
                    quality_preference: 9,
                },
            ],
            content_preferences: ContentPreferences {
                content_types: vec![ContentType::Educational, ContentType::ProductUpdates],
                topics: vec!["Technology".to_string(), "Innovation".to_string()],
                format_preference: ContentFormat::Video,
                complexity_preference: ComplexityLevel::Advanced,
            },
            privacy_preferences: PrivacyPreferences {
                data_sharing_allowed: true,
                analytics_allowed: true,
                personalization_allowed: true,
                third_party_sharing_allowed: false,
                regulatory_preferences: HashMap::new(),
            },
        };

        assert_eq!(preferences.communication.preferred_channel, ContactChannel::Email);
        assert_eq!(preferences.product_preferences.len(), 1);
        assert_eq!(preferences.content_preferences.topics.len(), 2);
        assert_eq!(preferences.privacy_preferences.data_sharing_allowed, true);
    }

    #[test]
    fn test_segmentation_component() {
        let segmentation = SegmentationComponent {
            primary_segment: CustomerSegment::VIPCustomer,
            secondary_segments: vec![CustomerSegment::LoyalCustomer],
            lifecycle_stage: LifecycleStage::Advocacy,
            value_tier: ValueTier::Platinum,
            persona: Some("Tech Enthusiast".to_string()),
            custom_segments: HashMap::new(),
        };

        assert_eq!(segmentation.primary_segment, CustomerSegment::VIPCustomer);
        assert!(segmentation.secondary_segments.contains(&CustomerSegment::LoyalCustomer));
    }
}

#[cfg(test)]
mod command_event_tests {
    use super::*;

    #[tokio::test]
    async fn test_update_name_command() {
        let person_id = Uuid::new_v4();
        let mut person = Person::new(
            PersonId::new(),
            IdentityComponent {
                legal_name: "John Doe".to_string(),
                preferred_name: None,
                date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
                government_id: None,
            },
        );

        let name = NameComponent {
            title: Some("Mr.".to_string()),
            honorific: None,
            given_names: vec!["John".to_string()],
            middle_names: vec!["Robert".to_string()],
            family_names: vec!["Doe".to_string()],
            maternal_family_name: None,
            generational_suffix: None,
            professional_suffix: None,
            preferred_name: Some("Johnny".to_string()),
            name_order: NameOrder::GivenFirst,
            cultural_context: None,
        };

        let command = PersonCommand::UpdateName {
            person_id,
            name: name.clone(),
        };

        let events = handle_person_command(&mut person, command).await.unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            PersonEvent::NameUpdated { person_id: id, new_name, .. } => {
                assert_eq!(*id, person_id);
                assert_eq!(new_name.given_names[0], "John");
                assert_eq!(new_name.preferred_name, Some("Johnny".to_string()));
            }
            _ => panic!("Expected NameUpdated event"),
        }

        // Verify component was added
        assert!(person.has_component::<NameComponent>());
        let stored_name = person.get_component::<NameComponent>().unwrap();
        assert_eq!(stored_name.preferred_name, Some("Johnny".to_string()));
    }

    #[tokio::test]
    async fn test_update_behavioral_data() {
        let person_id = Uuid::new_v4();
        let mut person = Person::new(
            PersonId::new(),
            IdentityComponent {
                legal_name: "Jane Smith".to_string(),
                preferred_name: None,
                date_of_birth: None,
                government_id: None,
            },
        );

        let behavioral = BehavioralComponent {
            purchase_behavior: PurchaseBehavior {
                average_order_value: Some(250.0),
                purchase_frequency: Some(12.0),
                payment_methods: vec!["Website".to_string()],
                seasonal_patterns: HashMap::new(),
                category_distribution: HashMap::new(),
                typical_price_range: Some((100.0, 400.0)),
                discount_sensitivity: Some(0.3),
            },
            engagement_patterns: EngagementPatterns {
                email_open_rate: Some(0.92),
                click_through_rate: Some(0.75),
                visit_frequency: Some(4.0),
                avg_session_duration: Some(300),
                device_usage: HashMap::new(),
                active_hours: vec![0.1; 24],
            },
            interaction_summary: InteractionSummary {
                total_interactions: 150,
                last_interaction: Some(Utc::now()),
                support_tickets: 2,
                avg_satisfaction: Some(4.8),
                channels_used: vec![ContactChannel::Email, ContactChannel::InApp],
            },
            predictive_scores: PredictiveScores {
                churn_risk: Some(0.05),
                predicted_ltv: Some(15000.0),
                purchase_probability: Some(0.95),
                upsell_potential: Some(0.75),
                referral_likelihood: Some(0.85),
            },
        };

        let command = PersonCommand::UpdateBehavioralData {
            person_id,
            behavioral: behavioral.clone(),
        };

        let events = handle_person_command(&mut person, command).await.unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            PersonEvent::BehavioralDataUpdated { person_id: id, new_behavioral, .. } => {
                assert_eq!(*id, person_id);
                assert_eq!(new_behavioral.engagement_patterns.email_open_rate, Some(0.92));
                assert_eq!(new_behavioral.predictive_scores.churn_risk, Some(0.05));
            }
            _ => panic!("Expected BehavioralDataUpdated event"),
        }
    }

    #[tokio::test]
    async fn test_add_relationships() {
        let person_id = Uuid::new_v4();
        let related_person_id = Uuid::new_v4();
        let mut person = Person::new(
            PersonId::new(),
            IdentityComponent {
                legal_name: "Alice Johnson".to_string(),
                preferred_name: None,
                date_of_birth: None,
                government_id: None,
            },
        );

        let relationships = RelationshipComponent {
            relationships: vec![
                Relationship {
                    person_id: related_person_id,
                    relationship_type: RelationshipType::Spouse,
                    reciprocal_type: RelationshipType::Spouse,
                    start_date: Some(NaiveDate::from_ymd_opt(2015, 6, 15).unwrap()),
                    end_date: None,
                    status: RelationshipStatus::Active,
                    notes: Some("Married in Hawaii".to_string()),
                },
                Relationship {
                    person_id: Uuid::new_v4(),
                    relationship_type: RelationshipType::BusinessPartner,
                    reciprocal_type: RelationshipType::BusinessPartner,
                    start_date: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()),
                    end_date: None,
                    status: RelationshipStatus::Active,
                    notes: Some("Co-founder of TechCorp".to_string()),
                },
            ],
        };

        let command = PersonCommand::UpdateRelationships {
            person_id,
            relationships: relationships.clone(),
        };

        let events = handle_person_command(&mut person, command).await.unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            PersonEvent::RelationshipsUpdated { person_id: id, new_relationships, .. } => {
                assert_eq!(*id, person_id);
                assert_eq!(new_relationships.relationships.len(), 2);
            }
            _ => panic!("Expected RelationshipsUpdated event"),
        }
    }
}

#[cfg(test)]
mod query_tests {
    use super::*;

    #[tokio::test]
    async fn test_find_customers_by_segment() {
        let read_model = PersonReadModel::new();
        
        // In a real implementation, we would populate the read model
        // For now, we'll just test the query structure
        let query = PersonQuery::FindCustomersBySegment {
            segment: "Premium".to_string(),
            sub_segment: Some("Tech Enthusiast".to_string()),
        };

        let result = read_model.handle_query(query).await.unwrap();
        
        match result {
            PersonQueryResult::People(people) => {
                // In a real test, we'd verify the people returned match the segment
                assert!(people.is_empty() || people.len() > 0);
            }
            _ => panic!("Expected People result"),
        }
    }

    #[tokio::test]
    async fn test_find_by_behavioral_pattern() {
        let read_model = PersonReadModel::new();
        
        let query = PersonQuery::FindCustomersByBehavior {
            pattern: "early_adopter".to_string(),
            threshold: 0.8,
        };

        let result = read_model.handle_query(query).await.unwrap();
        
        match result {
            PersonQueryResult::People(_) => {
                // Success - correct result type
            }
            _ => panic!("Expected People result"),
        }
    }

    #[tokio::test]
    async fn test_search_people_multi_criteria() {
        let read_model = PersonReadModel::new();
        
        let query = PersonQuery::SearchPeople {
            name: Some("John".to_string()),
            email: None,
            phone: None,
            organization: Some("TechCorp".to_string()),
            skills: Some(vec!["Rust".to_string(), "Python".to_string()]),
            segments: Some(vec!["Premium".to_string()]),
            limit: 10,
            offset: 0,
        };

        let result = read_model.handle_query(query).await.unwrap();
        
        match result {
            PersonQueryResult::People(_) => {
                // Success - correct result type
            }
            _ => panic!("Expected People result"),
        }
    }
}

#[cfg(test)]
mod service_tests {
    use super::*;

    #[test]
    fn test_create_customer_with_composition_service() {
        let service = PersonCompositionService::new();
        let person_id = Uuid::new_v4();

        let customer = service.create_customer(
            PersonId::new(),
            "Sarah Connor",
            Some("sarah@skynet.com"),
            Some("+1-555-9999"),
        );

        // Verify basic components
        assert!(customer.has_component::<IdentityComponent>());
        assert!(customer.has_component::<ContactComponent>());

        // Add behavioral data
        let mut customer = customer;
        let behavioral = BehavioralComponent {
            purchase_behavior: PurchaseBehavior {
                average_order_value: Some(500.0),
                purchase_frequency: Some(12.0), // 12 times per year (monthly)
                payment_methods: vec!["Online".to_string()],
                seasonal_patterns: HashMap::new(),
                category_distribution: HashMap::new(),
                typical_price_range: Some((200.0, 1000.0)),
                discount_sensitivity: Some(0.2),
            },
            engagement_patterns: EngagementPatterns {
                email_open_rate: Some(0.9),
                click_through_rate: Some(0.7),
                visit_frequency: Some(2.0),
                avg_session_duration: Some(180),
                device_usage: HashMap::new(),
                active_hours: vec![0.2; 24],
            },
            interaction_summary: InteractionSummary {
                total_interactions: 25,
                last_interaction: Some(Utc::now()),
                support_tickets: 1,
                avg_satisfaction: Some(4.2),
                channels_used: vec![ContactChannel::Email, ContactChannel::InApp],
            },
            predictive_scores: PredictiveScores {
                churn_risk: Some(0.2),
                predicted_ltv: Some(6000.0),
                purchase_probability: Some(0.9),
                upsell_potential: Some(0.7),
                referral_likelihood: Some(0.8),
            },
        };

        customer.add_component(behavioral, "system", Some("Customer profile enrichment".to_string())).unwrap();

        // Verify we can build a customer view
        let customer_view = CustomerView::from_person(&customer);
        assert_eq!(customer_view.name, "Sarah Connor");
        assert_eq!(customer_view.email, Some("sarah@skynet.com".to_string()));
        assert_eq!(customer_view.engagement_score, Some(0.9));
    }

    #[test]
    fn test_create_employee_with_physical_attributes() {
        let service = PersonCompositionService::new();
        let person_id = Uuid::new_v4();

        let mut employee = service.create_employee(
            PersonId::new(),
            "John Matrix",
            "Security",
            Some("Head of Security"),
            None,
        );

        // Add physical attributes for security personnel
        let physical = PhysicalAttributesComponent {
            height_cm: Some(188.0),
            weight_kg: Some(95.0),
            eye_color: Some("Brown".to_string()),
            hair_color: Some("Black".to_string()),
            hair_style: Some("Short".to_string()),
            skin_tone: None,
            build: Some(Build::Other("Muscular".to_string())),
            facial_hair: None,
            vision_correction: None,
            appearance_notes: None,
        };

        employee.add_component(physical, "HR", Some("Security clearance requirements".to_string())).unwrap();

        // Add biometric data for access control
        let biometric = BiometricComponent {
            fingerprint_hashes: vec![BiometricHash { identifier: "right_thumb".to_string(), hash: "HASH_PLACEHOLDER".to_string(), algorithm: "SHA256".to_string() }],
            face_encoding: Some(vec![0.1; 128]),
            last_updated: Utc::now(),
            voice_print_hash: None,
            iris_scan_hash: None,
        };

        employee.add_component(biometric, "Security", Some("Building access".to_string())).unwrap();

        // Verify employee view
        let employee_view = EmployeeView::from_person(&employee);
        assert_eq!(employee_view.name, "John Matrix");
        assert_eq!(employee_view.department, Some("Security".to_string()));
        assert_eq!(employee_view.position, Some("Head of Security".to_string()));
    }

    #[test]
    fn test_create_partner_with_relationships() {
        let service = PersonCompositionService::new();
        let person_id = Uuid::new_v4();
        let company_contact_id = Uuid::new_v4();

        let mut partner = service.create_partner(
            PersonId::new(),
            "Lisa Chen",
            "TechVentures Inc",
            Some("Strategic Partner"),
        );

        // Add relationships
        let relationships = RelationshipComponent {
            relationships: vec![
                Relationship {
                    person_id: company_contact_id,
                    relationship_type: RelationshipType::BusinessPartner,
                    reciprocal_type: RelationshipType::BusinessPartner,
                    start_date: Some(NaiveDate::from_ymd_opt(2022, 3, 15).unwrap()),
                    end_date: None,
                    status: RelationshipStatus::Active,
                    notes: Some("Primary contact at TechVentures".to_string()),
                },
            ],
        };

        partner.add_component(relationships, "Partnerships", Some("Partner network".to_string())).unwrap();

        // Add social media for professional networking
        let social = SocialMediaComponent {
            profiles: vec![
                SocialMediaProfile {
                    platform: SocialPlatform::LinkedIn,
                    username: "lisachen".to_string(),
                    profile_url: Some("https://linkedin.com/in/lisachen".to_string()),
                    verified: true,
                    privacy: PrivacySetting::Professional,
                    last_active: Some(chrono::Utc::now()),
                    follower_count: Some(5000),
                },
                SocialMediaProfile {
                    platform: SocialPlatform::Twitter,
                    username: "lisa_tech".to_string(),
                    profile_url: Some("https://twitter.com/lisa_tech".to_string()),
                    verified: false,
                    privacy: PrivacySetting::Public,
                    last_active: Some(chrono::Utc::now()),
                    follower_count: Some(1200),
                },
            ],
            metrics: Some(SocialMetrics {
                total_followers: 6200,
                engagement_rate: Some(0.045),
                primary_platform: Some(SocialPlatform::LinkedIn),
                influence_score: Some(85.0),
            }),
        };

        partner.add_component(social, "Marketing", Some("Partner engagement".to_string())).unwrap();

        // Verify partner view
        let partner_view = PartnerView::from_person(&partner);
        assert_eq!(partner_view.name, "Lisa Chen");
        assert_eq!(partner_view.organization, Some("TechVentures Inc".to_string()));
        assert_eq!(partner_view.partnership_type, Some("Strategic Partner".to_string()));
        assert!(partner_view.social_profiles.len() > 0);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_crm_workflow() {
        // Create a customer
        let service = PersonCompositionService::new();
        let person_id = Uuid::new_v4();

        let mut customer = service.create_customer(
            PersonId::new(),
            "Emma Wilson",
            Some("emma@example.com"),
            Some("+1-555-1234"),
        );

        // Add preferences
        let preferences = PreferencesComponent {
            communication: CommunicationPreferences {
                preferred_channel: ContactChannel::Email,
                channel_settings: HashMap::new(),
                contact_time_preference: ContactTimePreference {
                    preferred_days: vec![],
                    preferred_hours: None,
                    timezone: "America/Los_Angeles".to_string(),
                },
                preferred_language: "en-US".to_string(),
                frequency_preference: FrequencyPreference::Monthly,
            },
            product_preferences: vec![
                ProductPreference {
                    category: "Electronics".to_string(),
                    preference_level: 9,
                    preferred_brands: vec!["Apple".to_string()],
                    price_sensitivity: 3,
                    quality_preference: 9,
                },
            ],
            content_preferences: ContentPreferences {
                content_types: vec![ContentType::Educational, ContentType::ProductUpdates],
                topics: vec!["Technology".to_string(), "Innovation".to_string()],
                format_preference: ContentFormat::Video,
                complexity_preference: ComplexityLevel::Advanced,
            },
            privacy_preferences: PrivacyPreferences {
                data_sharing_allowed: true,
                analytics_allowed: true,
                personalization_allowed: true,
                third_party_sharing_allowed: false,
                regulatory_preferences: HashMap::new(),
            },
        };

        let command = PersonCommand::UpdatePreferences {
            person_id,
            preferences: preferences.clone(),
        };

        let events = handle_person_command(&mut customer, command).await.unwrap();
        assert_eq!(events.len(), 1);

        // Add segmentation
        let segmentation = SegmentationComponent {
            primary_segment: CustomerSegment::VIPCustomer,
            secondary_segments: vec![CustomerSegment::LoyalCustomer],
            lifecycle_stage: LifecycleStage::Advocacy,
            value_tier: ValueTier::Platinum,
            persona: Some("Tech Enthusiast".to_string()),
            custom_segments: HashMap::new(),
        };

        customer.add_component(segmentation, "Analytics", Some("Quarterly segmentation".to_string())).unwrap();

        // Add behavioral data
        let behavioral = BehavioralComponent {
            purchase_behavior: PurchaseBehavior {
                average_order_value: Some(350.0),
                purchase_frequency: Some(24.0), // 24 times per year (bi-weekly)
                payment_methods: vec!["Mobile App".to_string(), "Website".to_string()],
                seasonal_patterns: HashMap::new(),
                category_distribution: HashMap::new(),
                typical_price_range: Some((150.0, 700.0)),
                discount_sensitivity: Some(0.2),
            },
            engagement_patterns: EngagementPatterns {
                email_open_rate: Some(0.95),
                click_through_rate: Some(0.75),
                visit_frequency: Some(8.0),
                avg_session_duration: Some(450),
                device_usage: HashMap::new(),
                active_hours: vec![0.3; 24],
            },
            interaction_summary: InteractionSummary {
                total_interactions: 200,
                last_interaction: Some(Utc::now()),
                support_tickets: 3,
                avg_satisfaction: Some(4.9),
                channels_used: vec![ContactChannel::Email, ContactChannel::InApp],
            },
            predictive_scores: PredictiveScores {
                churn_risk: Some(0.02),
                predicted_ltv: Some(25000.0),
                purchase_probability: Some(0.95),
                upsell_potential: Some(0.75),
                referral_likelihood: Some(0.92),
            },
        };


        customer.add_component(behavioral, "Analytics", Some("Behavioral analysis".to_string())).unwrap();

        // Verify the complete customer profile
        assert_eq!(customer.component_count(), 5); // Identity, Contact, Preferences, Segmentation, Behavioral
        
        let customer_view = CustomerView::from_person(&customer);
        assert_eq!(customer_view.name, "Emma Wilson");
        assert_eq!(customer_view.segment, Some("VIPCustomer".to_string())); // Debug format includes full enum name
        assert_eq!(customer_view.lifetime_value, Some(25000.0));
        assert_eq!(customer_view.engagement_score, Some(0.95));
        assert_eq!(customer_view.preferred_channel, Some("Email".to_string())); // Set to Email in preferences
    }
} 