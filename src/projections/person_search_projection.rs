//! Person search projection for full-text and faceted search

use super::{PersonProjection, PersonSearchResult};
use crate::aggregate::PersonId;
use crate::events::*;
use crate::components::data::{ComponentData, ContactData, ProfessionalData};
use cim_domain::DomainResult;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Search index entry for a person
#[derive(Debug, Clone)]
struct SearchEntry {
    person_id: PersonId,
    name: String,
    name_tokens: Vec<String>,
    emails: Vec<String>,
    phones: Vec<String>,
    employer: Option<String>,
    role: Option<String>,
    skills: HashSet<String>,
    location: Option<String>,
    tags: HashSet<String>,
    last_updated: DateTime<Utc>,
}

impl SearchEntry {
    fn new(person_id: PersonId, name: String, created_at: DateTime<Utc>) -> Self {
        let name_tokens = tokenize(&name);
        Self {
            person_id,
            name: name.clone(),
            name_tokens,
            emails: Vec::new(),
            phones: Vec::new(),
            employer: None,
            role: None,
            skills: HashSet::new(),
            location: None,
            tags: HashSet::new(),
            last_updated: created_at,
        }
    }
    
    fn calculate_relevance(&self, query: &str) -> f32 {
        let query_tokens = tokenize(query);
        let mut score = 0.0;
        
        // Name matching (highest weight)
        for token in &query_tokens {
            if self.name_tokens.iter().any(|t| t.contains(token)) {
                score += 10.0;
            }
        }
        
        // Email matching
        for token in &query_tokens {
            if self.emails.iter().any(|e| e.contains(token)) {
                score += 5.0;
            }
        }
        
        // Employer/role matching
        if let Some(emp) = &self.employer {
            for token in &query_tokens {
                if emp.to_lowercase().contains(token) {
                    score += 3.0;
                }
            }
        }
        
        if let Some(role) = &self.role {
            for token in &query_tokens {
                if role.to_lowercase().contains(token) {
                    score += 3.0;
                }
            }
        }
        
        // Skills matching
        for token in &query_tokens {
            if self.skills.iter().any(|s| s.to_lowercase().contains(token)) {
                score += 2.0;
            }
        }
        
        // Tag matching
        for token in &query_tokens {
            if self.tags.iter().any(|t| t.to_lowercase().contains(token)) {
                score += 1.0;
            }
        }
        
        score
    }
}

/// Tokenize a string for search
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Projection that maintains a searchable index of persons
pub struct PersonSearchProjection {
    index: Arc<RwLock<HashMap<PersonId, SearchEntry>>>,
}

impl PersonSearchProjection {
    pub fn new() -> Self {
        Self {
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Search for persons using a query string
    pub async fn search(&self, query: &str, limit: usize) -> Vec<PersonSearchResult> {
        let index = self.index.read().await;
        
        let mut results: Vec<_> = index.values()
            .map(|entry| {
                let relevance = entry.calculate_relevance(query);
                (entry, relevance)
            })
            .filter(|(_, relevance)| *relevance > 0.0)
            .collect();
        
        // Sort by relevance (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top N results
        results.into_iter()
            .take(limit)
            .map(|(entry, relevance)| PersonSearchResult {
                person_id: entry.person_id,
                name: entry.name.clone(),
                email: entry.emails.first().cloned(),
                employer: entry.employer.clone(),
                role: entry.role.clone(),
                relevance_score: relevance,
            })
            .collect()
    }
    
    /// Search with filters
    pub async fn search_with_filters(
        &self,
        query: Option<&str>,
        employer_filter: Option<&str>,
        skill_filter: Option<&str>,
        location_filter: Option<&str>,
        limit: usize,
    ) -> Vec<PersonSearchResult> {
        let index = self.index.read().await;
        
        let mut results: Vec<_> = index.values()
            .filter(|entry| {
                // Apply filters
                if let Some(emp) = employer_filter {
                    if !entry.employer.as_ref().map(|e| e.contains(emp)).unwrap_or(false) {
                        return false;
                    }
                }
                
                if let Some(skill) = skill_filter {
                    if !entry.skills.iter().any(|s| s.contains(skill)) {
                        return false;
                    }
                }
                
                if let Some(loc) = location_filter {
                    if !entry.location.as_ref().map(|l| l.contains(loc)).unwrap_or(false) {
                        return false;
                    }
                }
                
                true
            })
            .map(|entry| {
                let relevance = query.map(|q| entry.calculate_relevance(q)).unwrap_or(1.0);
                (entry, relevance)
            })
            .filter(|(_, relevance)| query.is_none() || *relevance > 0.0)
            .collect();
        
        // Sort by relevance (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top N results
        results.into_iter()
            .take(limit)
            .map(|(entry, relevance)| PersonSearchResult {
                person_id: entry.person_id,
                name: entry.name.clone(),
                email: entry.emails.first().cloned(),
                employer: entry.employer.clone(),
                role: entry.role.clone(),
                relevance_score: relevance,
            })
            .collect()
    }
    
    /// Get all unique employers
    pub async fn get_employers(&self) -> Vec<String> {
        let index = self.index.read().await;
        let mut employers: HashSet<String> = HashSet::new();
        
        for entry in index.values() {
            if let Some(emp) = &entry.employer {
                employers.insert(emp.clone());
            }
        }
        
        let mut result: Vec<_> = employers.into_iter().collect();
        result.sort();
        result
    }
    
    /// Get all unique skills
    pub async fn get_skills(&self) -> Vec<String> {
        let index = self.index.read().await;
        let mut skills: HashSet<String> = HashSet::new();
        
        for entry in index.values() {
            for skill in &entry.skills {
                skills.insert(skill.clone());
            }
        }
        
        let mut result: Vec<_> = skills.into_iter().collect();
        result.sort();
        result
    }
}

#[async_trait::async_trait]
impl PersonProjection for PersonSearchProjection {
    async fn handle_event(&self, event: &PersonEvent) -> DomainResult<()> {
        match event {
            PersonEvent::PersonCreated(e) => {
                let entry = SearchEntry::new(e.person_id, e.name.display_name(), e.created_at);
                let mut index = self.index.write().await;
                index.insert(e.person_id, entry);
            }
            
            PersonEvent::NameUpdated(e) => {
                let mut index = self.index.write().await;
                if let Some(entry) = index.get_mut(&e.person_id) {
                    entry.name = e.new_name.display_name();
                    entry.name_tokens = tokenize(&entry.name);
                    entry.last_updated = e.updated_at;
                }
            }
            
            PersonEvent::ComponentDataUpdated(e) => {
                let mut index = self.index.write().await;
                if let Some(entry) = index.get_mut(&e.person_id) {
                    match &e.data {
                        ComponentData::Contact(contact) => {
                            match contact {
                                ContactData::Email(email) => {
                                    if !entry.emails.contains(&email.email.to_string()) {
                                        entry.emails.push(email.email.to_string());
                                    }
                                }
                                ContactData::Phone(phone) => {
                                    if !entry.phones.contains(&phone.phone.to_string()) {
                                        entry.phones.push(phone.phone.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
                        ComponentData::Professional(prof) => {
                            match prof {
                                ProfessionalData::Employment(emp) if emp.is_current => {
                                    entry.employer = Some(emp.company.clone());
                                    entry.role = Some(emp.position.clone());
                                }
                                ProfessionalData::Skills(skills) => {
                                    entry.skills.clear();
                                    for skill in &skills.skills {
                                        entry.skills.insert(skill.name.clone());
                                    }
                                }
                                _ => {}
                            }
                        }
                        ComponentData::Location(loc) => {
                            if let Some(addr) = &loc.address {
                                entry.location = Some(format!("{}, {}", addr.city, addr.country));
                            }
                        }
                        _ => {}
                    }
                    entry.last_updated = e.updated_at;
                }
            }
            
            PersonEvent::PersonDeactivated(e) => {
                let mut index = self.index.write().await;
                index.remove(&e.person_id);
            }
            
            PersonEvent::PersonMergedInto(e) => {
                let mut index = self.index.write().await;
                index.remove(&e.source_person_id);
            }
            
            _ => {} // Other events don't affect search index
        }
        
        Ok(())
    }
    
    fn projection_name(&self) -> &str {
        "PersonSearchProjection"
    }
    
    async fn clear(&self) -> DomainResult<()> {
        let mut index = self.index.write().await;
        index.clear();
        Ok(())
    }
} 