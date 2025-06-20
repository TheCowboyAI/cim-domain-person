//! Behavioral and CRM-specific components for customer relationship management
//!
//! These components capture behavioral patterns, preferences, and segmentation
//! data used for personalization and customer relationship management.

use cim_domain::Component;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

/// Communication preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreferencesComponent {
    /// Communication channel preferences
    pub communication: CommunicationPreferences,
    
    /// Product/service preferences
    pub product_preferences: Vec<ProductPreference>,
    
    /// Content preferences
    pub content_preferences: ContentPreferences,
    
    /// Privacy preferences
    pub privacy_preferences: PrivacyPreferences,
}

/// Communication channel preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunicationPreferences {
    /// Preferred contact method
    pub preferred_channel: ContactChannel,
    
    /// Channel-specific preferences
    pub channel_settings: HashMap<ContactChannel, ChannelSettings>,
    
    /// Best time to contact
    pub contact_time_preference: ContactTimePreference,
    
    /// Language preference
    pub preferred_language: String,
    
    /// Communication frequency preference
    pub frequency_preference: FrequencyPreference,
}

/// Contact channels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContactChannel {
    Email,
    Phone,
    SMS,
    WhatsApp,
    InApp,
    PostalMail,
    None,
}

/// Channel-specific settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelSettings {
    /// Opt-in status
    pub opted_in: bool,
    
    /// Marketing allowed
    pub marketing_allowed: bool,
    
    /// Transactional allowed
    pub transactional_allowed: bool,
    
    /// Last opt-in/out date
    pub last_consent_date: Option<chrono::NaiveDate>,
}

/// Contact time preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactTimePreference {
    /// Preferred days
    pub preferred_days: Vec<chrono::Weekday>,
    
    /// Preferred time range (in their timezone)
    pub preferred_hours: Option<(u8, u8)>, // (start_hour, end_hour)
    
    /// Their timezone
    pub timezone: String,
}

/// Communication frequency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrequencyPreference {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
    OnlyImportant,
    Never,
}

/// Product/service preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductPreference {
    /// Product category
    pub category: String,
    
    /// Preference level (1-10)
    pub preference_level: u8,
    
    /// Specific brands liked
    pub preferred_brands: Vec<String>,
    
    /// Price sensitivity (1-10, higher = more sensitive)
    pub price_sensitivity: u8,
    
    /// Quality preference (1-10, higher = prefers quality)
    pub quality_preference: u8,
}

/// Content preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPreferences {
    /// Preferred content types
    pub content_types: Vec<ContentType>,
    
    /// Topics of interest
    pub topics: Vec<String>,
    
    /// Content format preference
    pub format_preference: ContentFormat,
    
    /// Reading level preference
    pub complexity_preference: ComplexityLevel,
}

/// Content types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Educational,
    Promotional,
    News,
    Entertainment,
    HowTo,
    CaseStudies,
    ProductUpdates,
}

/// Content formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentFormat {
    Text,
    Video,
    Audio,
    Infographic,
    Interactive,
    Mixed,
}

/// Content complexity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Privacy preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyPreferences {
    /// Data sharing consent
    pub data_sharing_allowed: bool,
    
    /// Analytics tracking allowed
    pub analytics_allowed: bool,
    
    /// Personalization allowed
    pub personalization_allowed: bool,
    
    /// Third-party sharing allowed
    pub third_party_sharing_allowed: bool,
    
    /// GDPR/CCPA preferences
    pub regulatory_preferences: HashMap<String, bool>,
}

/// Behavioral patterns and history
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehavioralComponent {
    /// Purchase behavior
    pub purchase_behavior: PurchaseBehavior,
    
    /// Engagement patterns
    pub engagement_patterns: EngagementPatterns,
    
    /// Interaction history summary
    pub interaction_summary: InteractionSummary,
    
    /// Predictive scores
    pub predictive_scores: PredictiveScores,
}

/// Purchase behavior patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseBehavior {
    /// Average order value
    pub average_order_value: Option<f64>,
    
    /// Purchase frequency (orders per year)
    pub purchase_frequency: Option<f32>,
    
    /// Preferred payment methods
    pub payment_methods: Vec<String>,
    
    /// Seasonal patterns
    pub seasonal_patterns: HashMap<String, f32>, // month -> relative activity
    
    /// Category preferences
    pub category_distribution: HashMap<String, f32>, // category -> percentage
    
    /// Price range preference
    pub typical_price_range: Option<(f64, f64)>,
    
    /// Discount sensitivity (0-1)
    pub discount_sensitivity: Option<f32>,
}

/// Engagement patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngagementPatterns {
    /// Email open rate
    pub email_open_rate: Option<f32>,
    
    /// Click-through rate
    pub click_through_rate: Option<f32>,
    
    /// Website visit frequency
    pub visit_frequency: Option<f32>,
    
    /// Average session duration (seconds)
    pub avg_session_duration: Option<u64>,
    
    /// Preferred devices
    pub device_usage: HashMap<String, f32>, // device -> percentage
    
    /// Active hours distribution
    pub active_hours: Vec<f32>, // 24 hours, normalized activity
}

/// Interaction history summary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InteractionSummary {
    /// Total interactions
    pub total_interactions: u64,
    
    /// Last interaction date
    pub last_interaction: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Support tickets count
    pub support_tickets: u32,
    
    /// Average satisfaction score
    pub avg_satisfaction: Option<f32>,
    
    /// Interaction channels used
    pub channels_used: Vec<ContactChannel>,
}

/// Predictive scores for various outcomes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictiveScores {
    /// Churn risk (0-1)
    pub churn_risk: Option<f32>,
    
    /// Lifetime value prediction
    pub predicted_ltv: Option<f64>,
    
    /// Next purchase likelihood (0-1)
    pub purchase_probability: Option<f32>,
    
    /// Upsell potential (0-1)
    pub upsell_potential: Option<f32>,
    
    /// Referral likelihood (0-1)
    pub referral_likelihood: Option<f32>,
}

/// Customer segmentation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentationComponent {
    /// Primary segment
    pub primary_segment: CustomerSegment,
    
    /// Secondary segments
    pub secondary_segments: Vec<CustomerSegment>,
    
    /// Lifecycle stage
    pub lifecycle_stage: LifecycleStage,
    
    /// Value tier
    pub value_tier: ValueTier,
    
    /// Persona assignment
    pub persona: Option<String>,
    
    /// Custom segments
    pub custom_segments: HashMap<String, String>,
}

/// Customer segments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomerSegment {
    NewCustomer,
    ActiveCustomer,
    LoyalCustomer,
    AtRiskCustomer,
    LapsedCustomer,
    VIPCustomer,
    Prospect,
    Lead,
    Custom(String),
}

/// Customer lifecycle stages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStage {
    Awareness,
    Consideration,
    Purchase,
    Retention,
    Advocacy,
    Reactivation,
}

/// Value tiers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueTier {
    Platinum,
    Gold,
    Silver,
    Bronze,
    Basic,
}

// Component trait implementations

impl Component for PreferencesComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Preferences"
    }
}

impl Component for BehavioralComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Behavioral"
    }
}

impl Component for SegmentationComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Segmentation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communication_preferences() {
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

        let prefs = PreferencesComponent {
            communication: CommunicationPreferences {
                preferred_channel: ContactChannel::Email,
                channel_settings,
                contact_time_preference: ContactTimePreference {
                    preferred_days: vec![
                        chrono::Weekday::Mon,
                        chrono::Weekday::Wed,
                        chrono::Weekday::Fri,
                    ],
                    preferred_hours: Some((9, 17)),
                    timezone: "America/New_York".to_string(),
                },
                preferred_language: "en-US".to_string(),
                frequency_preference: FrequencyPreference::Weekly,
            },
            product_preferences: vec![],
            content_preferences: ContentPreferences {
                content_types: vec![ContentType::Educational, ContentType::HowTo],
                topics: vec!["Technology".to_string(), "Innovation".to_string()],
                format_preference: ContentFormat::Video,
                complexity_preference: ComplexityLevel::Intermediate,
            },
            privacy_preferences: PrivacyPreferences {
                data_sharing_allowed: false,
                analytics_allowed: true,
                personalization_allowed: true,
                third_party_sharing_allowed: false,
                regulatory_preferences: HashMap::new(),
            },
        };

        assert_eq!(prefs.communication.preferred_channel, ContactChannel::Email);
        assert_eq!(
            prefs.communication.frequency_preference,
            FrequencyPreference::Weekly
        );
    }

    #[test]
    fn test_behavioral_patterns() {
        let mut category_dist = HashMap::new();
        category_dist.insert("Electronics".to_string(), 0.4);
        category_dist.insert("Books".to_string(), 0.3);
        category_dist.insert("Clothing".to_string(), 0.3);

        let behavioral = BehavioralComponent {
            purchase_behavior: PurchaseBehavior {
                average_order_value: Some(125.50),
                purchase_frequency: Some(8.5),
                payment_methods: vec!["Credit Card".to_string(), "PayPal".to_string()],
                seasonal_patterns: HashMap::new(),
                category_distribution: category_dist,
                typical_price_range: Some((50.0, 200.0)),
                discount_sensitivity: Some(0.7),
            },
            engagement_patterns: EngagementPatterns {
                email_open_rate: Some(0.35),
                click_through_rate: Some(0.12),
                visit_frequency: Some(2.5),
                avg_session_duration: Some(180),
                device_usage: HashMap::new(),
                active_hours: vec![0.1; 24], // Simplified
            },
            interaction_summary: InteractionSummary {
                total_interactions: 45,
                last_interaction: Some(chrono::Utc::now()),
                support_tickets: 2,
                avg_satisfaction: Some(4.5),
                channels_used: vec![ContactChannel::Email, ContactChannel::Phone],
            },
            predictive_scores: PredictiveScores {
                churn_risk: Some(0.15),
                predicted_ltv: Some(2500.0),
                purchase_probability: Some(0.75),
                upsell_potential: Some(0.6),
                referral_likelihood: Some(0.8),
            },
        };

        assert_eq!(behavioral.purchase_behavior.average_order_value, Some(125.50));
        assert_eq!(behavioral.predictive_scores.churn_risk, Some(0.15));
    }

    #[test]
    fn test_segmentation() {
        let mut custom_segments = HashMap::new();
        custom_segments.insert("Industry".to_string(), "Technology".to_string());
        custom_segments.insert("Region".to_string(), "Northeast".to_string());

        let segmentation = SegmentationComponent {
            primary_segment: CustomerSegment::LoyalCustomer,
            secondary_segments: vec![CustomerSegment::VIPCustomer],
            lifecycle_stage: LifecycleStage::Retention,
            value_tier: ValueTier::Gold,
            persona: Some("Tech Enthusiast".to_string()),
            custom_segments,
        };

        assert_eq!(segmentation.primary_segment, CustomerSegment::LoyalCustomer);
        assert_eq!(segmentation.value_tier, ValueTier::Gold);
        assert_eq!(segmentation.lifecycle_stage, LifecycleStage::Retention);
    }
} 