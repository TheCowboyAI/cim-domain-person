//! Query specifications and filters
//!
//! This module provides formal query specification objects that define
//! how to query the read models. These are pure value objects with no behavior.
//!
//! Following the Specification pattern from Domain-Driven Design.

use crate::aggregate::PersonId;
use crate::projections::person_network_projection::RelationshipType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Specification for querying person summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSummaryQuery {
    /// Optional filter by person IDs
    pub person_ids: Option<Vec<PersonId>>,

    /// Optional filter by employer name
    pub employer: Option<String>,

    /// Optional filter by location
    pub location: Option<String>,

    /// Pagination: page number (0-indexed)
    pub page: usize,

    /// Pagination: page size
    pub page_size: usize,
}

impl PersonSummaryQuery {
    /// Create a query for a specific person
    pub fn for_person(person_id: PersonId) -> Self {
        Self {
            person_ids: Some(vec![person_id]),
            employer: None,
            location: None,
            page: 0,
            page_size: 1,
        }
    }

    /// Create a query for all persons
    pub fn all() -> Self {
        Self {
            person_ids: None,
            employer: None,
            location: None,
            page: 0,
            page_size: 100,
        }
    }

    /// Filter by employer
    pub fn by_employer(employer: String) -> Self {
        Self {
            person_ids: None,
            employer: Some(employer),
            location: None,
            page: 0,
            page_size: 100,
        }
    }

    /// Add pagination
    pub fn paginate(mut self, page: usize, page_size: usize) -> Self {
        self.page = page;
        self.page_size = page_size;
        self
    }
}

/// Specification for searching persons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonSearchQuery {
    /// Text search query
    pub query_text: Option<String>,

    /// Filter by employer
    pub employer_filter: Option<String>,

    /// Filter by skill
    pub skill_filter: Option<String>,

    /// Filter by location
    pub location_filter: Option<String>,

    /// Maximum results to return
    pub limit: usize,

    /// Minimum relevance score (0.0 - 1.0)
    pub min_relevance: f32,
}

impl PersonSearchQuery {
    /// Create a simple text search
    pub fn text_search(query: String) -> Self {
        Self {
            query_text: Some(query),
            employer_filter: None,
            skill_filter: None,
            location_filter: None,
            limit: 50,
            min_relevance: 0.0,
        }
    }

    /// Create an empty query (matches all)
    pub fn all() -> Self {
        Self {
            query_text: None,
            employer_filter: None,
            skill_filter: None,
            location_filter: None,
            limit: 100,
            min_relevance: 0.0,
        }
    }

    /// Add employer filter
    pub fn with_employer(mut self, employer: String) -> Self {
        self.employer_filter = Some(employer);
        self
    }

    /// Add skill filter
    pub fn with_skill(mut self, skill: String) -> Self {
        self.skill_filter = Some(skill);
        self
    }

    /// Add location filter
    pub fn with_location(mut self, location: String) -> Self {
        self.location_filter = Some(location);
        self
    }

    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set minimum relevance score
    pub fn min_relevance(mut self, score: f32) -> Self {
        self.min_relevance = score;
        self
    }
}

/// Specification for querying person skills
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsQuery {
    /// Query by person ID
    pub person_id: Option<PersonId>,

    /// Query by skill name
    pub skill_name: Option<String>,

    /// Minimum proficiency level
    pub min_proficiency: Option<f32>,

    /// Filter by skill category
    pub category: Option<String>,
}

impl SkillsQuery {
    /// Get skills for a person
    pub fn for_person(person_id: PersonId) -> Self {
        Self {
            person_id: Some(person_id),
            skill_name: None,
            min_proficiency: None,
            category: None,
        }
    }

    /// Find people with a skill
    pub fn with_skill(skill_name: String) -> Self {
        Self {
            person_id: None,
            skill_name: Some(skill_name),
            min_proficiency: None,
            category: None,
        }
    }

    /// Filter by minimum proficiency
    pub fn min_proficiency(mut self, proficiency: f32) -> Self {
        self.min_proficiency = Some(proficiency);
        self
    }

    /// Filter by category
    pub fn in_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }
}

/// Specification for querying person network/connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkQuery {
    /// Person ID to query network for
    pub person_id: PersonId,

    /// Depth of connections to traverse (1 = direct, 2 = friends-of-friends, etc.)
    pub depth: usize,

    /// Filter by relationship type
    pub relationship_type: Option<RelationshipType>,

    /// Include incoming connections
    pub include_incoming: bool,

    /// Include outgoing connections
    pub include_outgoing: bool,
}

impl NetworkQuery {
    /// Get direct connections for a person
    pub fn direct_connections(person_id: PersonId) -> Self {
        Self {
            person_id,
            depth: 1,
            relationship_type: None,
            include_incoming: true,
            include_outgoing: true,
        }
    }

    /// Set connection depth
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Filter by relationship type
    pub fn relationship_type(mut self, rel_type: RelationshipType) -> Self {
        self.relationship_type = Some(rel_type);
        self
    }

    /// Only incoming connections
    pub fn incoming_only(mut self) -> Self {
        self.include_incoming = true;
        self.include_outgoing = false;
        self
    }

    /// Only outgoing connections
    pub fn outgoing_only(mut self) -> Self {
        self.include_incoming = false;
        self.include_outgoing = true;
        self
    }
}

/// Specification for querying person timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineQuery {
    /// Person ID
    pub person_id: PersonId,

    /// Start date (inclusive)
    pub start_date: Option<DateTime<Utc>>,

    /// End date (inclusive)
    pub end_date: Option<DateTime<Utc>>,

    /// Filter by event type
    pub event_types: Option<Vec<String>>,

    /// Maximum events to return
    pub limit: Option<usize>,

    /// Sort order (true = ascending, false = descending)
    pub ascending: bool,
}

impl TimelineQuery {
    /// Get complete timeline for a person
    pub fn for_person(person_id: PersonId) -> Self {
        Self {
            person_id,
            start_date: None,
            end_date: None,
            event_types: None,
            limit: None,
            ascending: false, // Default to most recent first
        }
    }

    /// Filter by date range
    pub fn date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    /// Filter by event types
    pub fn event_types(mut self, types: Vec<String>) -> Self {
        self.event_types = Some(types);
        self
    }

    /// Limit number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sort ascending (oldest first)
    pub fn ascending(mut self) -> Self {
        self.ascending = true;
        self
    }

    /// Sort descending (newest first)
    pub fn descending(mut self) -> Self {
        self.ascending = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_summary_query_builder() {
        let query = PersonSummaryQuery::by_employer("Acme Corp".to_string())
            .paginate(2, 25);

        assert_eq!(query.employer, Some("Acme Corp".to_string()));
        assert_eq!(query.page, 2);
        assert_eq!(query.page_size, 25);
    }

    #[test]
    fn test_search_query_builder() {
        let query = PersonSearchQuery::text_search("John Smith".to_string())
            .with_employer("Tech Co".to_string())
            .with_skill("Rust".to_string())
            .limit(10)
            .min_relevance(0.5);

        assert_eq!(query.query_text, Some("John Smith".to_string()));
        assert_eq!(query.employer_filter, Some("Tech Co".to_string()));
        assert_eq!(query.skill_filter, Some("Rust".to_string()));
        assert_eq!(query.limit, 10);
        assert_eq!(query.min_relevance, 0.5);
    }

    #[test]
    fn test_skills_query_builder() {
        let person_id = PersonId::new();
        let query = SkillsQuery::for_person(person_id)
            .min_proficiency(0.7)
            .in_category("Programming".to_string());

        assert_eq!(query.person_id, Some(person_id));
        assert_eq!(query.min_proficiency, Some(0.7));
        assert_eq!(query.category, Some("Programming".to_string()));
    }

    #[test]
    fn test_network_query_builder() {
        let person_id = PersonId::new();
        let query = NetworkQuery::direct_connections(person_id)
            .depth(2)
            .relationship_type(RelationshipType::Colleague)
            .outgoing_only();

        assert_eq!(query.depth, 2);
        assert_eq!(query.relationship_type, Some(RelationshipType::Colleague));
        assert!(!query.include_incoming);
        assert!(query.include_outgoing);
    }

    #[test]
    fn test_timeline_query_builder() {
        let person_id = PersonId::new();
        let start = Utc::now();
        let end = Utc::now();

        let query = TimelineQuery::for_person(person_id)
            .date_range(start, end)
            .event_types(vec!["PersonCreated".to_string(), "NameUpdated".to_string()])
            .limit(50)
            .ascending();

        assert_eq!(query.start_date, Some(start));
        assert_eq!(query.end_date, Some(end));
        assert_eq!(query.limit, Some(50));
        assert!(query.ascending);
    }
}
