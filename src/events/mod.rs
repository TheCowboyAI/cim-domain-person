//! Person domain events

use cim_domain::EntityId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;

use crate::aggregate::PersonMarker;
use crate::value_objects::*;
use crate::commands::{MergeReason, EmploymentUpdate, SocialProfileUpdate};

/// Person ID type alias
pub type PersonId = EntityId<PersonMarker>;

/// Events for the Person domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonEvent {
    // Basic events
    PersonCreated(PersonCreated),
    NameUpdated(NameUpdated),
    
    // Contact events
    EmailAdded(EmailAdded),
    EmailRemoved(EmailRemoved),
    EmailVerified(EmailVerified),
    PhoneAdded(PhoneAdded),
    PhoneRemoved(PhoneRemoved),
    AddressAdded(AddressAdded),
    AddressRemoved(AddressRemoved),
    
    // Employment events
    EmploymentAdded(EmploymentAdded),
    EmploymentUpdated(EmploymentUpdated),
    EmploymentEnded(EmploymentEnded),
    
    // Skills & Education events
    SkillAdded(SkillAdded),
    SkillUpdated(SkillUpdated),
    SkillRemoved(SkillRemoved),
    CertificationAdded(CertificationAdded),
    EducationAdded(EducationAdded),
    
    // Relationship events
    RelationshipAdded(RelationshipAdded),
    RelationshipUpdated(RelationshipUpdated),
    RelationshipEnded(RelationshipEnded),
    
    // Social Media events
    SocialProfileAdded(SocialProfileAdded),
    SocialProfileUpdated(SocialProfileUpdated),
    SocialProfileRemoved(SocialProfileRemoved),
    
    // Customer/Business events
    CustomerSegmentSet(CustomerSegmentSet),
    BehavioralDataUpdated(BehavioralDataUpdated),
    CommunicationPreferencesSet(CommunicationPreferencesSet),
    PrivacyPreferencesSet(PrivacyPreferencesSet),
    
    // Tag & Metadata events
    TagAdded(TagAdded),
    TagRemoved(TagRemoved),
    CustomAttributeSet(CustomAttributeSet),
    
    // Lifecycle events
    PersonDeactivated(PersonDeactivated),
    PersonReactivated(PersonReactivated),
    PersonsMerged(PersonsMerged),
    PersonMergedInto(PersonMergedInto),
}

// ===== Basic Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCreated {
    pub person_id: PersonId,
    pub name: PersonName,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameUpdated {
    pub person_id: PersonId,
    pub old_name: PersonName,
    pub new_name: PersonName,
    pub reason: Option<String>,
    pub updated_at: DateTime<Utc>,
}

// ===== Contact Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAdded {
    pub person_id: PersonId,
    pub email: EmailAddress,
    pub primary: bool,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRemoved {
    pub person_id: PersonId,
    pub email: String,
    pub removed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerified {
    pub person_id: PersonId,
    pub email: String,
    pub verified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneAdded {
    pub person_id: PersonId,
    pub phone: PhoneNumber,
    pub primary: bool,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneRemoved {
    pub person_id: PersonId,
    pub phone: String,
    pub removed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressAdded {
    pub person_id: PersonId,
    pub address: PhysicalAddress,
    pub address_type: AddressType,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressRemoved {
    pub person_id: PersonId,
    pub address_type: AddressType,
    pub removed_at: DateTime<Utc>,
}

// ===== Employment Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentAdded {
    pub person_id: PersonId,
    pub employment: Employment,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentUpdated {
    pub person_id: PersonId,
    pub organization_id: Uuid,
    pub updates: EmploymentUpdate,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentEnded {
    pub person_id: PersonId,
    pub organization_id: Uuid,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub ended_at: DateTime<Utc>,
}

// ===== Skills & Education Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAdded {
    pub person_id: PersonId,
    pub skill: Skill,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdated {
    pub person_id: PersonId,
    pub skill_name: String,
    pub proficiency: ProficiencyLevel,
    pub last_used: Option<NaiveDate>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRemoved {
    pub person_id: PersonId,
    pub skill_name: String,
    pub removed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationAdded {
    pub person_id: PersonId,
    pub certification: Certification,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationAdded {
    pub person_id: PersonId,
    pub education: Education,
    pub added_at: DateTime<Utc>,
}

// ===== Relationship Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipAdded {
    pub person_id: PersonId,
    pub relationship: Relationship,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipUpdated {
    pub person_id: PersonId,
    pub related_person_id: Uuid,
    pub status: RelationshipStatus,
    pub notes: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipEnded {
    pub person_id: PersonId,
    pub related_person_id: Uuid,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub ended_at: DateTime<Utc>,
}

// ===== Social Media Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfileAdded {
    pub person_id: PersonId,
    pub profile: SocialProfile,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfileUpdated {
    pub person_id: PersonId,
    pub platform: SocialPlatform,
    pub updates: SocialProfileUpdate,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfileRemoved {
    pub person_id: PersonId,
    pub platform: SocialPlatform,
    pub removed_at: DateTime<Utc>,
}

// ===== Customer/Business Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSegmentSet {
    pub person_id: PersonId,
    pub segment: CustomerSegment,
    pub set_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralDataUpdated {
    pub person_id: PersonId,
    pub data: BehavioralData,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPreferencesSet {
    pub person_id: PersonId,
    pub preferences: CommunicationPreferences,
    pub set_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPreferencesSet {
    pub person_id: PersonId,
    pub preferences: PrivacyPreferences,
    pub set_at: DateTime<Utc>,
}

// ===== Tag & Metadata Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagAdded {
    pub person_id: PersonId,
    pub tag: Tag,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRemoved {
    pub person_id: PersonId,
    pub tag_name: String,
    pub category: String,
    pub removed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAttributeSet {
    pub person_id: PersonId,
    pub attribute: CustomAttribute,
    pub set_at: DateTime<Utc>,
}

// ===== Lifecycle Events =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonDeactivated {
    pub person_id: PersonId,
    pub reason: String,
    pub deactivated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonReactivated {
    pub person_id: PersonId,
    pub reason: String,
    pub reactivated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonsMerged {
    pub source_person_id: PersonId,
    pub target_person_id: PersonId,
    pub merge_reason: MergeReason,
    pub merged_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonMergedInto {
    pub person_id: PersonId,
    pub merged_into_id: PersonId,
    pub merge_reason: MergeReason,
    pub merged_at: DateTime<Utc>,
}
