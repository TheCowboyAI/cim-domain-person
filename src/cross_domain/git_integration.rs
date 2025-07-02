//! Integration with Git domain for contribution tracking

use crate::aggregate::PersonId;
use crate::events::ComponentDataEvent;
use crate::components::data::ProficiencyLevel;
use crate::components::{ComponentStore, InMemoryComponentStore};
use crate::infrastructure::PersonRepository;
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};

/// Events from Git domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitDomainEvent {
    CommitAnalyzed {
        commit_hash: String,
        repository: String,
        author_email: String,
        author_name: String,
        timestamp: DateTime<Utc>,
        files_changed: Vec<FileChange>,
        languages: Vec<String>,
        lines_added: u32,
        lines_deleted: u32,
    },
    AuthorIdentified {
        person_id: PersonId,
        git_email: String,
        git_username: Option<String>,
        repositories: Vec<String>,
    },
    ContributionMetricsCalculated {
        person_id: PersonId,
        repository: String,
        total_commits: u64,
        total_lines_added: u64,
        total_lines_deleted: u64,
        first_commit: DateTime<Utc>,
        last_commit: DateTime<Utc>,
        primary_languages: Vec<LanguageStats>,
    },
    PullRequestMerged {
        person_id: PersonId,
        repository: String,
        pr_number: u32,
        title: String,
        merged_at: DateTime<Utc>,
        commits: u32,
        additions: u32,
        deletions: u32,
    },
    CodeReviewCompleted {
        person_id: PersonId,
        repository: String,
        pr_number: u32,
        review_type: ReviewType,
        submitted_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: FileChangeType,
    pub language: Option<String>,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: String,
    pub files: u32,
    pub lines: u64,
    pub percentage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewType {
    Approved,
    ChangesRequested,
    Commented,
}

/// Handler for Git domain events
pub struct GitEventHandler {
    person_repository: Arc<PersonRepository>,
    component_store: Arc<InMemoryComponentStore>,
}

impl GitEventHandler {
    pub fn new(
        person_repository: Arc<PersonRepository>,
        component_store: Arc<InMemoryComponentStore>,
    ) -> Self {
        Self {
            person_repository,
            component_store,
        }
    }
    
    /// Process an event from the Git domain
    pub async fn handle_event(&self, event: GitDomainEvent) -> DomainResult<Vec<ComponentDataEvent>> {
        match event {
            GitDomainEvent::ContributionMetricsCalculated {
                person_id,
                primary_languages,
                ..
            } => {
                self.handle_contribution_metrics(person_id, primary_languages).await
            }
            GitDomainEvent::PullRequestMerged {
                person_id,
                repository,
                ..
            } => {
                self.handle_pull_request_merged(person_id, repository).await
            }
            _ => Ok(vec![]),
        }
    }
    
    async fn handle_contribution_metrics(
        &self,
        person_id: PersonId,
        primary_languages: Vec<LanguageStats>,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // Verify person exists
        let person = self.person_repository.load(person_id).await?;
        if person.is_none() {
            return Err(DomainError::AggregateNotFound(format!("Person {}", person_id)));
        }
        
        let mut events = Vec::new();
        
        // Get all skill components for the person
        let _skill_components = self.component_store.get_components_by_type(
            person_id,
            crate::aggregate::ComponentType::Skill,
        ).await?;
        
        // Create or update skill components based on language usage
        for lang_stats in primary_languages {
            if lang_stats.percentage < 5.0 {
                continue; // Skip languages with minimal usage
            }
            
            // Check if skill already exists
            let _existing_skills = self.component_store.get_components_by_type(
                person_id,
                crate::aggregate::ComponentType::Skill,
            ).await?;
            
            // For simplicity, we'll create a new skill component
            // In reality, we'd check if the skill already exists and update it
            let proficiency = self.calculate_proficiency(&lang_stats);
            
            let skill_data = crate::components::data::SkillComponentData {
                name: lang_stats.language.clone(),
                category: crate::components::data::SkillCategory::Programming,
                proficiency,
                years_of_experience: Some(0.0), // Could be calculated from first/last commit
                last_used: Some(Utc::now()),
                verified: false,
                endorsements: Vec::new(),
                certifications: Vec::new(),
                projects: vec![lang_stats.language.clone()],
            };
            
            let component = crate::components::data::ComponentInstance::new(person_id, skill_data)?;
            let component_id = component.id;
            
            // Store component
            self.component_store.store_component(component).await?;
            
            // Create event
            events.push(ComponentDataEvent::SkillAdded {
                person_id,
                component_id,
                skill_name: lang_stats.language,
                category: crate::components::data::SkillCategory::Programming,
                proficiency,
                timestamp: Utc::now(),
            });
        }
        
        Ok(events)
    }
    
    async fn handle_pull_request_merged(
        &self,
        person_id: PersonId,
        repository: String,
    ) -> DomainResult<Vec<ComponentDataEvent>> {
        // This could update achievements or create a contribution component
        tracing::info!(
            "Pull request merged for person {} in repository {}",
            person_id,
            repository
        );
        
        // Could create an achievement component or update existing skills
        Ok(vec![])
    }
    
    fn calculate_proficiency(&self, lang_stats: &LanguageStats) -> ProficiencyLevel {
        // Simple heuristic based on lines of code
        match lang_stats.lines {
            0..=1000 => ProficiencyLevel::Beginner,
            1001..=5000 => ProficiencyLevel::Intermediate,
            5001..=20000 => ProficiencyLevel::Advanced,
            _ => ProficiencyLevel::Expert,
        }
    }
}

/// Commands to send to Git domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitDomainCommand {
    /// Link a git identity to a person
    LinkGitIdentity {
        person_id: PersonId,
        git_email: String,
        git_username: Option<String>,
    },
    
    /// Analyze contributions for a person
    AnalyzePersonContributions {
        person_id: PersonId,
        repositories: Option<Vec<String>>,
        since: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    },
    
    /// Get contribution statistics
    GetContributionStats {
        person_id: PersonId,
        group_by: GroupingPeriod,
    },
    
    /// Find repositories a person has contributed to
    FindPersonRepositories {
        person_id: PersonId,
        min_commits: Option<u32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupingPeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Service for git-related operations
pub struct GitIntegrationService {
    #[allow(dead_code)]
    person_repository: Arc<PersonRepository>,
    component_store: Arc<InMemoryComponentStore>,
}

impl GitIntegrationService {
    pub fn new(
        person_repository: Arc<PersonRepository>,
        component_store: Arc<InMemoryComponentStore>,
    ) -> Self {
        Self {
            person_repository,
            component_store,
        }
    }
    
    /// Get programming languages for a person based on their contributions
    pub async fn get_person_languages(&self, person_id: PersonId) -> DomainResult<Vec<String>> {
        // Get all skill components
        let _skill_components = self.component_store.get_components_by_type(
            person_id,
            crate::aggregate::ComponentType::Skill,
        ).await?;
        
        // Filter for programming language skills
        // This is simplified - in reality we'd load the components and check
        Ok(vec![])
    }
} 