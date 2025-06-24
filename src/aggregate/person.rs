//! Person aggregate root

use cim_domain::{AggregateRoot, DomainError, DomainResult, EntityId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::value_objects::*;
use crate::commands::*;
use crate::events::*;

/// Marker type for Person entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonMarker;

/// Person ID type alias
pub type PersonId = EntityId<PersonMarker>;

/// The Person aggregate root
#[derive(Debug, Clone)]
pub struct Person {
    /// Unique identifier
    pub id: PersonId,
    
    /// Person's name
    pub name: PersonName,
    
    /// Whether the person is active
    pub is_active: bool,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    // Contact information
    pub emails: HashMap<String, EmailAddress>,
    pub primary_email: Option<String>,
    pub phones: HashMap<String, PhoneNumber>,
    pub primary_phone: Option<String>,
    pub addresses: HashMap<AddressType, PhysicalAddress>,
    
    // Employment
    pub employments: HashMap<Uuid, Employment>,
    pub current_employment: Option<Uuid>,
    
    // Skills & Education
    pub skills: HashMap<String, Skill>,
    pub certifications: Vec<Certification>,
    pub education: Vec<Education>,
    
    // Relationships
    pub relationships: HashMap<Uuid, Relationship>,
    
    // Social Media
    pub social_profiles: HashMap<SocialPlatform, SocialProfile>,
    
    // Customer/Business
    pub customer_segment: Option<CustomerSegment>,
    pub behavioral_data: Option<BehavioralData>,
    pub communication_preferences: Option<CommunicationPreferences>,
    pub privacy_preferences: Option<PrivacyPreferences>,
    
    // Tags & Metadata
    pub tags: Vec<Tag>,
    pub custom_attributes: HashMap<String, CustomAttribute>,
    
    // Merge tracking
    pub merged_into: Option<PersonId>,
    pub merge_status: MergeStatus,
    
    // Event sourcing
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergeStatus {
    Active,
    MergedInto(PersonId),
}

impl Person {
    /// Create a new person
    pub fn new(id: PersonId, name: PersonName, _source: String) -> Self {
        Self {
            id,
            name,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            emails: HashMap::new(),
            primary_email: None,
            phones: HashMap::new(),
            primary_phone: None,
            addresses: HashMap::new(),
            employments: HashMap::new(),
            current_employment: None,
            skills: HashMap::new(),
            certifications: Vec::new(),
            education: Vec::new(),
            relationships: HashMap::new(),
            social_profiles: HashMap::new(),
            customer_segment: None,
            behavioral_data: None,
            communication_preferences: None,
            privacy_preferences: None,
            tags: Vec::new(),
            custom_attributes: HashMap::new(),
            merged_into: None,
            merge_status: MergeStatus::Active,
            version: 0,
        }
    }

    /// Create an empty person for event replay
    pub fn empty() -> Self {
        Self {
            id: PersonId::new(), // Will be overwritten by PersonCreated event
            name: PersonName::new("".to_string(), "".to_string()),
            is_active: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            emails: HashMap::new(),
            primary_email: None,
            phones: HashMap::new(),
            primary_phone: None,
            addresses: HashMap::new(),
            employments: HashMap::new(),
            current_employment: None,
            skills: HashMap::new(),
            certifications: Vec::new(),
            education: Vec::new(),
            relationships: HashMap::new(),
            social_profiles: HashMap::new(),
            customer_segment: None,
            behavioral_data: None,
            communication_preferences: None,
            privacy_preferences: None,
            tags: Vec::new(),
            custom_attributes: HashMap::new(),
            merged_into: None,
            merge_status: MergeStatus::Active,
            version: 0,
        }
    }

    /// Replay an event stream to reconstruct the aggregate state
    pub fn replay_events(events: Vec<PersonEvent>) -> Result<Self, String> {
        if events.is_empty() {
            return Err("Cannot replay empty event stream".to_string());
        }

        // Verify first event is PersonCreated
        match events.first() {
            Some(PersonEvent::PersonCreated(_)) => {},
            _ => return Err("Event stream must start with PersonCreated event".to_string()),
        }

        // Start with empty aggregate
        let mut person = Self::empty();

        // Apply each event in sequence
        for event in events {
            person = person.apply_event(&event);
        }

        Ok(person)
    }

    /// Replay events from a specific point (for snapshots)
    pub fn replay_from_snapshot(
        snapshot: Self,
        events: Vec<PersonEvent>,
        expected_version: u64,
    ) -> Result<Self, String> {
        // Verify snapshot version matches
        if snapshot.version() != expected_version {
            return Err(format!(
                "Snapshot version {} does not match expected version {}",
                snapshot.version(),
                expected_version
            ));
        }

        let mut person = snapshot;

        // Apply events after snapshot
        for event in events {
            person = person.apply_event(&event);
        }

        Ok(person)
    }

    /// Get the current version (event count)
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Handle a command
    pub fn handle_command(&mut self, command: PersonCommand) -> Result<Vec<PersonEvent>, String> {
        // Check if person is merged
        if let MergeStatus::MergedInto(_) = self.merge_status {
            return Err("Cannot modify a merged person".to_string());
        }

        match command {
            PersonCommand::CreatePerson(cmd) => self.handle_create_person(cmd),
            PersonCommand::UpdateName(cmd) => self.handle_update_name(cmd),
            
            // Contact commands
            PersonCommand::AddEmail(cmd) => self.handle_add_email(cmd),
            PersonCommand::RemoveEmail(cmd) => self.handle_remove_email(cmd),
            PersonCommand::VerifyEmail(cmd) => self.handle_verify_email(cmd),
            PersonCommand::AddPhone(cmd) => self.handle_add_phone(cmd),
            PersonCommand::RemovePhone(cmd) => self.handle_remove_phone(cmd),
            PersonCommand::AddAddress(cmd) => self.handle_add_address(cmd),
            PersonCommand::RemoveAddress(cmd) => self.handle_remove_address(cmd),
            
            // Employment commands
            PersonCommand::AddEmployment(cmd) => self.handle_add_employment(cmd),
            PersonCommand::UpdateEmployment(cmd) => self.handle_update_employment(cmd),
            PersonCommand::EndEmployment(cmd) => self.handle_end_employment(cmd),
            
            // Skills & Education commands
            PersonCommand::AddSkill(cmd) => self.handle_add_skill(cmd),
            PersonCommand::UpdateSkill(cmd) => self.handle_update_skill(cmd),
            PersonCommand::RemoveSkill(cmd) => self.handle_remove_skill(cmd),
            PersonCommand::AddCertification(cmd) => self.handle_add_certification(cmd),
            PersonCommand::AddEducation(cmd) => self.handle_add_education(cmd),
            
            // Relationship commands
            PersonCommand::AddRelationship(cmd) => self.handle_add_relationship(cmd),
            PersonCommand::UpdateRelationship(cmd) => self.handle_update_relationship(cmd),
            PersonCommand::EndRelationship(cmd) => self.handle_end_relationship(cmd),
            
            // Social Media commands
            PersonCommand::AddSocialProfile(cmd) => self.handle_add_social_profile(cmd),
            PersonCommand::UpdateSocialProfile(cmd) => self.handle_update_social_profile(cmd),
            PersonCommand::RemoveSocialProfile(cmd) => self.handle_remove_social_profile(cmd),
            
            // Customer/Business commands
            PersonCommand::SetCustomerSegment(cmd) => self.handle_set_customer_segment(cmd),
            PersonCommand::UpdateBehavioralData(cmd) => self.handle_update_behavioral_data(cmd),
            PersonCommand::SetCommunicationPreferences(cmd) => self.handle_set_communication_preferences(cmd),
            PersonCommand::SetPrivacyPreferences(cmd) => self.handle_set_privacy_preferences(cmd),
            
            // Tag & Metadata commands
            PersonCommand::AddTag(cmd) => self.handle_add_tag(cmd),
            PersonCommand::RemoveTag(cmd) => self.handle_remove_tag(cmd),
            PersonCommand::SetCustomAttribute(cmd) => self.handle_set_custom_attribute(cmd),
            
            // Lifecycle commands
            PersonCommand::DeactivatePerson(cmd) => self.handle_deactivate(cmd),
            PersonCommand::ReactivatePerson(cmd) => self.handle_reactivate(cmd),
            PersonCommand::MergePersons(cmd) => self.handle_merge(cmd),
        }
    }

    /// Apply an event to create a new state (immutable)
    pub fn apply_event(self, event: &PersonEvent) -> Self {
        let mut new_state = self;
        
        // Increment version for each event
        new_state.version += 1;
        
        match event {
            PersonEvent::PersonCreated(e) => {
                new_state.id = e.person_id;
                new_state.name = e.name.clone();
                new_state.created_at = e.created_at;
                new_state.updated_at = e.created_at;
            }
            PersonEvent::NameUpdated(e) => {
                new_state.name = e.new_name.clone();
                new_state.updated_at = e.updated_at;
            }
            
            // Contact events
            PersonEvent::EmailAdded(e) => {
                new_state.emails.insert(e.email.address.clone(), e.email.clone());
                if e.primary {
                    new_state.primary_email = Some(e.email.address.clone());
                }
                new_state.updated_at = e.added_at;
            }
            PersonEvent::EmailRemoved(e) => {
                new_state.emails.remove(&e.email);
                if new_state.primary_email.as_ref() == Some(&e.email) {
                    new_state.primary_email = None;
                }
                new_state.updated_at = e.removed_at;
            }
            PersonEvent::EmailVerified(e) => {
                if let Some(email) = new_state.emails.get_mut(&e.email) {
                    email.verified = true;
                }
                new_state.updated_at = e.verified_at;
            }
            PersonEvent::PhoneAdded(e) => {
                new_state.phones.insert(e.phone.number.clone(), e.phone.clone());
                if e.primary {
                    new_state.primary_phone = Some(e.phone.number.clone());
                }
                new_state.updated_at = e.added_at;
            }
            PersonEvent::PhoneRemoved(e) => {
                new_state.phones.remove(&e.phone);
                if new_state.primary_phone.as_ref() == Some(&e.phone) {
                    new_state.primary_phone = None;
                }
                new_state.updated_at = e.removed_at;
            }
            PersonEvent::AddressAdded(e) => {
                new_state.addresses.insert(e.address_type.clone(), e.address.clone());
                new_state.updated_at = e.added_at;
            }
            PersonEvent::AddressRemoved(e) => {
                new_state.addresses.remove(&e.address_type);
                new_state.updated_at = e.removed_at;
            }
            
            // Employment events
            PersonEvent::EmploymentAdded(e) => {
                let org_id = e.employment.organization_id;
                new_state.employments.insert(org_id, e.employment.clone());
                if e.employment.end_date.is_none() {
                    new_state.current_employment = Some(org_id);
                }
                new_state.updated_at = e.added_at;
            }
            PersonEvent::EmploymentUpdated(e) => {
                if let Some(employment) = new_state.employments.get_mut(&e.organization_id) {
                    if let Some(dept) = &e.updates.department {
                        employment.department = Some(dept.clone());
                    }
                    if let Some(pos) = &e.updates.position {
                        employment.position = pos.clone();
                    }
                    if let Some(mgr) = &e.updates.manager_id {
                        employment.manager_id = Some(*mgr);
                    }
                }
                new_state.updated_at = e.updated_at;
            }
            PersonEvent::EmploymentEnded(e) => {
                if let Some(employment) = new_state.employments.get_mut(&e.organization_id) {
                    employment.end_date = Some(e.end_date);
                }
                if new_state.current_employment == Some(e.organization_id) {
                    new_state.current_employment = None;
                }
                new_state.updated_at = e.ended_at;
            }
            
            // Skills & Education events
            PersonEvent::SkillAdded(e) => {
                new_state.skills.insert(e.skill.name.clone(), e.skill.clone());
                new_state.updated_at = e.added_at;
            }
            PersonEvent::SkillUpdated(e) => {
                if let Some(skill) = new_state.skills.get_mut(&e.skill_name) {
                    skill.proficiency = e.proficiency.clone();
                    skill.last_used = e.last_used;
                }
                new_state.updated_at = e.updated_at;
            }
            PersonEvent::SkillRemoved(e) => {
                new_state.skills.remove(&e.skill_name);
                new_state.updated_at = e.removed_at;
            }
            PersonEvent::CertificationAdded(e) => {
                new_state.certifications.push(e.certification.clone());
                new_state.updated_at = e.added_at;
            }
            PersonEvent::EducationAdded(e) => {
                new_state.education.push(e.education.clone());
                new_state.updated_at = e.added_at;
            }
            
            // Relationship events
            PersonEvent::RelationshipAdded(e) => {
                new_state.relationships.insert(e.relationship.person_id, e.relationship.clone());
                new_state.updated_at = e.added_at;
            }
            PersonEvent::RelationshipUpdated(e) => {
                if let Some(rel) = new_state.relationships.get_mut(&e.related_person_id) {
                    rel.status = e.status.clone();
                    rel.notes = e.notes.clone();
                }
                new_state.updated_at = e.updated_at;
            }
            PersonEvent::RelationshipEnded(e) => {
                if let Some(rel) = new_state.relationships.get_mut(&e.related_person_id) {
                    rel.end_date = Some(e.end_date);
                    rel.status = RelationshipStatus::Inactive;
                }
                new_state.updated_at = e.ended_at;
            }
            
            // Social Media events
            PersonEvent::SocialProfileAdded(e) => {
                new_state.social_profiles.insert(e.profile.platform.clone(), e.profile.clone());
                new_state.updated_at = e.added_at;
            }
            PersonEvent::SocialProfileUpdated(e) => {
                if let Some(profile) = new_state.social_profiles.get_mut(&e.platform) {
                    if let Some(username) = &e.updates.username {
                        profile.username = username.clone();
                    }
                    if let Some(verified) = e.updates.verified {
                        profile.verified = verified;
                    }
                    if let Some(followers) = e.updates.follower_count {
                        profile.follower_count = Some(followers);
                    }
                    if let Some(engagement) = e.updates.engagement_rate {
                        profile.engagement_rate = Some(engagement);
                    }
                }
                new_state.updated_at = e.updated_at;
            }
            PersonEvent::SocialProfileRemoved(e) => {
                new_state.social_profiles.remove(&e.platform);
                new_state.updated_at = e.removed_at;
            }
            
            // Customer/Business events
            PersonEvent::CustomerSegmentSet(e) => {
                new_state.customer_segment = Some(e.segment.clone());
                new_state.updated_at = e.set_at;
            }
            PersonEvent::BehavioralDataUpdated(e) => {
                new_state.behavioral_data = Some(e.data.clone());
                new_state.updated_at = e.updated_at;
            }
            PersonEvent::CommunicationPreferencesSet(e) => {
                new_state.communication_preferences = Some(e.preferences.clone());
                new_state.updated_at = e.set_at;
            }
            PersonEvent::PrivacyPreferencesSet(e) => {
                new_state.privacy_preferences = Some(e.preferences.clone());
                new_state.updated_at = e.set_at;
            }
            
            // Tag & Metadata events
            PersonEvent::TagAdded(e) => {
                new_state.tags.push(e.tag.clone());
                new_state.updated_at = e.added_at;
            }
            PersonEvent::TagRemoved(e) => {
                new_state.tags.retain(|t| t.name != e.tag_name || t.category != e.category);
                new_state.updated_at = e.removed_at;
            }
            PersonEvent::CustomAttributeSet(e) => {
                new_state.custom_attributes.insert(e.attribute.name.clone(), e.attribute.clone());
                new_state.updated_at = e.set_at;
            }
            
            // Lifecycle events
            PersonEvent::PersonDeactivated(e) => {
                new_state.is_active = false;
                new_state.updated_at = e.deactivated_at;
            }
            PersonEvent::PersonReactivated(e) => {
                new_state.is_active = true;
                new_state.updated_at = e.reactivated_at;
            }
            PersonEvent::PersonMergedInto(e) => {
                new_state.merged_into = Some(e.merged_into_id);
                new_state.merge_status = MergeStatus::MergedInto(e.merged_into_id);
                new_state.updated_at = e.merged_at;
            }
            PersonEvent::PersonsMerged(_) => {
                // This event is for the target person, not the source
            }
        }
        
        new_state
    }

    // Command handlers
    
    fn handle_create_person(&mut self, cmd: CreatePerson) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::PersonCreated(PersonCreated {
            person_id: cmd.person_id,
            name: cmd.name,
            source: cmd.source,
            created_at: Utc::now(),
        })])
    }

    fn handle_update_name(&mut self, cmd: UpdateName) -> Result<Vec<PersonEvent>, String> {
        if !self.is_active {
            return Err("Cannot update inactive person".to_string());
        }
        
        Ok(vec![PersonEvent::NameUpdated(NameUpdated {
            person_id: self.id,
            old_name: self.name.clone(),
            new_name: cmd.name,
            reason: cmd.reason,
            updated_at: Utc::now(),
        })])
    }

    fn handle_add_email(&mut self, cmd: AddEmail) -> Result<Vec<PersonEvent>, String> {
        if self.emails.contains_key(&cmd.email.address) {
            return Err("Email already exists".to_string());
        }
        
        Ok(vec![PersonEvent::EmailAdded(EmailAdded {
            person_id: self.id,
            email: cmd.email,
            primary: cmd.primary,
            added_at: Utc::now(),
        })])
    }

    fn handle_remove_email(&mut self, cmd: RemoveEmail) -> Result<Vec<PersonEvent>, String> {
        if !self.emails.contains_key(&cmd.email) {
            return Err("Email not found".to_string());
        }
        
        Ok(vec![PersonEvent::EmailRemoved(EmailRemoved {
            person_id: self.id,
            email: cmd.email,
            removed_at: Utc::now(),
        })])
    }

    fn handle_verify_email(&mut self, cmd: VerifyEmail) -> Result<Vec<PersonEvent>, String> {
        if !self.emails.contains_key(&cmd.email) {
            return Err("Email not found".to_string());
        }
        
        // In a real system, verify the token
        Ok(vec![PersonEvent::EmailVerified(EmailVerified {
            person_id: self.id,
            email: cmd.email,
            verified_at: Utc::now(),
        })])
    }

    fn handle_add_phone(&mut self, cmd: AddPhone) -> Result<Vec<PersonEvent>, String> {
        if self.phones.contains_key(&cmd.phone.number) {
            return Err("Phone already exists".to_string());
        }
        
        Ok(vec![PersonEvent::PhoneAdded(PhoneAdded {
            person_id: self.id,
            phone: cmd.phone,
            primary: cmd.primary,
            added_at: Utc::now(),
        })])
    }

    fn handle_remove_phone(&mut self, cmd: RemovePhone) -> Result<Vec<PersonEvent>, String> {
        if !self.phones.contains_key(&cmd.phone) {
            return Err("Phone not found".to_string());
        }
        
        Ok(vec![PersonEvent::PhoneRemoved(PhoneRemoved {
            person_id: self.id,
            phone: cmd.phone,
            removed_at: Utc::now(),
        })])
    }

    fn handle_add_address(&mut self, cmd: AddAddress) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::AddressAdded(AddressAdded {
            person_id: self.id,
            address: cmd.address,
            address_type: cmd.address_type,
            added_at: Utc::now(),
        })])
    }

    fn handle_remove_address(&mut self, cmd: RemoveAddress) -> Result<Vec<PersonEvent>, String> {
        if !self.addresses.contains_key(&cmd.address_type) {
            return Err("Address not found".to_string());
        }
        
        Ok(vec![PersonEvent::AddressRemoved(AddressRemoved {
            person_id: self.id,
            address_type: cmd.address_type,
            removed_at: Utc::now(),
        })])
    }

    fn handle_add_employment(&mut self, cmd: AddEmployment) -> Result<Vec<PersonEvent>, String> {
        if self.employments.contains_key(&cmd.employment.organization_id) {
            return Err("Employment already exists for this organization".to_string());
        }
        
        Ok(vec![PersonEvent::EmploymentAdded(EmploymentAdded {
            person_id: self.id,
            employment: cmd.employment,
            added_at: Utc::now(),
        })])
    }

    fn handle_update_employment(&mut self, cmd: UpdateEmployment) -> Result<Vec<PersonEvent>, String> {
        if !self.employments.contains_key(&cmd.organization_id) {
            return Err("Employment not found".to_string());
        }
        
        Ok(vec![PersonEvent::EmploymentUpdated(EmploymentUpdated {
            person_id: self.id,
            organization_id: cmd.organization_id,
            updates: cmd.updates,
            updated_at: Utc::now(),
        })])
    }

    fn handle_end_employment(&mut self, cmd: EndEmployment) -> Result<Vec<PersonEvent>, String> {
        if !self.employments.contains_key(&cmd.organization_id) {
            return Err("Employment not found".to_string());
        }
        
        Ok(vec![PersonEvent::EmploymentEnded(EmploymentEnded {
            person_id: self.id,
            organization_id: cmd.organization_id,
            end_date: cmd.end_date,
            reason: cmd.reason,
            ended_at: Utc::now(),
        })])
    }

    fn handle_add_skill(&mut self, cmd: AddSkill) -> Result<Vec<PersonEvent>, String> {
        if self.skills.contains_key(&cmd.skill.name) {
            return Err("Skill already exists".to_string());
        }
        
        Ok(vec![PersonEvent::SkillAdded(SkillAdded {
            person_id: self.id,
            skill: cmd.skill,
            added_at: Utc::now(),
        })])
    }

    fn handle_update_skill(&mut self, cmd: UpdateSkill) -> Result<Vec<PersonEvent>, String> {
        if !self.skills.contains_key(&cmd.skill_name) {
            return Err("Skill not found".to_string());
        }
        
        Ok(vec![PersonEvent::SkillUpdated(SkillUpdated {
            person_id: self.id,
            skill_name: cmd.skill_name,
            proficiency: cmd.proficiency,
            last_used: cmd.last_used,
            updated_at: Utc::now(),
        })])
    }

    fn handle_remove_skill(&mut self, cmd: RemoveSkill) -> Result<Vec<PersonEvent>, String> {
        if !self.skills.contains_key(&cmd.skill_name) {
            return Err("Skill not found".to_string());
        }
        
        Ok(vec![PersonEvent::SkillRemoved(SkillRemoved {
            person_id: self.id,
            skill_name: cmd.skill_name,
            removed_at: Utc::now(),
        })])
    }

    fn handle_add_certification(&mut self, cmd: AddCertification) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::CertificationAdded(CertificationAdded {
            person_id: self.id,
            certification: cmd.certification,
            added_at: Utc::now(),
        })])
    }

    fn handle_add_education(&mut self, cmd: AddEducation) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::EducationAdded(EducationAdded {
            person_id: self.id,
            education: cmd.education,
            added_at: Utc::now(),
        })])
    }

    fn handle_add_relationship(&mut self, cmd: AddRelationship) -> Result<Vec<PersonEvent>, String> {
        if self.relationships.contains_key(&cmd.relationship.person_id) {
            return Err("Relationship already exists".to_string());
        }
        
        Ok(vec![PersonEvent::RelationshipAdded(RelationshipAdded {
            person_id: self.id,
            relationship: cmd.relationship,
            added_at: Utc::now(),
        })])
    }

    fn handle_update_relationship(&mut self, cmd: UpdateRelationship) -> Result<Vec<PersonEvent>, String> {
        if !self.relationships.contains_key(&cmd.related_person_id) {
            return Err("Relationship not found".to_string());
        }
        
        Ok(vec![PersonEvent::RelationshipUpdated(RelationshipUpdated {
            person_id: self.id,
            related_person_id: cmd.related_person_id,
            status: cmd.status,
            notes: cmd.notes,
            updated_at: Utc::now(),
        })])
    }

    fn handle_end_relationship(&mut self, cmd: EndRelationship) -> Result<Vec<PersonEvent>, String> {
        if !self.relationships.contains_key(&cmd.related_person_id) {
            return Err("Relationship not found".to_string());
        }
        
        Ok(vec![PersonEvent::RelationshipEnded(RelationshipEnded {
            person_id: self.id,
            related_person_id: cmd.related_person_id,
            end_date: cmd.end_date,
            reason: cmd.reason,
            ended_at: Utc::now(),
        })])
    }

    fn handle_add_social_profile(&mut self, cmd: AddSocialProfile) -> Result<Vec<PersonEvent>, String> {
        if self.social_profiles.contains_key(&cmd.profile.platform) {
            return Err("Social profile already exists for this platform".to_string());
        }
        
        Ok(vec![PersonEvent::SocialProfileAdded(SocialProfileAdded {
            person_id: self.id,
            profile: cmd.profile,
            added_at: Utc::now(),
        })])
    }

    fn handle_update_social_profile(&mut self, cmd: UpdateSocialProfile) -> Result<Vec<PersonEvent>, String> {
        if !self.social_profiles.contains_key(&cmd.platform) {
            return Err("Social profile not found".to_string());
        }
        
        Ok(vec![PersonEvent::SocialProfileUpdated(SocialProfileUpdated {
            person_id: self.id,
            platform: cmd.platform,
            updates: cmd.updates,
            updated_at: Utc::now(),
        })])
    }

    fn handle_remove_social_profile(&mut self, cmd: RemoveSocialProfile) -> Result<Vec<PersonEvent>, String> {
        if !self.social_profiles.contains_key(&cmd.platform) {
            return Err("Social profile not found".to_string());
        }
        
        Ok(vec![PersonEvent::SocialProfileRemoved(SocialProfileRemoved {
            person_id: self.id,
            platform: cmd.platform,
            removed_at: Utc::now(),
        })])
    }

    fn handle_set_customer_segment(&mut self, cmd: SetCustomerSegment) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::CustomerSegmentSet(CustomerSegmentSet {
            person_id: self.id,
            segment: cmd.segment,
            set_at: Utc::now(),
        })])
    }

    fn handle_update_behavioral_data(&mut self, cmd: UpdateBehavioralData) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::BehavioralDataUpdated(BehavioralDataUpdated {
            person_id: self.id,
            data: cmd.data,
            updated_at: Utc::now(),
        })])
    }

    fn handle_set_communication_preferences(&mut self, cmd: SetCommunicationPreferences) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::CommunicationPreferencesSet(CommunicationPreferencesSet {
            person_id: self.id,
            preferences: cmd.preferences,
            set_at: Utc::now(),
        })])
    }

    fn handle_set_privacy_preferences(&mut self, cmd: SetPrivacyPreferences) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::PrivacyPreferencesSet(PrivacyPreferencesSet {
            person_id: self.id,
            preferences: cmd.preferences,
            set_at: Utc::now(),
        })])
    }

    fn handle_add_tag(&mut self, cmd: AddTag) -> Result<Vec<PersonEvent>, String> {
        // Check for duplicate
        if self.tags.iter().any(|t| t.name == cmd.tag.name && t.category == cmd.tag.category) {
            return Err("Tag already exists".to_string());
        }
        
        Ok(vec![PersonEvent::TagAdded(TagAdded {
            person_id: self.id,
            tag: cmd.tag,
            added_at: Utc::now(),
        })])
    }

    fn handle_remove_tag(&mut self, cmd: RemoveTag) -> Result<Vec<PersonEvent>, String> {
        // Check if tag exists
        if !self.tags.iter().any(|t| t.name == cmd.tag_name && t.category == cmd.category) {
            return Err("Tag not found".to_string());
        }
        
        Ok(vec![PersonEvent::TagRemoved(TagRemoved {
            person_id: self.id,
            tag_name: cmd.tag_name,
            category: cmd.category,
            removed_at: Utc::now(),
        })])
    }

    fn handle_set_custom_attribute(&mut self, cmd: SetCustomAttribute) -> Result<Vec<PersonEvent>, String> {
        Ok(vec![PersonEvent::CustomAttributeSet(CustomAttributeSet {
            person_id: self.id,
            attribute: cmd.attribute,
            set_at: Utc::now(),
        })])
    }

    fn handle_deactivate(&mut self, cmd: DeactivatePerson) -> Result<Vec<PersonEvent>, String> {
        if !self.is_active {
            return Err("Person is already inactive".to_string());
        }
        
        Ok(vec![PersonEvent::PersonDeactivated(PersonDeactivated {
            person_id: self.id,
            reason: cmd.reason,
            deactivated_at: Utc::now(),
        })])
    }

    fn handle_reactivate(&mut self, cmd: ReactivatePerson) -> Result<Vec<PersonEvent>, String> {
        if self.is_active {
            return Err("Person is already active".to_string());
        }
        
        Ok(vec![PersonEvent::PersonReactivated(PersonReactivated {
            person_id: self.id,
            reason: cmd.reason,
            reactivated_at: Utc::now(),
        })])
    }

    fn handle_merge(&mut self, cmd: MergePersons) -> Result<Vec<PersonEvent>, String> {
        if cmd.source_person_id != self.id {
            return Err("Source person ID mismatch".to_string());
        }
        
        Ok(vec![
            PersonEvent::PersonMergedInto(PersonMergedInto {
                person_id: cmd.source_person_id,
                merged_into_id: cmd.target_person_id,
                merge_reason: cmd.merge_reason.clone(),
                merged_at: Utc::now(),
            }),
            PersonEvent::PersonsMerged(PersonsMerged {
                source_person_id: cmd.source_person_id,
                target_person_id: cmd.target_person_id,
                merge_reason: cmd.merge_reason,
                merged_at: Utc::now(),
            }),
        ])
    }
}

impl AggregateRoot for Person {
    type Id = PersonId;
    
    fn id(&self) -> Self::Id {
        self.id
    }
    
    fn version(&self) -> u64 {
        self.version
    }
    
    fn increment_version(&mut self) {
        // Version is not tracked in the new implementation
    }
} 