//! Social and relationship components for CRM functionality
//!
//! These components capture a person's social connections, online presence,
//! and personal interests for relationship management.

use cim_domain::Component;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

/// Relationships with other people
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipComponent {
    /// List of relationships
    pub relationships: Vec<Relationship>,
}

/// A relationship with another person
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    /// ID of the related person
    pub person_id: uuid::Uuid,
    
    /// Type of relationship
    pub relationship_type: RelationshipType,
    
    /// Reciprocal type (from their perspective)
    pub reciprocal_type: RelationshipType,
    
    /// When the relationship started
    pub start_date: Option<chrono::NaiveDate>,
    
    /// When the relationship ended (if applicable)
    pub end_date: Option<chrono::NaiveDate>,
    
    /// Relationship status
    pub status: RelationshipStatus,
    
    /// Additional context or notes
    pub notes: Option<String>,
}

/// Type of relationship
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    // Family relationships
    Parent,
    Child,
    Sibling,
    Spouse,
    Partner,
    Grandparent,
    Grandchild,
    Aunt,
    Uncle,
    Cousin,
    InLaw(String),
    
    // Professional relationships
    Manager,
    DirectReport,
    Colleague,
    Mentor,
    Mentee,
    BusinessPartner,
    Client,
    Vendor,
    
    // Social relationships
    Friend,
    Acquaintance,
    Neighbor,
    Roommate,
    
    // Other
    Other(String),
}

/// Relationship status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipStatus {
    Active,
    Inactive,
    Estranged,
    Deceased,
    Unknown,
}

/// Social media presence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocialMediaComponent {
    /// Social media profiles
    pub profiles: Vec<SocialMediaProfile>,
    
    /// Aggregated social metrics
    pub metrics: Option<SocialMetrics>,
}

/// A social media profile
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialMediaProfile {
    /// Platform name
    pub platform: SocialPlatform,
    
    /// Username/handle
    pub username: String,
    
    /// Profile URL
    pub profile_url: Option<String>,
    
    /// Whether this is verified/authenticated
    pub verified: bool,
    
    /// Privacy setting
    pub privacy: PrivacySetting,
    
    /// Last known activity
    pub last_active: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Follower/connection count
    pub follower_count: Option<u64>,
}

/// Social media platforms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialPlatform {
    LinkedIn,
    Twitter,
    Facebook,
    Instagram,
    TikTok,
    YouTube,
    GitHub,
    Reddit,
    Discord,
    Slack,
    Other(String),
}

/// Privacy settings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacySetting {
    Public,
    Private,
    FriendsOnly,
    Professional,
    Unknown,
}

/// Aggregated social media metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocialMetrics {
    /// Total followers across platforms
    pub total_followers: u64,
    
    /// Engagement rate (if calculable)
    pub engagement_rate: Option<f32>,
    
    /// Primary platform
    pub primary_platform: Option<SocialPlatform>,
    
    /// Influence score (if calculated)
    pub influence_score: Option<f32>,
}

/// Personal interests and hobbies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterestsComponent {
    /// Categories of interests
    pub interests: HashMap<InterestCategory, Vec<Interest>>,
    
    /// Derived interest profile
    pub interest_profile: Option<InterestProfile>,
}

/// Interest categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterestCategory {
    Sports,
    Arts,
    Technology,
    Travel,
    Food,
    Music,
    Reading,
    Gaming,
    Outdoors,
    Fitness,
    Volunteering,
    Politics,
    Business,
    Other(String),
}

/// A specific interest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Interest {
    /// Name of the interest
    pub name: String,
    
    /// Level of interest (1-10)
    pub level: u8,
    
    /// How long they've had this interest
    pub duration_years: Option<f32>,
    
    /// Specific aspects or sub-interests
    pub specifics: Vec<String>,
    
    /// Related activities
    pub activities: Vec<String>,
}

/// Derived interest profile for segmentation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterestProfile {
    /// Primary interest category
    pub primary_category: InterestCategory,
    
    /// Interest diversity score (0-1)
    pub diversity_score: f32,
    
    /// Activity level (0-1)
    pub activity_level: f32,
    
    /// Social vs solo preference (-1 to 1)
    pub social_preference: f32,
}

// Component trait implementations

impl Component for RelationshipComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Relationship"
    }
}

impl Component for SocialMediaComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "SocialMedia"
    }
}

impl Component for InterestsComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Interests"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_relationships() {
        let relationships = RelationshipComponent {
            relationships: vec![
                Relationship {
                    person_id: uuid::Uuid::new_v4(),
                    relationship_type: RelationshipType::Parent,
                    reciprocal_type: RelationshipType::Child,
                    start_date: None, // Birth
                    end_date: None,
                    status: RelationshipStatus::Active,
                    notes: Some("Mother".to_string()),
                },
                Relationship {
                    person_id: uuid::Uuid::new_v4(),
                    relationship_type: RelationshipType::Sibling,
                    reciprocal_type: RelationshipType::Sibling,
                    start_date: None,
                    end_date: None,
                    status: RelationshipStatus::Active,
                    notes: Some("Younger brother".to_string()),
                },
            ],
        };

        assert_eq!(relationships.relationships.len(), 2);
        assert_eq!(
            relationships.relationships[0].relationship_type,
            RelationshipType::Parent
        );
    }

    #[test]
    fn test_social_media_profiles() {
        let social = SocialMediaComponent {
            profiles: vec![
                SocialMediaProfile {
                    platform: SocialPlatform::LinkedIn,
                    username: "john-doe-123".to_string(),
                    profile_url: Some("https://linkedin.com/in/john-doe-123".to_string()),
                    verified: true,
                    privacy: PrivacySetting::Professional,
                    last_active: Some(chrono::Utc::now()),
                    follower_count: Some(500),
                },
                SocialMediaProfile {
                    platform: SocialPlatform::Twitter,
                    username: "@johndoe".to_string(),
                    profile_url: Some("https://twitter.com/johndoe".to_string()),
                    verified: false,
                    privacy: PrivacySetting::Public,
                    last_active: Some(chrono::Utc::now()),
                    follower_count: Some(1200),
                },
            ],
            metrics: Some(SocialMetrics {
                total_followers: 1700,
                engagement_rate: Some(0.045),
                primary_platform: Some(SocialPlatform::Twitter),
                influence_score: Some(6.5),
            }),
        };

        assert_eq!(social.profiles.len(), 2);
        assert_eq!(social.metrics.as_ref().unwrap().total_followers, 1700);
    }

    #[test]
    fn test_interests() {
        let mut interests_map = HashMap::new();
        
        interests_map.insert(
            InterestCategory::Sports,
            vec![
                Interest {
                    name: "Tennis".to_string(),
                    level: 8,
                    duration_years: Some(5.0),
                    specifics: vec!["Singles".to_string(), "Clay courts".to_string()],
                    activities: vec!["Weekly league".to_string(), "Coaching".to_string()],
                },
            ],
        );
        
        interests_map.insert(
            InterestCategory::Technology,
            vec![
                Interest {
                    name: "AI/ML".to_string(),
                    level: 9,
                    duration_years: Some(3.0),
                    specifics: vec!["Deep learning".to_string(), "NLP".to_string()],
                    activities: vec!["Online courses".to_string(), "Hackathons".to_string()],
                },
            ],
        );

        let interests = InterestsComponent {
            interests: interests_map,
            interest_profile: Some(InterestProfile {
                primary_category: InterestCategory::Technology,
                diversity_score: 0.7,
                activity_level: 0.8,
                social_preference: 0.2,
            }),
        };

        assert_eq!(interests.interests.len(), 2);
        assert!(interests.interests.contains_key(&InterestCategory::Sports));
        assert_eq!(
            interests.interest_profile.as_ref().unwrap().primary_category,
            InterestCategory::Technology
        );
    }
} 