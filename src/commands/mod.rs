//! Person domain commands

use cim_domain::EntityId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;

use crate::aggregate::PersonMarker;
use crate::value_objects::*;

/// Person ID type alias
pub type PersonId = EntityId<PersonMarker>;

/// Re-export AddressType for use in events
pub use crate::value_objects::AddressType;

/// Commands for the Person domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonCommand {
    /// Create a new person
    CreatePerson(CreatePerson),
    
    /// Update person's name
    UpdateName(UpdateName),
    
    /// Add an email address
    AddEmail(AddEmail),
    
    /// Remove an email address
    RemoveEmail(RemoveEmail),
    
    /// Verify an email address
    VerifyEmail(VerifyEmail),
    
    /// Add a phone number
    AddPhone(AddPhone),
    
    /// Remove a phone number
    RemovePhone(RemovePhone),
    
    /// Add a physical address
    AddAddress(AddAddress),
    
    /// Remove a physical address
    RemoveAddress(RemoveAddress),
    
    /// Add an employment
    AddEmployment(AddEmployment),
    
    /// Update an employment
    UpdateEmployment(UpdateEmployment),
    
    /// End an employment
    EndEmployment(EndEmployment),
    
    /// Add a skill
    AddSkill(AddSkill),
    
    /// Update a skill
    UpdateSkill(UpdateSkill),
    
    /// Remove a skill
    RemoveSkill(RemoveSkill),
    
    /// Add a certification
    AddCertification(AddCertification),
    
    /// Add an education
    AddEducation(AddEducation),
    
    /// Add a relationship
    AddRelationship(AddRelationship),
    
    /// Update a relationship
    UpdateRelationship(UpdateRelationship),
    
    /// End a relationship
    EndRelationship(EndRelationship),
    
    /// Add a social profile
    AddSocialProfile(AddSocialProfile),
    
    /// Update a social profile
    UpdateSocialProfile(UpdateSocialProfile),
    
    /// Remove a social profile
    RemoveSocialProfile(RemoveSocialProfile),
    
    /// Set a customer segment
    SetCustomerSegment(SetCustomerSegment),
    
    /// Update behavioral data
    UpdateBehavioralData(UpdateBehavioralData),
    
    /// Set communication preferences
    SetCommunicationPreferences(SetCommunicationPreferences),
    
    /// Set privacy preferences
    SetPrivacyPreferences(SetPrivacyPreferences),
    
    /// Add a tag
    AddTag(AddTag),
    
    /// Remove a tag
    RemoveTag(RemoveTag),
    
    /// Set a custom attribute
    SetCustomAttribute(SetCustomAttribute),
    
    /// Deactivate the person
    DeactivatePerson(DeactivatePerson),
    
    /// Reactivate the person
    ReactivatePerson(ReactivatePerson),
    
    /// Merge two persons
    MergePersons(MergePersons),
}

// ===== Basic Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePerson {
    pub person_id: PersonId,
    pub name: PersonName,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateName {
    pub person_id: PersonId,
    pub name: PersonName,
    pub reason: Option<String>,
}

// ===== Contact Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEmail {
    pub person_id: PersonId,
    pub email: EmailAddress,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveEmail {
    pub person_id: PersonId,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyEmail {
    pub person_id: PersonId,
    pub email: String,
    pub verification_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPhone {
    pub person_id: PersonId,
    pub phone: PhoneNumber,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovePhone {
    pub person_id: PersonId,
    pub phone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAddress {
    pub person_id: PersonId,
    pub address: PhysicalAddress,
    pub address_type: AddressType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAddress {
    pub person_id: PersonId,
    pub address_type: AddressType,
}

// ===== Employment Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEmployment {
    pub person_id: PersonId,
    pub employment: Employment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEmployment {
    pub person_id: PersonId,
    pub organization_id: Uuid,
    pub updates: EmploymentUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentUpdate {
    pub department: Option<String>,
    pub position: Option<String>,
    pub manager_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndEmployment {
    pub person_id: PersonId,
    pub organization_id: Uuid,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

// ===== Skills & Education Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSkill {
    pub person_id: PersonId,
    pub skill: Skill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSkill {
    pub person_id: PersonId,
    pub skill_name: String,
    pub proficiency: ProficiencyLevel,
    pub last_used: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSkill {
    pub person_id: PersonId,
    pub skill_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCertification {
    pub person_id: PersonId,
    pub certification: Certification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddEducation {
    pub person_id: PersonId,
    pub education: Education,
}

// ===== Relationship Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRelationship {
    pub person_id: PersonId,
    pub relationship: Relationship,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRelationship {
    pub person_id: PersonId,
    pub related_person_id: Uuid,
    pub status: RelationshipStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndRelationship {
    pub person_id: PersonId,
    pub related_person_id: Uuid,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

// ===== Social Media Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSocialProfile {
    pub person_id: PersonId,
    pub profile: SocialProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSocialProfile {
    pub person_id: PersonId,
    pub platform: SocialPlatform,
    pub updates: SocialProfileUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfileUpdate {
    pub username: Option<String>,
    pub verified: Option<bool>,
    pub follower_count: Option<u64>,
    pub engagement_rate: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSocialProfile {
    pub person_id: PersonId,
    pub platform: SocialPlatform,
}

// ===== Customer/Business Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCustomerSegment {
    pub person_id: PersonId,
    pub segment: CustomerSegment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBehavioralData {
    pub person_id: PersonId,
    pub data: BehavioralData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCommunicationPreferences {
    pub person_id: PersonId,
    pub preferences: CommunicationPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPrivacyPreferences {
    pub person_id: PersonId,
    pub preferences: PrivacyPreferences,
}

// ===== Tag & Metadata Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTag {
    pub person_id: PersonId,
    pub tag: Tag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTag {
    pub person_id: PersonId,
    pub tag_name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCustomAttribute {
    pub person_id: PersonId,
    pub attribute: CustomAttribute,
}

// ===== Lifecycle Commands =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeactivatePerson {
    pub person_id: PersonId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactivatePerson {
    pub person_id: PersonId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergePersons {
    pub source_person_id: PersonId,
    pub target_person_id: PersonId,
    pub merge_reason: MergeReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeReason {
    DuplicateIdentity,
    DataQualityIssue,
    UserRequested,
    PolicyDetermined,
}


