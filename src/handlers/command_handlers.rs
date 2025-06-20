//! Command handlers for person domain

use crate::{
    aggregate::Person,
    commands::PersonCommand,
    events::PersonEvent,
    value_objects::*,
};
use cim_domain::DomainResult;
use chrono::Utc;
use std::collections::HashMap;

/// Handle person commands
pub async fn handle_person_command(
    aggregate: &mut Person,
    command: PersonCommand,
) -> DomainResult<Vec<PersonEvent>> {
    match command {
        PersonCommand::RegisterPerson { person_id, identity, contact } => {
            // This would typically be handled at aggregate creation
            Ok(vec![PersonEvent::PersonRegistered {
                person_id,
                identity,
                contact,
                registered_at: Utc::now(),
            }])
        }

        PersonCommand::AddEmployment { person_id, employment } => {
            aggregate.add_component(employment.clone(), "system", Some("Employment added".to_string()))?;

            Ok(vec![PersonEvent::EmploymentAdded {
                person_id,
                employment,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::ChangeEmploymentStatus { person_id, organization_id, status, end_date } => {
            if let Some(mut employment) = aggregate.get_component::<EmploymentComponent>().cloned() {
                let old_status = employment.status.clone();
                employment.status = status.clone();
                employment.end_date = end_date;

                aggregate.remove_component::<EmploymentComponent>()?;
                aggregate.add_component(employment, "system", Some("Status update".to_string()))?;

                Ok(vec![PersonEvent::EmploymentStatusChanged {
                    person_id,
                    organization_id,
                    old_status,
                    new_status: status,
                    end_date,
                    changed_at: Utc::now(),
                }])
            } else {
                Err(cim_domain::DomainError::ValidationError(
                    "No employment found for organization".to_string()
                ))
            }
        }

        PersonCommand::AddPosition { person_id, position } => {
            aggregate.add_component(position.clone(), "system", Some("Position added".to_string()))?;

            Ok(vec![PersonEvent::PositionAdded {
                person_id,
                position,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::ChangeSkills { person_id, skills } => {
            // Generate remove/add event sequence
            let mut events = Vec::new();
            
            // If there are existing skills, remove them first
            if let Some(old_skills) = aggregate.get_component::<SkillsComponent>().cloned() {
                aggregate.remove_component::<SkillsComponent>().ok();
                events.push(PersonEvent::SkillsRemoved {
                    person_id,
                    old_skills,
                    removed_at: Utc::now(),
                });
            }
            
            // Add new skills
            aggregate.add_component(skills.clone(), "system", Some("Skills change".to_string()))?;
            events.push(PersonEvent::SkillsAdded {
                person_id,
                new_skills: skills,
                added_at: Utc::now(),
            });

            Ok(events)
        }

        PersonCommand::GrantAccess { person_id, access } => {
            aggregate.add_component(access.clone(), "system", Some("Access granted".to_string()))?;

            Ok(vec![PersonEvent::AccessGranted {
                person_id,
                access,
                granted_at: Utc::now(),
            }])
        }

        PersonCommand::AddExternalIdentifier { person_id, system, identifier } => {
            let mut external_ids = aggregate.get_component::<ExternalIdentifiersComponent>()
                .cloned()
                .unwrap_or_else(|| ExternalIdentifiersComponent {
                    ldap_dn: None,
                    ad_sid: None,
                    oauth_subjects: HashMap::new(),
                    external_ids: HashMap::new(),
                });

            external_ids.external_ids.insert(system.clone(), identifier.clone());

            aggregate.remove_component::<ExternalIdentifiersComponent>().ok();
            aggregate.add_component(external_ids, "system", Some("External ID added".to_string()))?;

            Ok(vec![PersonEvent::ExternalIdentifierAdded {
                person_id,
                system,
                identifier,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::ChangeContact { person_id, contact } => {
            // Generate remove/add event sequence
            let mut events = Vec::new();
            
            // If there's existing contact, remove it first
            if let Some(old_contact) = aggregate.get_component::<ContactComponent>().cloned() {
                aggregate.remove_component::<ContactComponent>().ok();
                events.push(PersonEvent::ContactRemoved {
                    person_id,
                    old_contact,
                    removed_at: Utc::now(),
                });
            }
            
            // Add new contact
            aggregate.add_component(contact.clone(), "system", Some("Contact change".to_string()))?;
            events.push(PersonEvent::ContactAdded {
                person_id,
                new_contact: contact,
                added_at: Utc::now(),
            });

            Ok(events)
        }

        // New CRM command handlers

        PersonCommand::UpdateName { person_id, name } => {
            let old_name = aggregate.get_component::<NameComponent>().cloned();
            
            if old_name.is_some() {
                aggregate.remove_component::<NameComponent>().ok();
            }
            
            aggregate.add_component(name.clone(), "system", Some("Name update".to_string()))?;

            Ok(vec![PersonEvent::NameUpdated {
                person_id,
                old_name,
                new_name: name,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::AddAlternativeNames { person_id, alternative_names } => {
            aggregate.add_component(alternative_names.clone(), "system", Some("Alternative names added".to_string()))?;

            Ok(vec![PersonEvent::AlternativeNamesAdded {
                person_id,
                alternative_names,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::UpdatePhysicalAttributes { person_id, attributes } => {
            let old_attributes = aggregate.get_component::<PhysicalAttributesComponent>().cloned();
            
            if old_attributes.is_some() {
                aggregate.remove_component::<PhysicalAttributesComponent>().ok();
            }
            
            aggregate.add_component(attributes.clone(), "system", Some("Physical attributes update".to_string()))?;

            Ok(vec![PersonEvent::PhysicalAttributesUpdated {
                person_id,
                old_attributes,
                new_attributes: attributes,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::AddDistinguishingMarks { person_id, marks } => {
            aggregate.add_component(marks.clone(), "system", Some("Distinguishing marks added".to_string()))?;

            Ok(vec![PersonEvent::DistinguishingMarksAdded {
                person_id,
                marks,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateMedicalIdentity { person_id, medical } => {
            let old_medical = aggregate.get_component::<MedicalIdentityComponent>().cloned();
            
            if old_medical.is_some() {
                aggregate.remove_component::<MedicalIdentityComponent>().ok();
            }
            
            aggregate.add_component(medical.clone(), "system", Some("Medical identity update".to_string()))?;

            Ok(vec![PersonEvent::MedicalIdentityUpdated {
                person_id,
                old_medical,
                new_medical: medical,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateRelationships { person_id, relationships } => {
            let old_relationships = aggregate.get_component::<RelationshipComponent>().cloned();
            
            if old_relationships.is_some() {
                aggregate.remove_component::<RelationshipComponent>().ok();
            }
            
            aggregate.add_component(relationships.clone(), "system", Some("Relationships update".to_string()))?;

            Ok(vec![PersonEvent::RelationshipsUpdated {
                person_id,
                old_relationships,
                new_relationships: relationships,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateSocialMedia { person_id, social_media } => {
            let old_social_media = aggregate.get_component::<SocialMediaComponent>().cloned();
            
            if old_social_media.is_some() {
                aggregate.remove_component::<SocialMediaComponent>().ok();
            }
            
            aggregate.add_component(social_media.clone(), "system", Some("Social media update".to_string()))?;

            Ok(vec![PersonEvent::SocialMediaUpdated {
                person_id,
                old_social_media,
                new_social_media: social_media,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateInterests { person_id, interests } => {
            let old_interests = aggregate.get_component::<InterestsComponent>().cloned();
            
            if old_interests.is_some() {
                aggregate.remove_component::<InterestsComponent>().ok();
            }
            
            aggregate.add_component(interests.clone(), "system", Some("Interests update".to_string()))?;

            Ok(vec![PersonEvent::InterestsUpdated {
                person_id,
                old_interests,
                new_interests: interests,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::UpdatePreferences { person_id, preferences } => {
            let old_preferences = aggregate.get_component::<PreferencesComponent>().cloned();
            
            if old_preferences.is_some() {
                aggregate.remove_component::<PreferencesComponent>().ok();
            }
            
            aggregate.add_component(preferences.clone(), "system", Some("Preferences update".to_string()))?;

            Ok(vec![PersonEvent::PreferencesUpdated {
                person_id,
                old_preferences,
                new_preferences: preferences,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateBehavioralData { person_id, behavioral } => {
            let old_behavioral = aggregate.get_component::<BehavioralComponent>().cloned();
            
            if old_behavioral.is_some() {
                aggregate.remove_component::<BehavioralComponent>().ok();
            }
            
            aggregate.add_component(behavioral.clone(), "system", Some("Behavioral data update".to_string()))?;

            Ok(vec![PersonEvent::BehavioralDataUpdated {
                person_id,
                old_behavioral,
                new_behavioral: behavioral,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::UpdateSegmentation { person_id, segmentation } => {
            let old_segmentation = aggregate.get_component::<SegmentationComponent>().cloned();
            
            if old_segmentation.is_some() {
                aggregate.remove_component::<SegmentationComponent>().ok();
            }
            
            aggregate.add_component(segmentation.clone(), "system", Some("Segmentation update".to_string()))?;

            Ok(vec![PersonEvent::SegmentationUpdated {
                person_id,
                old_segmentation,
                new_segmentation: segmentation,
                updated_at: Utc::now(),
            }])
        }

        PersonCommand::AddBiometricData { person_id, biometric } => {
            aggregate.add_component(biometric.clone(), "system", Some("Biometric data added".to_string()))?;

            Ok(vec![PersonEvent::BiometricDataAdded {
                person_id,
                biometric,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::AddComponent { person_id, component_type, component_data, added_by, reason } => {
            // This would need to handle dynamic component types
            // For now, we'll just emit the event
            Ok(vec![PersonEvent::ComponentAdded {
                person_id,
                component_type,
                component_data,
                added_by,
                reason,
                added_at: Utc::now(),
            }])
        }

        PersonCommand::RemoveComponent { person_id, component_type, removed_by, reason } => {
            // This would need to handle dynamic component removal
            // For now, we'll just emit the event
            Ok(vec![PersonEvent::ComponentRemoved {
                person_id,
                component_type,
                removed_by,
                reason,
                removed_at: Utc::now(),
            }])
        }
    }
}
