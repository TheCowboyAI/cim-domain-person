//! Query service for the Person domain

use crate::aggregate::PersonId;
use crate::projections::*;
use std::sync::Arc;
use chrono::{DateTime, Utc};

mod async_query_processor;
pub use async_query_processor::{
    AsyncQueryProcessor, PersonQueryProcessor, QueryResult,
    SearchCriteria, TimelineEvent, PersonUpdate,
    consume_query_result
};

/// Query service that coordinates access to projections
pub struct PersonQueryService {
    summary_projection: Arc<PersonSummaryProjection>,
    search_projection: Arc<PersonSearchProjection>,
    skills_projection: Arc<PersonSkillsProjection>,
    network_projection: Arc<PersonNetworkProjection>,
    timeline_projection: Arc<PersonTimelineProjection>,
}

impl PersonQueryService {
    pub fn new(
        summary_projection: Arc<PersonSummaryProjection>,
        search_projection: Arc<PersonSearchProjection>,
        skills_projection: Arc<PersonSkillsProjection>,
        network_projection: Arc<PersonNetworkProjection>,
        timeline_projection: Arc<PersonTimelineProjection>,
    ) -> Self {
        Self {
            summary_projection,
            search_projection,
            skills_projection,
            network_projection,
            timeline_projection,
        }
    }
    
    // Summary queries
    
    /// Get a person's summary
    pub async fn get_person_summary(&self, person_id: &PersonId) -> Option<PersonSummary> {
        self.summary_projection.get_summary(person_id).await
    }
    
    /// Get all person summaries
    pub async fn get_all_summaries(&self) -> Vec<PersonSummary> {
        self.summary_projection.get_all_summaries().await
    }
    
    /// Get summaries by employer
    pub async fn get_summaries_by_employer(&self, employer: &str) -> Vec<PersonSummary> {
        self.summary_projection.get_by_employer(employer).await
    }
    
    // Search queries
    
    /// Search for persons
    pub async fn search_persons(&self, query: &str, limit: usize) -> Vec<PersonSearchResult> {
        self.search_projection.search(query, limit).await
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
        self.search_projection.search_with_filters(
            query,
            employer_filter,
            skill_filter,
            location_filter,
            limit
        ).await
    }
    
    /// Get all unique employers
    pub async fn get_all_employers(&self) -> Vec<String> {
        self.search_projection.get_employers().await
    }
    
    // Skills queries
    
    /// Get skills for a person
    pub async fn get_person_skills(&self, person_id: &PersonId) -> Vec<SkillSummary> {
        self.skills_projection.get_person_skills(person_id).await
    }
    
    /// Find people with a specific skill
    pub async fn find_people_with_skill(&self, skill_name: &str) -> Vec<PersonId> {
        self.skills_projection.find_people_with_skill(skill_name).await
    }
    
    /// Find people with multiple skills
    pub async fn find_people_with_skills(&self, required_skills: &[String]) -> Vec<PersonId> {
        self.skills_projection.find_people_with_skills(required_skills).await
    }
    
    /// Get skill recommendations
    pub async fn get_skill_recommendations(&self, person_id: &PersonId, limit: usize) -> Vec<String> {
        self.skills_projection.get_skill_recommendations(person_id, limit).await
    }
    
    /// Get skill statistics
    pub async fn get_skill_statistics(&self) -> std::collections::HashMap<String, usize> {
        self.skills_projection.get_skill_statistics().await
    }
    
    // Network queries
    
    /// Get a person's connections
    pub async fn get_person_connections(&self, person_id: &PersonId) -> Vec<PersonRelationship> {
        self.network_projection.get_connections(person_id).await
    }
    
    /// Get incoming connections
    pub async fn get_incoming_connections(&self, person_id: &PersonId) -> Vec<PersonRelationship> {
        self.network_projection.get_incoming_connections(person_id).await
    }
    
    /// Get network statistics
    pub async fn get_network_stats(&self, person_id: &PersonId) -> NetworkStats {
        self.network_projection.get_network_stats(person_id).await
    }
    
    /// Find shortest path between two people
    pub async fn find_shortest_path(&self, from: &PersonId, to: &PersonId) -> Option<Vec<PersonId>> {
        self.network_projection.find_shortest_path(from, to).await
    }
    
    // Timeline queries
    
    /// Get a person's timeline
    pub async fn get_person_timeline(&self, person_id: &PersonId, limit: Option<usize>) -> Vec<TimelineEntry> {
        self.timeline_projection.get_timeline(person_id, limit).await
    }
    
    /// Get timeline within date range
    pub async fn get_timeline_range(
        &self,
        person_id: &PersonId,
        start: DateTime<Utc>,
        end: DateTime<Utc>
    ) -> Vec<TimelineEntry> {
        self.timeline_projection.get_timeline_range(person_id, start, end).await
    }
    
    /// Get timeline by event type
    pub async fn get_timeline_by_type(
        &self,
        person_id: &PersonId,
        event_type: &str
    ) -> Vec<TimelineEntry> {
        self.timeline_projection.get_timeline_by_type(person_id, event_type).await
    }
}

/// Query request types for NATS integration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "query_type")]
pub enum PersonQuery {
    GetSummary { person_id: PersonId },
    GetAllSummaries,
    GetByEmployer { employer: String },
    Search { query: String, limit: usize },
    SearchWithFilters {
        query: Option<String>,
        employer_filter: Option<String>,
        skill_filter: Option<String>,
        location_filter: Option<String>,
        limit: usize,
    },
    GetSkills { person_id: PersonId },
    FindPeopleWithSkill { skill_name: String },
    FindPeopleWithSkills { required_skills: Vec<String> },
    GetSkillRecommendations { person_id: PersonId, limit: usize },
    GetConnections { person_id: PersonId },
    GetNetworkStats { person_id: PersonId },
    FindShortestPath { from: PersonId, to: PersonId },
    GetTimeline { person_id: PersonId, limit: Option<usize> },
    GetTimelineRange { person_id: PersonId, start: DateTime<Utc>, end: DateTime<Utc> },
}

/// Query response types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "response_type")]
pub enum PersonQueryResponse {
    Summary(Option<PersonSummary>),
    Summaries(Vec<PersonSummary>),
    SearchResults(Vec<PersonSearchResult>),
    Skills(Vec<SkillSummary>),
    PersonIds(Vec<PersonId>),
    SkillRecommendations(Vec<String>),
    Connections(Vec<PersonRelationship>),
    NetworkStats(NetworkStats),
    Path(Option<Vec<PersonId>>),
    Timeline(Vec<TimelineEntry>),
    Error { message: String },
}


