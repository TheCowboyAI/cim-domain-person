//! Component command handler implementation

use cim_domain::{DomainResult, DomainError};
use std::sync::Arc;
use chrono::Utc;
use tracing::{debug, info};

use crate::aggregate::PersonId;
use crate::commands::ComponentCommand;
use crate::events::{ComponentDataEvent, PersonEvent};
use crate::infrastructure::{EventStore, InMemoryComponentStore, PersonRepository, ComponentStore};
use crate::components::data::{
    ComponentInstance, ComponentInstanceId, EmailComponentData, PhoneComponentData,
    SkillComponentData, SocialMediaProfileData, EmploymentHistoryData,
    Endorsement, ComponentData, ContactData, ProfessionalData,
    SocialData, SkillsData,
    Skill,
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
        let _person = self.person_repository.load(person_id).await?
            .ok_or_else(|| DomainError::generic(format!("Person {person_id} not found")))?;
        
        debug!("Handling component command for person: {}", person_id);
        
        // Process command and generate events
        let events = match command {
            ComponentCommand::AddEmail { person_id, email, email_type, is_preferred, can_receive_notifications, can_receive_marketing } => {
                self.handle_add_email(person_id, email, email_type, is_preferred, can_receive_notifications, can_receive_marketing).await?
            }
            ComponentCommand::UpdateEmail { person_id, component_id, email, email_type, is_preferred, can_receive_notifications, can_receive_marketing } => {
                let changes = crate::events::EmailChanges {
                    email,
                    email_type,
                    is_preferred,
                    can_receive_notifications,
                    can_receive_marketing,
                };
                self.handle_update_email(person_id, component_id, changes).await?
            }
            ComponentCommand::RemoveEmail { person_id, component_id } => {
                self.handle_remove_email(person_id, component_id).await?
            }
            ComponentCommand::AddPhone { person_id, phone_number, phone_type, country_code, is_mobile, can_receive_sms, can_receive_calls } => {
                self.handle_add_phone(person_id, phone_number, phone_type, country_code, is_mobile, can_receive_sms, can_receive_calls).await?
            }
            ComponentCommand::UpdatePhone { person_id, component_id, phone_number, phone_type, can_receive_sms, can_receive_calls } => {
                let changes = crate::events::PhoneChanges {
                    phone_number,
                    phone_type,
                    can_receive_sms,
                    can_receive_calls,
                };
                self.handle_update_phone(person_id, component_id, changes).await?
            }
            ComponentCommand::RemovePhone { person_id, component_id } => {
                self.handle_remove_phone(person_id, component_id).await?
            }
            ComponentCommand::AddSkill { person_id, skill_name, category, proficiency, years_of_experience } => {
                self.handle_add_skill(person_id, skill_name, category, proficiency, years_of_experience).await?
            }
            ComponentCommand::UpdateSkill { person_id, component_id, proficiency, years_of_experience, last_used } => {
                let changes = crate::events::SkillChanges {
                    proficiency,
                    years_of_experience,
                    last_used,
                };
                self.handle_update_skill(person_id, component_id, changes).await?
            }
            ComponentCommand::EndorseSkill { person_id, component_id, endorser_id, endorser_name, comment } => {
                self.handle_endorse_skill(person_id, component_id, endorser_id, endorser_name, comment).await?
            }
            ComponentCommand::RemoveSkill { person_id, component_id } => {
                self.handle_remove_skill(person_id, component_id).await?
            }
            ComponentCommand::AddSocialProfile { person_id, platform, username, profile_url, display_name, bio } => {
                self.handle_add_social_profile(person_id, platform, username, profile_url, display_name, bio).await?
            }
            ComponentCommand::UpdateSocialProfile { person_id, component_id, username, profile_url, display_name, bio, is_verified } => {
                let changes = crate::events::SocialProfileChanges {
                    username,
                    profile_url,
                    display_name,
                    bio,
                    is_verified,
                };
                self.handle_update_social_profile(person_id, component_id, changes).await?
            }
            ComponentCommand::RemoveSocialProfile { person_id, component_id } => {
                self.handle_remove_social_profile(person_id, component_id).await?
            }
            ComponentCommand::AddEmployment { person_id, company_name, company_id, job_title, department, employment_type, start_date, is_current, location, remote_type } => {
                self.handle_add_employment(person_id, company_name, company_id, job_title, department, employment_type, start_date, is_current, location, remote_type).await?
            }
            ComponentCommand::UpdateEmployment { person_id, component_id, job_title, department, end_date, is_current, responsibilities, achievements } => {
                let changes = crate::events::EmploymentChanges {
                    job_title,
                    department,
                    is_current,
                    responsibilities,
                    achievements,
                };
                self.handle_update_employment(person_id, component_id, changes, end_date).await?
            }
            ComponentCommand::RemoveEmployment { person_id, component_id } => {
                self.handle_remove_employment(person_id, component_id).await?
            }
            ComponentCommand::UpdateCommunicationPreferences { person_id, preferred_language, preferred_channels, contact_frequency, email_format } => {
                let changes = crate::events::CommunicationPreferenceChanges {
                    preferred_language,
                    preferred_channels,
                    contact_frequency,
                    email_format,
                };
                self.handle_update_communication_preferences(person_id, changes).await?
            }
            ComponentCommand::UpdatePrivacyPreferences { person_id, allow_analytics, allow_marketing, allow_third_party_sharing, profile_visibility, data_retention_preference } => {
                let changes = crate::events::PrivacyPreferenceChanges {
                    allow_analytics,
                    allow_marketing,
                    allow_third_party_sharing,
                    profile_visibility,
                    data_retention_preference,
                };
                self.handle_update_privacy_preferences(person_id, changes).await?
            }
            ComponentCommand::RecordConsent { person_id, consent_type, granted, version, ip_address } => {
                self.handle_record_consent(person_id, consent_type, granted, version, ip_address).await?
            }
        };
        
        // Store events in the event store
        if !events.is_empty() {
            let person_events: Vec<PersonEvent> = events.iter()
                .map(|e| PersonEvent::ComponentDataUpdated(crate::events::ComponentDataUpdated {
                    person_id,
                    component_id: uuid::Uuid::new_v4(),
                    data: self.event_to_component_data(e),
                    updated_at: chrono::Utc::now(),
                }))
                .collect();
            
            self.event_store.append_events(person_id, person_events, None).await?;
            
            info!("Stored {} events for person {}", events.len(), person_id);
        }
        
        Ok(events)
    }
    
    /// Convert a ComponentDataEvent to ComponentData for storage
    fn event_to_component_data(&self, event: &ComponentDataEvent) -> crate::components::data::ComponentData {
        use crate::components::data::*;
        
        match event {
            ComponentDataEvent::EmailAdded { email, email_type, is_preferred, .. } => {
                ComponentData::Contact(ContactData::Email(EmailData {
                    email: EmailAddress::new(email.clone()).unwrap(),
                    email_type: email_type.clone(),
                    is_preferred_contact: *is_preferred,
                    can_receive_notifications: true,
                    can_receive_marketing: false,
                }))
            }
            ComponentDataEvent::PhoneAdded { phone_number, phone_type, country_code, is_mobile, .. } => {
                ComponentData::Contact(ContactData::Phone(PhoneData {
                    phone: PhoneNumber::new(phone_number.clone()).unwrap(),
                    phone_type: phone_type.clone(),
                    country_code: country_code.clone(),
                    is_mobile: *is_mobile,
                    can_receive_sms: true,
                    can_receive_calls: true,
                    preferred_contact_time: None,
                }))
            }
            ComponentDataEvent::SkillAdded { skill_name, category, proficiency, .. } => {
                ComponentData::Professional(ProfessionalData::Skills(SkillsData {
                    skills: vec![Skill {
                        name: skill_name.clone(),
                        category: format!("{:?}", category),
                        proficiency: format!("{proficiency:?}"),
                        years_experience: None,
                        last_used: None,
                        endorsement_count: None,
                        certifications: vec![],
                    }],
                    specializations: vec![],
                }))
            }
            ComponentDataEvent::SocialProfileAdded { platform, username, profile_url, .. } => {
                ComponentData::Social(SocialData::SocialMedia(SocialMediaData {
                    platform: platform.clone(),
                    username: username.clone(),
                    profile_url: profile_url.clone(),
                    display_name: None,
                    bio: None,
                    follower_count: None,
                    following_count: None,
                    is_verified: false,
                    is_business_account: false,
                    last_synced: None,
                }))
            }
            ComponentDataEvent::EmploymentAdded { company_name, job_title, employment_type, start_date, .. } => {
                ComponentData::Professional(ProfessionalData::Employment(EmploymentData {
                    company: company_name.clone(),
                    position: job_title.clone(),
                    start_date: start_date.date_naive(),
                    end_date: None,
                    employment_type: employment_type.clone(),
                    is_current: true,
                    description: None,
                    achievements: vec![],
                }))
            }
            _ => {
                // For other events, create a generic component data
                // In a real implementation, we'd handle all event types
                ComponentData::Contact(ContactData::Email(EmailData {
                    email: EmailAddress::new("placeholder@example.com".to_string()).unwrap(),
                    email_type: EmailType::Personal,
                    is_preferred_contact: false,
                    can_receive_notifications: false,
                    can_receive_marketing: false,
                }))
            }
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
    
    async fn handle_add_email(
        &self,
        person_id: PersonId,
        email: String,
        email_type: crate::components::data::EmailType,
        is_preferred: bool,
        can_receive_notifications: bool,
        can_receive_marketing: bool,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create email component data
        let email_data = EmailComponentData {
            email: EmailAddress::new(email.clone())
                .map_err(|e| DomainError::ValidationError(e))?,
            email_type: email_type.clone(),
            is_preferred_contact: is_preferred,
            can_receive_notifications,
            can_receive_marketing,
        };
        
        // Create component instance
        let component = ComponentInstance::new(person_id, email_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        Ok(vec![ComponentDataEvent::EmailAdded {
            person_id,
            component_id,
            email,
            email_type,
            is_preferred,
            timestamp: chrono::Utc::now(),
        }])
    }
    
    pub async fn handle_update_email(
        &self,
        person_id: PersonId,
        component_id: ComponentInstanceId,
        changes: crate::events::EmailChanges,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Get existing component
        let existing: Option<ComponentInstance<EmailComponentData>> = 
            self.component_store.get_component(component_id).await?;
        
        if existing.is_none() {
            return Err(DomainError::generic(format!("Email component {component_id} not found")));
        }
        
        let mut component = existing.unwrap();
        
        // Apply changes
        if let Some(new_email) = &changes.email {
            component.data.email = EmailAddress::new(new_email.clone())
                .map_err(|e| DomainError::ValidationError(e))?;
        }
        if let Some(new_type) = changes.email_type {
            component.data.email_type = new_type;
        }
        if let Some(pref) = changes.is_preferred {
            component.data.is_preferred_contact = pref;
        }
        if let Some(notif) = changes.can_receive_notifications {
            component.data.can_receive_notifications = notif;
        }
        if let Some(marketing) = changes.can_receive_marketing {
            component.data.can_receive_marketing = marketing;
        }
        
        // Update component
        let component_data = ComponentData::Contact(ContactData::Email(component.data.clone()));
        self.component_store.update_component(person_id, component_id, component_data).await?;
        
        // Create event
        let event = ComponentDataEvent::EmailUpdated {
            person_id,
            component_id,
            changes: changes.clone(),
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
    
    async fn handle_add_phone(
        &self,
        person_id: PersonId,
        phone_number: String,
        phone_type: crate::components::data::PhoneType,
        country_code: String,
        is_mobile: bool,
        can_receive_sms: bool,
        can_receive_calls: bool,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create phone component data
        let phone_data = PhoneComponentData {
            phone: PhoneNumber::new(phone_number.clone())
                .map_err(|e| DomainError::ValidationError(e))?,
            phone_type: phone_type.clone(),
            country_code: country_code.clone(),
            is_mobile,
            can_receive_sms,
            can_receive_calls,
            preferred_contact_time: None,
        };
        
        // Create component instance
        let component = ComponentInstance::new(person_id, phone_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        Ok(vec![ComponentDataEvent::PhoneAdded {
            person_id,
            component_id,
            phone_number,
            phone_type,
            country_code,
            is_mobile,
            timestamp: chrono::Utc::now(),
        }])
    }
    
    async fn handle_add_skill(
        &self,
        person_id: PersonId,
        skill_name: String,
        category: crate::components::data::SkillCategory,
        proficiency: crate::components::data::ProficiencyLevel,
        years_of_experience: Option<f32>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create skill component data
        let skill_data = SkillComponentData {
            name: skill_name.clone(),
            category: category.clone(),
            proficiency: proficiency.clone(),
            years_of_experience,
            last_used: None,
            verified: false,
            endorsements: vec![],
            certifications: vec![],
            projects: vec![],
        };
        
        // Create component instance
        let component = ComponentInstance::new(person_id, skill_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        Ok(vec![ComponentDataEvent::SkillAdded {
            person_id,
            component_id,
            skill_name,
            category,
            proficiency,
            timestamp: chrono::Utc::now(),
        }])
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
            return Err(DomainError::generic(format!("Skill component {component_id} not found")));
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
        let skills_data = SkillsData {
            skills: vec![Skill {
                name: component.data.name.clone(),
                category: format!("{:?}", component.data.category),
                proficiency: format!("{:?}", component.data.proficiency),
                years_experience: component.data.years_of_experience,
                last_used: component.data.last_used,
                endorsement_count: Some(component.data.endorsements.len()),
                certifications: component.data.certifications.clone(),
            }],
            specializations: vec![],
        };
        let component_data = ComponentData::Professional(ProfessionalData::Skills(skills_data));
        self.component_store.update_component(person_id, component_id, component_data).await?;
        
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
        // Create social profile component data
        let social_data = SocialMediaProfileData {
            platform: platform.clone(),
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
        
        // Create component instance
        let component = ComponentInstance::new(person_id, social_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        Ok(vec![ComponentDataEvent::SocialProfileAdded {
            person_id,
            component_id,
            platform,
            username,
            profile_url,
            timestamp: chrono::Utc::now(),
        }])
    }
    
    async fn handle_add_employment(
        &self,
        person_id: PersonId,
        company_name: String,
        _company_id: Option<String>,
        job_title: String,
        _department: Option<String>,
        employment_type: crate::components::data::EmploymentType,
        start_date: chrono::DateTime<chrono::Utc>,
        is_current: bool,
        _location: Option<String>,
        _remote_type: crate::components::data::RemoteType,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Create employment component data
        let employment_data = EmploymentHistoryData {
            company: company_name.clone(),
            position: job_title.clone(),
            start_date: start_date.date_naive(),
            end_date: if is_current { None } else { Some(chrono::Utc::now().date_naive()) },
            employment_type: employment_type.clone(),
            is_current,
            description: None,
            achievements: vec![],
        };
        
        // Create component instance
        let component = ComponentInstance::new(person_id, employment_data)?;
        let component_id = component.id;
        
        // Store component
        self.component_store.store_component(component).await?;
        
        Ok(vec![ComponentDataEvent::EmploymentAdded {
            person_id,
            component_id,
            company_name,
            job_title,
            employment_type,
            start_date,
            timestamp: chrono::Utc::now(),
        }])
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
    
    async fn handle_update_employment(
        &self,
        person_id: PersonId,
        component_id: crate::components::data::ComponentInstanceId,
        changes: crate::events::EmploymentChanges,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        let components = self.component_store.get_components(person_id).await?;
        
        for comp in components {
            if let ComponentData::Professional(ProfessionalData::Employment(mut employment)) = comp {
                // Check if this is the right component by checking some unique property
                // Since we don't have direct access to the component ID in the data,
                // we'll need to find another way or enhance the data structure
                
                // Apply changes
                if let Some(new_title) = &changes.job_title {
                    employment.position = new_title.clone();
                }
                if let Some(new_dept) = &changes.department {
                    employment.description = Some(new_dept.clone());
                }
                if let Some(current) = changes.is_current {
                    employment.is_current = current;
                }
                if let Some(achievements) = &changes.achievements {
                    employment.achievements = achievements.clone();
                }
                if let Some(end) = end_date {
                    employment.end_date = Some(end.date_naive());
                    employment.is_current = false;
                }
                
                // Update in store
                self.component_store.update_component(
                    person_id,
                    component_id,
                    ComponentData::Professional(ProfessionalData::Employment(employment)),
                ).await?;
                
                // If end_date was set, generate EmploymentEnded event
                if let Some(end) = end_date {
                    return Ok(vec![ComponentDataEvent::EmploymentEnded {
                        person_id,
                        component_id,
                        end_date: end,
                        timestamp: chrono::Utc::now(),
                    }]);
                } else {
                    return Ok(vec![ComponentDataEvent::EmploymentUpdated {
                        person_id,
                        component_id,
                        changes,
                        timestamp: chrono::Utc::now(),
                    }]);
                }
            }
        }
        
        Err(DomainError::generic(format!("Employment component {component_id} not found")))
    }
    
    async fn handle_update_phone(
        &self,
        person_id: PersonId,
        component_id: crate::components::data::ComponentInstanceId,
        changes: crate::events::PhoneChanges,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        let components = self.component_store.get_components(person_id).await?;
        
        for comp in components {
            if let ComponentData::Contact(ContactData::Phone(mut phone)) = comp {
                // Apply changes
                if let Some(new_number) = &changes.phone_number {
                    phone.phone = PhoneNumber::new(new_number.clone())
                        .map_err(|e| DomainError::ValidationError(e))?;
                }
                if let Some(new_type) = &changes.phone_type {
                    phone.phone_type = new_type.clone();
                }
                if let Some(sms) = changes.can_receive_sms {
                    phone.can_receive_sms = sms;
                }
                if let Some(calls) = changes.can_receive_calls {
                    phone.can_receive_calls = calls;
                }
                
                // Update in store
                self.component_store.update_component(
                    person_id,
                    component_id,
                    ComponentData::Contact(ContactData::Phone(phone)),
                ).await?;
                
                return Ok(vec![ComponentDataEvent::PhoneUpdated {
                    person_id,
                    component_id,
                    changes,
                    timestamp: chrono::Utc::now(),
                }]);
            }
        }
        
        Err(DomainError::generic(format!("Phone component {component_id} not found")))
    }
    
    async fn handle_remove_phone(
        &self,
        person_id: PersonId,
        component_id: crate::components::data::ComponentInstanceId,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        self.component_store.remove_component(person_id, component_id).await?;
        
        Ok(vec![ComponentDataEvent::PhoneRemoved {
            person_id,
            component_id,
            timestamp: chrono::Utc::now(),
        }])
    }
    
    async fn handle_update_skill(
        &self,
        person_id: PersonId,
        component_id: crate::components::data::ComponentInstanceId,
        changes: crate::events::SkillChanges,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // For skills, we need to get the existing component first
        let existing: Option<ComponentInstance<SkillComponentData>> = 
            self.component_store.get_component(component_id).await?;
        
        if let Some(mut component) = existing {
            // Apply changes
            if let Some(prof) = &changes.proficiency {
                component.data.proficiency = prof.clone();
            }
            if let Some(years) = changes.years_of_experience {
                component.data.years_of_experience = Some(years);
            }
            if let Some(last) = changes.last_used {
                component.data.last_used = Some(last);
            }
            
            // Convert to SkillsData for update
            let skills_data = SkillsData {
                skills: vec![Skill {
                    name: component.data.name.clone(),
                    category: format!("{:?}", component.data.category),
                    proficiency: format!("{:?}", component.data.proficiency),
                    years_experience: component.data.years_of_experience,
                    last_used: component.data.last_used,
                    endorsement_count: Some(component.data.endorsements.len()),
                    certifications: component.data.certifications.clone(),
                }],
                specializations: vec![],
            };
            
            // Update in store
            self.component_store.update_component(
                person_id,
                component_id,
                ComponentData::Professional(ProfessionalData::Skills(skills_data)),
            ).await?;
            
            return Ok(vec![ComponentDataEvent::SkillUpdated {
                person_id,
                component_id,
                changes,
                timestamp: chrono::Utc::now(),
            }]);
        }
        
        Err(DomainError::generic(format!("Skill component {component_id} not found")))
    }
    
    async fn handle_remove_skill(
        &self,
        person_id: PersonId,
        component_id: crate::components::data::ComponentInstanceId,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        self.component_store.remove_component(person_id, component_id).await?;
        
        Ok(vec![ComponentDataEvent::SkillRemoved {
            person_id,
            component_id,
            timestamp: chrono::Utc::now(),
        }])
    }
    
    async fn handle_update_social_profile(
        &self,
        person_id: PersonId,
        component_id: crate::components::data::ComponentInstanceId,
        changes: crate::events::SocialProfileChanges,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        let components = self.component_store.get_components(person_id).await?;
        
        for comp in components {
            if let ComponentData::Social(SocialData::SocialMedia(mut profile)) = comp {
                // Apply changes
                if let Some(username) = &changes.username {
                    profile.username = username.clone();
                }
                if let Some(url) = &changes.profile_url {
                    profile.profile_url = url.clone();
                }
                if let Some(display) = &changes.display_name {
                    profile.display_name = Some(display.clone());
                }
                if let Some(bio) = &changes.bio {
                    profile.bio = Some(bio.clone());
                }
                if let Some(verified) = changes.is_verified {
                    profile.is_verified = verified;
                }
                
                // Update in store
                self.component_store.update_component(
                    person_id,
                    component_id,
                    ComponentData::Social(SocialData::SocialMedia(profile)),
                ).await?;
                
                return Ok(vec![ComponentDataEvent::SocialProfileUpdated {
                    person_id,
                    component_id,
                    changes,
                    timestamp: chrono::Utc::now(),
                }]);
            }
        }
        
        Err(DomainError::generic(format!("Social profile component {component_id} not found")))
    }
    
    async fn handle_remove_social_profile(
        &self,
        person_id: PersonId,
        component_id: crate::components::data::ComponentInstanceId,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        self.component_store.remove_component(person_id, component_id).await?;
        
        Ok(vec![ComponentDataEvent::SocialProfileRemoved {
            person_id,
            component_id,
            timestamp: chrono::Utc::now(),
        }])
    }
    
    async fn handle_remove_employment(
        &self,
        person_id: PersonId,
        component_id: crate::components::data::ComponentInstanceId,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        self.component_store.remove_component(person_id, component_id).await?;
        
        Ok(vec![ComponentDataEvent::EmploymentRemoved {
            person_id,
            component_id,
            timestamp: chrono::Utc::now(),
        }])
    }
    
    async fn handle_update_communication_preferences(
        &self,
        person_id: PersonId,
        changes: crate::events::CommunicationPreferenceChanges,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Communication preferences are typically stored as a single component
        // For now, we'll just generate the event
        Ok(vec![ComponentDataEvent::CommunicationPreferencesUpdated {
            person_id,
            changes,
            timestamp: chrono::Utc::now(),
        }])
    }
    
    async fn handle_update_privacy_preferences(
        &self,
        person_id: PersonId,
        changes: crate::events::PrivacyPreferenceChanges,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Privacy preferences are typically stored as a single component
        // For now, we'll just generate the event
        Ok(vec![ComponentDataEvent::PrivacyPreferencesUpdated {
            person_id,
            changes,
            timestamp: chrono::Utc::now(),
        }])
    }
} 