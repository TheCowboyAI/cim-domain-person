//! Social media and networking component data structures

use super::ComponentDataTrait;
use crate::aggregate::ComponentType;
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Social media profile component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMediaProfileData {
    pub platform: SocialPlatform,
    pub username: String,
    pub profile_url: String,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub follower_count: Option<u64>,
    pub following_count: Option<u64>,
    pub is_verified: bool,
    pub is_business_account: bool,
    pub last_synced: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialPlatform {
    LinkedIn,
    Twitter,
    Facebook,
    Instagram,
    GitHub,
    YouTube,
    TikTok,
    Reddit,
    Pinterest,
    Mastodon,
    Other,
}

impl ComponentDataTrait for SocialMediaProfileData {
    fn component_type(&self) -> ComponentType {
        ComponentType::SocialProfile
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.username.trim().is_empty() {
            return Err(DomainError::ValidationError("Username cannot be empty".to_string()));
        }
        
        if self.profile_url.trim().is_empty() {
            return Err(DomainError::ValidationError("Profile URL cannot be empty".to_string()));
        }
        
        // Basic URL validation
        if !self.profile_url.starts_with("http://") && !self.profile_url.starts_with("https://") {
            return Err(DomainError::ValidationError("Profile URL must start with http:// or https://".to_string()));
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        let verified = if self.is_verified { " ✓" } else { "" };
        format!("{} on {:?}{}", self.username, self.platform, verified)
    }
}

/// Personal website component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalWebsiteData {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub website_type: WebsiteType,
    pub is_primary: bool,
    pub technologies: Vec<String>,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebsiteType {
    Portfolio,
    Blog,
    Business,
    Personal,
    Documentation,
    Other,
}

impl ComponentDataTrait for PersonalWebsiteData {
    fn component_type(&self) -> ComponentType {
        ComponentType::Website
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.url.trim().is_empty() {
            return Err(DomainError::ValidationError("URL cannot be empty".to_string()));
        }
        
        if self.title.trim().is_empty() {
            return Err(DomainError::ValidationError("Title cannot be empty".to_string()));
        }
        
        // Basic URL validation
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err(DomainError::ValidationError("URL must start with http:// or https://".to_string()));
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("{} ({:?})", self.title, self.website_type)
    }
}

/// Professional network component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionalNetworkData {
    pub network_name: String,
    pub member_id: String,
    pub profile_url: Option<String>,
    pub connection_count: Option<u32>,
    pub membership_level: MembershipLevel,
    pub specializations: Vec<String>,
    pub joined_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembershipLevel {
    Basic,
    Premium,
    Professional,
    Enterprise,
    Custom,
}

impl ComponentDataTrait for ProfessionalNetworkData {
    fn component_type(&self) -> ComponentType {
        ComponentType::ProfessionalNetwork
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.network_name.trim().is_empty() {
            return Err(DomainError::ValidationError("Network name cannot be empty".to_string()));
        }
        
        if self.member_id.trim().is_empty() {
            return Err(DomainError::ValidationError("Member ID cannot be empty".to_string()));
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("{} - {:?} member", self.network_name, self.membership_level)
    }
}

/// Wrapper enum for all social data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SocialData {
    SocialMedia(SocialMediaData),
    Website(WebsiteData),
    ProfessionalNetwork(ProfessionalNetworkData),
    Relationship(RelationshipData),
}

// Type aliases for clarity
pub type SocialMediaData = SocialMediaProfileData;
pub type WebsiteData = PersonalWebsiteData;

/// Relationship data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipData {
    pub other_person_id: crate::aggregate::PersonId,
    pub relationship_type: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

impl SocialData {
    pub fn social_type(&self) -> &'static str {
        match self {
            SocialData::SocialMedia(_) => "SocialMedia",
            SocialData::Website(_) => "Website",
            SocialData::ProfessionalNetwork(_) => "ProfessionalNetwork",
            SocialData::Relationship(_) => "Relationship",
        }
    }
    
    pub fn component_type(&self) -> ComponentType {
        match self {
            SocialData::SocialMedia(data) => data.component_type(),
            SocialData::Website(data) => data.component_type(),
            SocialData::ProfessionalNetwork(data) => data.component_type(),
            SocialData::Relationship(_) => ComponentType::Relationship,
        }
    }
    
    pub fn validate(&self) -> DomainResult<()> {
        match self {
            SocialData::SocialMedia(data) => data.validate(),
            SocialData::Website(data) => data.validate(),
            SocialData::ProfessionalNetwork(data) => data.validate(),
            SocialData::Relationship(data) => {
                if data.relationship_type.is_empty() {
                    return Err(DomainError::ValidationError("Relationship type is required".to_string()));
                }
                Ok(())
            }
        }
    }
    
    pub fn summary(&self) -> String {
        match self {
            SocialData::SocialMedia(data) => data.summary(),
            SocialData::Website(data) => data.summary(),
            SocialData::ProfessionalNetwork(data) => data.summary(),
            SocialData::Relationship(data) => format!("{} relationship", data.relationship_type),
        }
    }
} 