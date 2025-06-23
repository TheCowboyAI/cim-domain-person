//! Query handlers for person domain

use crate::{
    aggregate::{PersonId, Person},
    projections::{PersonProjection, LdapProjection},
    queries::PersonQuery,
    services::{CustomerView, PartnerView, EmployeeView},
    value_objects::*,
};
use cim_domain::DomainResult;
use std::collections::HashMap;
use chrono::Datelike;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Person read model for queries
pub struct PersonReadModel {
    projections: HashMap<PersonId, PersonProjection>,
    // In a real system, these would be separate projections
    people: HashMap<PersonId, Person>,
}

impl Default for PersonReadModel {
    fn default() -> Self {
        Self::new()
    }
}

impl PersonReadModel {
    pub fn new() -> Self {
        Self {
            projections: HashMap::new(),
            people: HashMap::new(),
        }
    }

    /// Handle a person query
    pub async fn handle_query(&self, query: PersonQuery) -> DomainResult<PersonQueryResult> {
        match query {
            PersonQuery::GetPersonById { person_id } => {
                if let Some(projection) = self.projections.get(&person_id) {
                    Ok(PersonQueryResult::Person(projection.clone()))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::FindPersonByEmail { email } => {
                for projection in self.projections.values() {
                    if projection.emails.iter().any(|e| e.email == email) {
                        return Ok(PersonQueryResult::Person(projection.clone()));
                    }
                }
                Ok(PersonQueryResult::NotFound)
            }

            PersonQuery::ListActivePeople { limit, offset } => {
                let people: Vec<_> = self.projections.values()
                    .skip(offset)
                    .take(limit)
                    .cloned()
                    .collect();
                Ok(PersonQueryResult::People(people))
            }

            PersonQuery::SearchPeopleByName { name_pattern } => {
                let pattern = name_pattern.to_lowercase();
                let people: Vec<_> = self.projections.values()
                    .filter(|p| p.name.to_lowercase().contains(&pattern))
                    .cloned()
                    .collect();
                Ok(PersonQueryResult::People(people))
            }

            PersonQuery::GetEmployeeView { person_id } => {
                if let Some(person) = self.people.get(&person_id) {
                    let view = EmployeeView::from_person(person);
                    Ok(PersonQueryResult::EmployeeView(view))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::GetLdapProjection { person_id, base_dn } => {
                if let Some(projection) = self.projections.get(&person_id) {
                    let ldap = LdapProjection::from_projection(projection, &base_dn);
                    Ok(PersonQueryResult::LdapProjection(ldap))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::GetCustomerView { person_id } => {
                if let Some(person) = self.people.get(&person_id) {
                    let view = CustomerView::from_person(person);
                    Ok(PersonQueryResult::CustomerView(view))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::GetPartnerView { person_id } => {
                if let Some(person) = self.people.get(&person_id) {
                    let view = PartnerView::from_person(person);
                    Ok(PersonQueryResult::PartnerView(view))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::GetComponentHistory { person_id, component_type, since } => {
                // For now, just return the current components
                if let Some(person) = self.people.get(&person_id) {
                    let components = if let Some(ct) = component_type {
                        person.component_types().into_iter()
                            .filter(|t| t == &ct)
                            .collect()
                    } else {
                        person.component_types()
                    };
                    Ok(PersonQueryResult::Components(components))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }

            PersonQuery::FindPeopleByOrganization { organization_id, include_inactive } => {
                let employees = self.people.values()
                    .filter(|p| {
                        if let Some(emp) = p.get_component::<EmploymentComponent>() {
                            emp.organization_id == organization_id &&
                            (include_inactive || emp.status == "active")
                        } else {
                            false
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                Ok(PersonQueryResult::People(employees))
            }

            PersonQuery::FindPeopleByInterest { interest_category, interest_name } => {
                let people = self.people.values()
                    .filter(|p| {
                        if let Some(interests) = p.get_component::<InterestsComponent>() {
                            let category_key = match interest_category.as_str() {
                                "Sports" => InterestCategory::Sports,
                                "Technology" => InterestCategory::Technology,
                                "Arts" => InterestCategory::Arts,
                                "Travel" => InterestCategory::Travel,
                                _ => InterestCategory::Other(interest_category.clone()),
                            };
                            
                            if let Some(category_interests) = interests.interests.get(&category_key) {
                                interest_name.as_ref()
                                    .map(|name| {
                                        category_interests.iter().any(|interest| {
                                            interest.name.contains(name)
                                        })
                                    })
                                    .unwrap_or(true)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                Ok(PersonQueryResult::People(people))
            }

            PersonQuery::GetPeopleWithBirthdays { start_date, end_date } => {
                let people = self.people.values()
                    .filter(|p| {
                        if let Some(identity) = p.get_component::<IdentityComponent>() {
                            if let Some(dob) = identity.date_of_birth {
                                let birthday_in_range = dob.month() >= start_date.month() && 
                                    dob.day() >= start_date.day() &&
                                    dob.month() <= end_date.month() && 
                                    dob.day() <= end_date.day();
                                return birthday_in_range;
                            }
                        }
                        false
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                Ok(PersonQueryResult::People(people))
            }

            PersonQuery::FindPeopleBySocialMedia { platform, has_verified } => {
                let people = self.people.values()
                    .filter(|p| {
                        if let Some(social) = p.get_component::<SocialMediaComponent>() {
                            social.profiles.iter().any(|profile| {
                                let platform_matches = match &profile.platform {
                                    SocialPlatform::Other(s) => s == &platform,
                                    _ => format!("{:?}", profile.platform) == platform,
                                };
                                platform_matches &&
                                has_verified.map(|v| profile.verified == v).unwrap_or(true)
                            })
                        } else {
                            false
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                Ok(PersonQueryResult::People(people))
            }

            PersonQuery::SearchPeople { name, email, phone, organization, skills, segments, limit, offset } => {
                let mut results = Vec::new();

                for person in self.people.values() {
                    let mut matches = true;

                    // Check name
                    if let Some(name_pattern) = &name {
                        if let Some(projection) = self.projections.get(&person.id()) {
                            if !projection.name.to_lowercase().contains(&name_pattern.to_lowercase()) {
                                matches = false;
                            }
                        }
                    }

                    // Check email
                    if matches && email.is_some() {
                        if let Some(projection) = self.projections.get(&person.id()) {
                            if !projection.emails.iter().any(|e| &e.email == email.as_ref().unwrap()) {
                                matches = false;
                            }
                        }
                    }

                    // Check phone
                    if matches && phone.is_some() {
                        if let Some(projection) = self.projections.get(&person.id()) {
                            if !projection.phones.iter().any(|p| &p.number == phone.as_ref().unwrap()) {
                                matches = false;
                            }
                        }
                    }

                    // Check organization
                    if matches && organization.is_some() {
                        if let Some(projection) = self.projections.get(&person.id()) {
                            let org_matches = organization.as_ref()
                                .map(|o| {
                                    projection.organization_id.as_ref()
                                        .map(|po| po.to_string().contains(o))
                                        .unwrap_or(false)
                                })
                                .unwrap_or(false);
                            if !org_matches {
                                matches = false;
                            }
                        }
                    }

                    // Check skills
                    if matches && skills.is_some() {
                        if let Some(skills_comp) = person.get_component::<SkillsComponent>() {
                            let skill_matches = skills.as_ref().unwrap().iter()
                                .all(|skill| {
                                    skills_comp.skills.values()
                                        .any(|s| s.skill.to_lowercase().contains(&skill.to_lowercase()))
                                });
                            if !skill_matches {
                                matches = false;
                            }
                        } else if skills.is_some() {
                            matches = false;
                        }
                    }

                    // Check segments
                    if matches && segments.is_some() {
                        if let Some(seg) = person.get_component::<SegmentationComponent>() {
                            let seg_matches = segments.as_ref().unwrap().iter()
                                .any(|s| {
                                    (match &seg.primary_segment {
                                        CustomerSegment::Custom(cs) => cs == s,
                                        _ => format!("{:?}", seg.primary_segment) == *s,
                                    }) ||
                                    seg.secondary_segments.iter().any(|ss| {
                                        match ss {
                                            CustomerSegment::Custom(cs) => cs == s,
                                            _ => format!("{:?}", ss) == *s,
                                        }
                                    })
                                });
                            if !seg_matches {
                                matches = false;
                            }
                        } else if segments.is_some() {
                            matches = false;
                        }
                    }

                    if matches {
                        if let Some(projection) = self.projections.get(&person.id()) {
                            results.push(projection.clone());
                        }
                    }
                }

                // Apply pagination
                let paginated: Vec<_> = results.into_iter()
                    .skip(offset)
                    .take(limit)
                    .collect();

                Ok(PersonQueryResult::People(paginated))
            }

            PersonQuery::FindPeopleBySkill { skill_name, min_proficiency } => {
                let people = self.people.values()
                    .filter(|p| {
                        if let Some(skills) = p.get_component::<SkillsComponent>() {
                            skills.skills.values().any(|s| {
                                s.skill.to_lowercase() == skill_name.to_lowercase() &&
                                min_proficiency.as_ref().map(|ml| &s.level >= ml).unwrap_or(true)
                            })
                        } else {
                            false
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                Ok(PersonQueryResult::People(people))
            }

            // Find people by role
            PersonQuery::FindPeopleByRole { role } => {
                let people = self.projections.values()
                    .filter(|p| p.roles.contains(&role))
                    .cloned()
                    .collect();
                    
                Ok(PersonQueryResult::People(people))
            }
            
            // Find customers by segment
            PersonQuery::FindCustomersBySegment { segment, sub_segment } => {
                let people = self.people.values()
                    .filter(|p| {
                        if let Some(seg) = p.get_component::<SegmentationComponent>() {
                            let primary_matches = match &seg.primary_segment {
                                CustomerSegment::Custom(cs) => cs == &segment,
                                _ => format!("{:?}", seg.primary_segment) == segment,
                            };
                            
                            let sub_matches = sub_segment.as_ref()
                                .map(|sub| {
                                    seg.secondary_segments.iter().any(|ss| {
                                        match ss {
                                            CustomerSegment::Custom(cs) => cs == sub,
                                            _ => format!("{:?}", ss) == *sub,
                                        }
                                    })
                                })
                                .unwrap_or(true);
                                
                            primary_matches && sub_matches
                        } else {
                            false
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                    
                Ok(PersonQueryResult::People(people))
            }
            
            // Find customers by preferences
            PersonQuery::FindCustomersByPreferences { 
                preference_category, 
                preference_value 
            } => {
                let people = self.people.values()
                    .filter(|p| {
                        if let Some(prefs) = p.get_component::<PreferencesComponent>() {
                            // Check different preference categories
                            match preference_category.as_str() {
                                "communication_channel" => {
                                    format!("{:?}", prefs.communication.preferred_channel) == preference_value
                                }
                                "language" => {
                                    prefs.communication.preferred_language == preference_value
                                }
                                "frequency" => {
                                    format!("{:?}", prefs.communication.frequency_preference) == preference_value
                                }
                                "content_type" => {
                                    prefs.content_preferences.content_types.iter()
                                        .any(|ct| format!("{:?}", ct) == preference_value)
                                }
                                "privacy" => {
                                    match preference_value.as_str() {
                                        "data_sharing" => prefs.privacy_preferences.data_sharing_allowed,
                                        "analytics" => prefs.privacy_preferences.analytics_allowed,
                                        "personalization" => prefs.privacy_preferences.personalization_allowed,
                                        _ => false,
                                    }
                                }
                                _ => false,
                            }
                        } else {
                            false
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                    
                Ok(PersonQueryResult::People(people))
            }
            
            // Find customers by behavior
            PersonQuery::FindCustomersByBehavior { pattern, threshold } => {
                let people = self.people.values()
                    .filter(|p| {
                        if let Some(behavioral) = p.get_component::<BehavioralComponent>() {
                            match pattern.as_str() {
                                "purchase_frequency" => {
                                    behavioral.purchase_behavior.purchase_frequency
                                        .map(|freq| freq >= threshold)
                                        .unwrap_or(false)
                                }
                                "average_order_value" => {
                                    behavioral.purchase_behavior.average_order_value
                                        .map(|aov| aov >= threshold as f64)
                                        .unwrap_or(false)
                                }
                                "engagement_score" => {
                                    behavioral.engagement_patterns.email_open_rate
                                        .map(|rate| rate >= threshold)
                                        .unwrap_or(false)
                                }
                                _ => false,
                            }
                        } else {
                            false
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                    
                Ok(PersonQueryResult::People(people))
            }
            

            
            // Find people by relationship
            PersonQuery::FindPeopleByRelationship { relationship_type, related_person_id } => {
                let people = self.people.values()
                    .filter(|p| {
                        if let Some(relationships) = p.get_component::<RelationshipComponent>() {
                            relationships.relationships.iter().any(|r| {
                                format!("{:?}", r.relationship_type) == relationship_type &&
                                related_person_id.map(|id| r.person_id == Uuid::from(id)).unwrap_or(true)
                            })
                        } else {
                            false
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                    
                Ok(PersonQueryResult::People(people))
            }
            
            // Find people with components
            PersonQuery::FindPeopleWithComponents { component_types, match_all } => {
                let people = self.people.values()
                    .filter(|p| {
                        let person_components = p.component_types();
                        if match_all {
                            component_types.iter().all(|ct| person_components.contains(ct))
                        } else {
                            component_types.iter().any(|ct| person_components.contains(ct))
                        }
                    })
                    .map(|p| self.projections.get(&p.id()).cloned())
                    .filter_map(|opt| opt)
                    .collect();
                    
                Ok(PersonQueryResult::People(people))
            }
            
            // Get full profile
            PersonQuery::GetFullProfile { person_id, include_history } => {
                if let Some(projection) = self.projections.get(&person_id) {
                    Ok(PersonQueryResult::Person(projection.clone()))
                } else {
                    Ok(PersonQueryResult::NotFound)
                }
            }
        }
    }
}

/// Query result types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonQueryResult {
    /// Single person projection
    Person(PersonProjection),
    
    /// Multiple people
    People(Vec<PersonProjection>),
    
    /// Employee view
    EmployeeView(EmployeeView),
    
    /// Customer view
    CustomerView(CustomerView),
    
    /// Partner view
    PartnerView(PartnerView),
    
    /// LDAP projection
    LdapProjection(LdapProjection),
    
    /// Component list
    Components(Vec<String>),
    
    /// Not found
    NotFound,
}

/// Alias for compatibility with examples
pub type PersonQueryHandler = PersonReadModel;
