//! Component command handler

use cim_domain::{DomainResult, DomainError};
use std::sync::Arc;
use chrono::Utc;

use crate::aggregate::PersonId;
use crate::commands::ComponentCommand;
use crate::events::ComponentDataEvent;
use crate::infrastructure::{EventStore, InMemoryComponentStore, PersonRepository, ComponentStore};
use crate::components::data::{
    ComponentInstance, ComponentInstanceId, EmailComponentData, PhoneComponentData,
    SkillComponentData, SocialMediaProfileData, EmploymentHistoryData,
    EmailType, PhoneType, Endorsement,
};
use crate::value_objects::{EmailAddress, PhoneNumber};

/// Handler for component commands
pub struct ComponentCommandHandler {
    event_store: Arc<dyn EventStore>,
    component_store: Arc<InMemoryComponentStore>,
    person_repository: Arc<PersonRepository>,
}

impl ComponentCommandHandler {
    pub fn new(
        event_store: Arc<dyn EventStore>,
        component_store: Arc<InMemoryComponentStore>,
        person_repository: Arc<PersonRepository>,
    ) -> Self {
        Self {
            event_store,
            component_store,
            person_repository,
        }
    }
    
    /// Handle a component command
    pub async fn handle(&self, command: ComponentCommand) -> DomainResult<Vec<ComponentDataEvent>> {
        // Verify person exists
        let person_id = self.get_person_id(&command)?;
        let person = self.person_repository.load(person_id).await?;
        
        if person.is_none() {
            return Err(DomainError::AggregateNotFound(format!("Person {}", person_id)));
        }
        
        // Process command
        match command {
            ComponentCommand::AddEmail { person_id, email, email_type, is_preferred, can_receive_notifications, can_receive_marketing } => {
                self.handle_add_email(person_id, email, email_type, is_preferred, can_receive_notifications, can_receive_marketing).await
            }
            ComponentCommand::UpdateEmail { person_id, component_id, email, email_type, is_preferred, can_receive_notifications, can_receive_marketing } => {
                self.handle_update_email(person_id, component_id, email, email_type, is_preferred, can_receive_notifications, can_receive_marketing).await
            }
            ComponentCommand::RemoveEmail { person_id, component_id } => {
                self.handle_remove_email(person_id, component_id).await
            }
            ComponentCommand::AddPhone { person_id, phone_number, phone_type, country_code, is_mobile, can_receive_sms, can_receive_calls } => {
                self.handle_add_phone(person_id, phone_number, phone_type, country_code, is_mobile, can_receive_sms, can_receive_calls).await
            }
            ComponentCommand::AddSkill { person_id, skill_name, category, proficiency, years_of_experience } => {
                self.handle_add_skill(person_id, skill_name, category, proficiency, years_of_experience).await
            }
            ComponentCommand::EndorseSkill { person_id, component_id, endorser_id, endorser_name, comment } => {
                self.handle_endorse_skill(person_id, component_id, endorser_id, endorser_name, comment).await
            }
            ComponentCommand::AddSocialProfile { person_id, platform, username, profile_url, display_name, bio } => {
                self.handle_add_social_profile(person_id, platform, username, profile_url, display_name, bio).await
            }
            ComponentCommand::AddEmployment { person_id, company_name, company_id, job_title, department, employment_type, start_date, is_current: _, location, remote_type } => {
                self.handle_add_employment(person_id, company_name, company_id, job_title, department, start_date, employment_type, location, remote_type).await
            }
            ComponentCommand::RecordConsent { person_id, consent_type, granted, version, ip_address } => {
                self.handle_record_consent(person_id, consent_type, granted, version, ip_address).await
            }
            _ => Err(DomainError::generic("Component command not yet implemented")),
        }
    }
    
    fn get_person_id(&self, command: &ComponentCommand) -> DomainResult<PersonId> {
        match command {
            ComponentCommand::AddEmail { person_id, .. } |
            ComponentCommand::UpdateEmail { person_id, .. } |
            ComponentCommand::RemoveEmail { person_id, .. } |
            ComponentCommand::AddPhone { person_id, .. } |
            ComponentCommand::UpdatePhone { person_id, .. } |
            ComponentCommand::RemovePhone { person_id, .. } |
            ComponentCommand::AddSkill { person_id, .. } |
            ComponentCommand::UpdateSkill { person_id, .. } |
            ComponentCommand::EndorseSkill { person_id, .. } |
            ComponentCommand::RemoveSkill { person_id, .. } |
            ComponentCommand::AddSocialProfile { person_id, .. } |
            ComponentCommand::UpdateSocialProfile { person_id, .. } |
            ComponentCommand::RemoveSocialProfile { person_id, .. } |
            ComponentCommand::AddEmployment { person_id, .. } |
            ComponentCommand::UpdateEmployment { person_id, .. } |
            ComponentCommand::RemoveEmployment { person_id, .. } |
            ComponentCommand::UpdateCommunicationPreferences { person_id, .. } |
            ComponentCommand::UpdatePrivacyPreferences { person_id, .. } |
            ComponentCommand::RecordConsent { person_id, .. } => Ok(*person_id),
        }
    }
    
    pub async fn handle_add_email(
        &self,
        person_id: PersonId,
        email: String,
        email_type: EmailType,
        is_preferred: bool,
        can_receive_notifications: bool,
        can_receive_marketing: bool,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create email component
        let email_address = EmailAddress::new(email.clone())
            .map_err(|e| DomainError::ValidationError(e))?;
        let email_data = EmailComponentData {
            email: email_address,
            email_type,
            is_preferred_contact: is_preferred,
            can_receive_notifications,
            can_receive_marketing,
        };
        
        let component = ComponentInstance::new(person_id, email_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        // Create event
        let event = ComponentDataEvent::EmailAdded {
            person_id,
            component_id,
            email,
            email_type,
            is_preferred,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    pub async fn handle_update_email(
        &self,
        person_id: PersonId,
        component_id: ComponentInstanceId,
        email: Option<String>,
        email_type: Option<EmailType>,
        is_preferred: Option<bool>,
        can_receive_notifications: Option<bool>,
        can_receive_marketing: Option<bool>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Get existing component
        let existing: Option<ComponentInstance<EmailComponentData>> = 
            self.component_store.get_component(component_id).await?;
        
        if existing.is_none() {
            return Err(DomainError::generic(format!("Email component {} not found", component_id)));
        }
        
        let mut component = existing.unwrap();
        
        // Apply changes
        if let Some(new_email) = &email {
            component.data.email = EmailAddress::new(new_email.clone())
                .map_err(|e| DomainError::ValidationError(e))?;
        }
        if let Some(new_type) = email_type {
            component.data.email_type = new_type;
        }
        if let Some(pref) = is_preferred {
            component.data.is_preferred_contact = pref;
        }
        if let Some(notif) = can_receive_notifications {
            component.data.can_receive_notifications = notif;
        }
        if let Some(marketing) = can_receive_marketing {
            component.data.can_receive_marketing = marketing;
        }
        
        // Update component
        self.component_store.update_component(component).await?;
        
        // Create event
        let event = ComponentDataEvent::EmailUpdated {
            person_id,
            component_id,
            changes: crate::events::EmailChanges {
                email,
                email_type,
                is_preferred,
                can_receive_notifications,
                can_receive_marketing,
            },
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    pub async fn handle_remove_email(
        &self,
        person_id: PersonId,
        component_id: ComponentInstanceId,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Delete component
        self.component_store.delete_component(component_id).await?;
        
        // Create event
        let event = ComponentDataEvent::EmailRemoved {
            person_id,
            component_id,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    pub async fn handle_add_phone(
        &self,
        person_id: PersonId,
        phone_number: String,
        phone_type: PhoneType,
        country_code: String,
        is_mobile: bool,
        can_receive_sms: bool,
        can_receive_calls: bool,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create phone component
        let phone = PhoneNumber::new(phone_number.clone())
            .map_err(|e| DomainError::ValidationError(e))?;
        let phone_data = PhoneComponentData {
            phone,
            phone_type,
            country_code: country_code.clone(),
            is_mobile,
            can_receive_sms,
            can_receive_calls,
            preferred_contact_time: None,
        };
        
        let component = ComponentInstance::new(person_id, phone_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        // Create event
        let event = ComponentDataEvent::PhoneAdded {
            person_id,
            component_id,
            phone_number,
            phone_type,
            country_code,
            is_mobile,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    pub async fn handle_add_skill(
        &self,
        person_id: PersonId,
        skill_name: String,
        category: crate::components::data::SkillCategory,
        proficiency: crate::components::data::ProficiencyLevel,
        years_of_experience: Option<f32>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create skill component
        let skill_data = SkillComponentData {
            name: skill_name.clone(),
            category,
            proficiency,
            years_of_experience,
            last_used: None,
            verified: false,
            endorsements: Vec::new(),
            certifications: Vec::new(),
            projects: Vec::new(),
        };
        
        let component = ComponentInstance::new(person_id, skill_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        // Create event
        let event = ComponentDataEvent::SkillAdded {
            person_id,
            component_id,
            skill_name,
            category,
            proficiency,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    async fn handle_endorse_skill(
        &self,
        person_id: PersonId,
        component_id: ComponentInstanceId,
        endorser_id: Option<PersonId>,
        endorser_name: String,
        comment: Option<String>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Get existing skill
        let existing: Option<ComponentInstance<SkillComponentData>> = 
            self.component_store.get_component(component_id).await?;
        
        if existing.is_none() {
            return Err(DomainError::generic(format!("Skill component {} not found", component_id)));
        }
        
        let mut component = existing.unwrap();
        
        // Add endorsement
        component.data.endorsements.push(Endorsement {
            endorser_id,
            endorser_name: endorser_name.clone(),
            date: Utc::now(),
            comment: comment.clone(),
        });
        
        // Update component
        self.component_store.update_component(component).await?;
        
        // Create event
        let event = ComponentDataEvent::SkillEndorsed {
            person_id,
            component_id,
            endorser_id,
            endorser_name,
            comment,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    async fn handle_add_social_profile(
        &self,
        person_id: PersonId,
        platform: crate::components::data::SocialPlatform,
        username: String,
        profile_url: String,
        display_name: Option<String>,
        bio: Option<String>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create social profile component
        let profile_data = SocialMediaProfileData {
            platform,
            username: username.clone(),
            profile_url: profile_url.clone(),
            display_name,
            bio,
            follower_count: None,
            following_count: None,
            is_verified: false,
            is_business_account: false,
            last_synced: None,
        };
        
        let component = ComponentInstance::new(person_id, profile_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        // Create event
        let event = ComponentDataEvent::SocialProfileAdded {
            person_id,
            component_id,
            platform,
            username,
            profile_url,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    async fn handle_add_employment(
        &self,
        person_id: PersonId,
        company_name: String,
        _company_id: Option<String>,
        job_title: String,
        _department: Option<String>,
        start_date: chrono::DateTime<chrono::Utc>,
        employment_type: crate::components::data::EmploymentType,
        _location: Option<String>,
        _remote_type: crate::components::data::RemoteType,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create employment component
        let employment_data = EmploymentHistoryData {
            company: company_name.clone(),
            position: job_title.clone(),
            start_date: start_date.date_naive(),
            end_date: None,
            employment_type,
            is_current: true,
            description: None,
            achievements: Vec::new(),
        };
        
        let component = ComponentInstance::new(person_id, employment_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        // Create event
        let event = ComponentDataEvent::EmploymentAdded {
            person_id,
            component_id,
            company_name: company_name,
            job_title: job_title,
            employment_type,
            start_date,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
    
    async fn handle_record_consent(
        &self,
        person_id: PersonId,
        consent_type: String,
        granted: bool,
        version: String,
        ip_address: Option<String>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // For consent, we update privacy preferences
        // This would typically involve getting existing preferences and updating them
        
        // Create event
        let event = ComponentDataEvent::ConsentRecorded {
            person_id,
            consent_type,
            granted,
            version,
            ip_address,
            timestamp: Utc::now(),
        };
        
        Ok(vec![event])
    }
} 