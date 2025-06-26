//! Person skills projection for skill-based analytics and recommendations

use super::{PersonProjection, SkillSummary};
use crate::aggregate::PersonId;
use crate::events::*;
use crate::components::data::{ComponentData, ProfessionalData, Skill};
use cim_domain::DomainResult;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Skill profile for a person
#[derive(Debug, Clone)]
struct PersonSkillProfile {
    person_id: PersonId,
    skills: HashMap<String, SkillInfo>,
    skill_categories: HashMap<String, HashSet<String>>,
    last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
struct SkillInfo {
    skill: Skill,
    added_at: DateTime<Utc>,
    sources: HashSet<String>, // Where the skill came from (manual, git, etc.)
}

/// Global skill statistics
#[derive(Debug, Clone, Default)]
struct SkillStatistics {
    skill_counts: HashMap<String, usize>, // How many people have each skill
    skill_categories: HashMap<String, HashSet<String>>, // Skills by category
    skill_relationships: HashMap<String, HashSet<String>>, // Skills that often appear together
}

/// Projection that maintains skill profiles and analytics
pub struct PersonSkillsProjection {
    profiles: Arc<RwLock<HashMap<PersonId, PersonSkillProfile>>>,
    statistics: Arc<RwLock<SkillStatistics>>,
}

impl PersonSkillsProjection {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(SkillStatistics::default())),
        }
    }
    
    /// Get skills for a person
    pub async fn get_person_skills(&self, person_id: &PersonId) -> Vec<SkillSummary> {
        let profiles = self.profiles.read().await;
        
        if let Some(profile) = profiles.get(person_id) {
            profile.skills.values()
                .map(|info| SkillSummary {
                    skill_name: info.skill.name.clone(),
                    category: info.skill.category.clone(),
                    proficiency: info.skill.proficiency.clone(),
                    years_experience: info.skill.years_experience,
                    last_used: info.skill.last_used,
                    endorsement_count: info.skill.endorsement_count.unwrap_or(0),
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Find people with a specific skill
    pub async fn find_people_with_skill(&self, skill_name: &str) -> Vec<PersonId> {
        let profiles = self.profiles.read().await;
        let skill_lower = skill_name.to_lowercase();
        
        profiles.values()
            .filter(|profile| {
                profile.skills.keys().any(|s| s.to_lowercase() == skill_lower)
            })
            .map(|profile| profile.person_id)
            .collect()
    }
    
    /// Find people with multiple skills
    pub async fn find_people_with_skills(&self, required_skills: &[String]) -> Vec<PersonId> {
        let profiles = self.profiles.read().await;
        let required_lower: HashSet<String> = required_skills.iter()
            .map(|s| s.to_lowercase())
            .collect();
        
        profiles.values()
            .filter(|profile| {
                let person_skills: HashSet<String> = profile.skills.keys()
                    .map(|s| s.to_lowercase())
                    .collect();
                required_lower.is_subset(&person_skills)
            })
            .map(|profile| profile.person_id)
            .collect()
    }
    
    /// Get skill recommendations based on existing skills
    pub async fn get_skill_recommendations(&self, person_id: &PersonId, limit: usize) -> Vec<String> {
        let profiles = self.profiles.read().await;
        let statistics = self.statistics.read().await;
        
        if let Some(profile) = profiles.get(person_id) {
            let person_skills: HashSet<&String> = profile.skills.keys().collect();
            let mut recommendations: HashMap<String, usize> = HashMap::new();
            
            // Find skills that often appear with the person's current skills
            for skill in &person_skills {
                if let Some(related) = statistics.skill_relationships.get(*skill) {
                    for related_skill in related {
                        if !person_skills.contains(related_skill) {
                            *recommendations.entry(related_skill.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
            
            // Sort by frequency and take top N
            let mut rec_vec: Vec<_> = recommendations.into_iter().collect();
            rec_vec.sort_by(|a, b| b.1.cmp(&a.1));
            rec_vec.into_iter()
                .take(limit)
                .map(|(skill, _)| skill)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get skill statistics
    pub async fn get_skill_statistics(&self) -> HashMap<String, usize> {
        let statistics = self.statistics.read().await;
        statistics.skill_counts.clone()
    }
    
    /// Get skills by category
    pub async fn get_skills_by_category(&self, category: &str) -> Vec<String> {
        let statistics = self.statistics.read().await;
        statistics.skill_categories
            .get(category)
            .map(|skills| skills.iter().cloned().collect())
            .unwrap_or_else(Vec::new)
    }
    
    /// Get all skill categories
    pub async fn get_skill_categories(&self) -> Vec<String> {
        let statistics = self.statistics.read().await;
        let mut categories: Vec<_> = statistics.skill_categories.keys().cloned().collect();
        categories.sort();
        categories
    }
    
    /// Update skill relationships based on co-occurrence
    async fn update_skill_relationships(&self, person_skills: &[String]) {
        let mut statistics = self.statistics.write().await;
        
        // Update relationships for all pairs of skills
        for i in 0..person_skills.len() {
            for j in (i + 1)..person_skills.len() {
                let skill1 = &person_skills[i];
                let skill2 = &person_skills[j];
                
                statistics.skill_relationships
                    .entry(skill1.clone())
                    .or_insert_with(HashSet::new)
                    .insert(skill2.clone());
                
                statistics.skill_relationships
                    .entry(skill2.clone())
                    .or_insert_with(HashSet::new)
                    .insert(skill1.clone());
            }
        }
    }
}

#[async_trait::async_trait]
impl PersonProjection for PersonSkillsProjection {
    async fn handle_event(&self, event: &PersonEvent) -> DomainResult<()> {
        match event {
            PersonEvent::PersonCreated(e) => {
                let profile = PersonSkillProfile {
                    person_id: e.person_id,
                    skills: HashMap::new(),
                    skill_categories: HashMap::new(),
                    last_updated: e.created_at,
                };
                
                let mut profiles = self.profiles.write().await;
                profiles.insert(e.person_id, profile);
            }
            
            PersonEvent::ComponentDataUpdated(e) => {
                if let ComponentData::Professional(ProfessionalData::Skills(skills_data)) = &e.data {
                    let mut profiles = self.profiles.write().await;
                    let mut statistics = self.statistics.write().await;
                    
                    if let Some(profile) = profiles.get_mut(&e.person_id) {
                        // Remove old skills from statistics
                        for skill_name in profile.skills.keys() {
                            if let Some(count) = statistics.skill_counts.get_mut(skill_name) {
                                *count = count.saturating_sub(1);
                                if *count == 0 {
                                    statistics.skill_counts.remove(skill_name);
                                }
                            }
                        }
                        
                        // Clear and rebuild profile skills
                        profile.skills.clear();
                        profile.skill_categories.clear();
                        
                        // Add new skills
                        for skill in &skills_data.skills {
                            let info = SkillInfo {
                                skill: skill.clone(),
                                added_at: e.updated_at,
                                sources: HashSet::from(["manual".to_string()]),
                            };
                            
                            profile.skills.insert(skill.name.clone(), info);
                            profile.skill_categories
                                .entry(skill.category.clone())
                                .or_insert_with(HashSet::new)
                                .insert(skill.name.clone());
                            
                            // Update global statistics
                            *statistics.skill_counts
                                .entry(skill.name.clone())
                                .or_insert(0) += 1;
                            
                            statistics.skill_categories
                                .entry(skill.category.clone())
                                .or_insert_with(HashSet::new)
                                .insert(skill.name.clone());
                        }
                        
                        profile.last_updated = e.updated_at;
                        
                        // Update skill relationships
                        drop(statistics); // Release the lock
                        let skill_names: Vec<_> = skills_data.skills.iter()
                            .map(|s| s.name.clone())
                            .collect();
                        self.update_skill_relationships(&skill_names).await;
                    }
                }
            }
            
            PersonEvent::PersonDeactivated(e) => {
                let mut profiles = self.profiles.write().await;
                let mut statistics = self.statistics.write().await;
                
                // Remove skills from statistics
                if let Some(profile) = profiles.remove(&e.person_id) {
                    for skill_name in profile.skills.keys() {
                        if let Some(count) = statistics.skill_counts.get_mut(skill_name) {
                            *count = count.saturating_sub(1);
                            if *count == 0 {
                                statistics.skill_counts.remove(skill_name);
                            }
                        }
                    }
                }
            }
            
            PersonEvent::PersonMergedInto(e) => {
                let mut profiles = self.profiles.write().await;
                let mut statistics = self.statistics.write().await;
                
                // Remove skills from statistics
                if let Some(profile) = profiles.remove(&e.source_person_id) {
                    for skill_name in profile.skills.keys() {
                        if let Some(count) = statistics.skill_counts.get_mut(skill_name) {
                            *count = count.saturating_sub(1);
                            if *count == 0 {
                                statistics.skill_counts.remove(skill_name);
                            }
                        }
                    }
                }
            }
            
            _ => {} // Other events don't affect skills
        }
        
        Ok(())
    }
    
    fn projection_name(&self) -> &str {
        "PersonSkillsProjection"
    }
    
    async fn clear(&self) -> DomainResult<()> {
        let mut profiles = self.profiles.write().await;
        let mut statistics = self.statistics.write().await;
        profiles.clear();
        *statistics = SkillStatistics::default();
        Ok(())
    }
} 