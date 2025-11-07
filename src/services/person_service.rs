//! Person Service - Explicit CQRS Implementation
//!
//! This module provides the top-level PersonService that explicitly separates
//! Commands (writes) from Queries (reads) following CQRS principles.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────┐
//! │         PersonService                │
//! │  ┌────────────┐    ┌──────────────┐ │
//! │  │  Commands  │    │   Queries    │ │
//! │  │  (Writes)  │    │   (Reads)    │ │
//! │  └──────┬─────┘    └──────┬───────┘ │
//! └─────────┼──────────────────┼─────────┘
//!           │                  │
//!           v                  v
//!    ┌─────────────┐    ┌────────────┐
//!    │   Domain    │    │ Read Models│
//!    │ Aggregates  │    │(Projections│
//!    │  + Events   │    │            │
//!    └──────┬──────┘    └─────▲──────┘
//!           │                  │
//!           v                  │
//!    ┌─────────────────────────┼──────┐
//!    │  Infrastructure Layer   │      │
//!    │   (Event Store, NATS)   ├──────┘
//!    └─────────────────────────┘
//! ```
//!
//! # FRP/CT Compliance
//!
//! - **Command Side**: Pure functions that produce events (no side effects)
//! - **Query Side**: Pure read operations against projections
//! - **Complete Separation**: Commands and queries never mix
//! - **Event-Driven**: Commands produce events, projections consume events

use std::sync::Arc;
use cim_domain::DomainResult;

use crate::aggregate::PersonId;
use crate::commands::PersonCommand;
use crate::handlers::{AsyncCommandProcessor, PersonCommandProcessor, CommandResult};
use crate::queries::{
    PersonQueryService, PersonSummaryQuery, PersonSearchQuery,
    SkillsQuery, NetworkQuery, TimelineQuery,
};
use crate::projections::*;

/// Top-level Person Service with explicit CQRS separation
///
/// This service provides a unified interface while maintaining strict
/// separation between commands (writes) and queries (reads).
///
/// # Example
///
/// ```rust,ignore
/// use cim_domain_person::services::PersonService;
/// use cim_domain_person::commands::CreatePerson;
/// use cim_domain_person::queries::PersonSummaryQuery;
///
/// let service = PersonService::new(command_processor, query_service);
///
/// // Execute command (write)
/// let command = CreatePerson { /* ... */ };
/// let result = service.execute_command(command.into()).await?;
///
/// // Execute query (read)
/// let query = PersonSummaryQuery::all();
/// let summaries = service.query_summaries(&query).await?;
/// ```
pub struct PersonService {
    /// Command processor (write side)
    /// Handles all state-changing operations
    commands: Arc<PersonCommandProcessor>,

    /// Query service (read side)
    /// Handles all read-only operations
    queries: Arc<PersonQueryService>,
}

impl PersonService {
    /// Create a new PersonService
    ///
    /// # Parameters
    /// - `commands`: Command processor for handling writes
    /// - `queries`: Query service for handling reads
    pub fn new(
        commands: Arc<PersonCommandProcessor>,
        queries: Arc<PersonQueryService>,
    ) -> Self {
        Self { commands, queries }
    }

    // ========================================================================
    // Command Side (Writes)
    // ========================================================================

    /// Execute a command (write operation)
    ///
    /// This is the single entry point for all state-changing operations.
    /// Commands are processed through the domain model and produce events.
    ///
    /// # FRP/CT Compliance
    /// - Commands are pure: same input always produces same events
    /// - Side effects (persistence, publishing) happen in infrastructure
    /// - Domain logic remains pure and testable
    pub async fn execute_command(&self, command: PersonCommand) -> DomainResult<CommandResult> {
        self.commands.process_command(command).await
    }

    /// Execute a command with correlation ID for distributed tracing
    pub async fn execute_command_with_correlation(
        &self,
        command: PersonCommand,
        correlation_id: uuid::Uuid,
    ) -> DomainResult<CommandResult> {
        self.commands
            .process_command_with_correlation(command, correlation_id)
            .await
    }

    // ========================================================================
    // Query Side (Reads)
    // ========================================================================

    // --- Person Summary Queries ---

    /// Query person summaries
    ///
    /// # FRP/CT Compliance
    /// - Pure read operation with no side effects
    /// - Uses query specification pattern
    /// - Reads from projections, never from domain model
    pub async fn query_summaries(&self, query: &PersonSummaryQuery) -> DomainResult<Vec<PersonSummary>> {
        // Apply the query specification to get results
        if let Some(employer) = &query.employer {
            let all = self.queries.get_summaries_by_employer(employer).await;
            Ok(Self::paginate(all, query.page, query.page_size))
        } else if let Some(person_ids) = &query.person_ids {
            let mut results = Vec::new();
            for id in person_ids {
                if let Some(summary) = self.queries.get_person_summary(id).await {
                    results.push(summary);
                }
            }
            Ok(results)
        } else {
            let all = self.queries.get_all_summaries().await;
            Ok(Self::paginate(all, query.page, query.page_size))
        }
    }

    /// Get a single person summary
    pub async fn get_summary(&self, person_id: &PersonId) -> DomainResult<Option<PersonSummary>> {
        Ok(self.queries.get_person_summary(person_id).await)
    }

    // --- Search Queries ---

    /// Search for persons
    pub async fn search_persons(&self, query: &PersonSearchQuery) -> DomainResult<Vec<PersonSearchResult>> {
        let results = self.queries.search_with_filters(
            query.query_text.as_deref(),
            query.employer_filter.as_deref(),
            query.skill_filter.as_deref(),
            query.location_filter.as_deref(),
            query.limit,
        ).await;

        // Apply minimum relevance filter
        let filtered = results.into_iter()
            .filter(|r| r.relevance_score >= query.min_relevance)
            .collect();

        Ok(filtered)
    }

    // --- Skills Queries ---

    /// Query person skills
    pub async fn query_skills(&self, query: &SkillsQuery) -> DomainResult<Vec<SkillSummary>> {
        if let Some(person_id) = &query.person_id {
            let skills = self.queries.get_person_skills(person_id).await;

            // Apply filters
            let filtered = skills.into_iter()
                .filter(|skill| {
                    // Filter by category if specified
                    if let Some(cat) = &query.category {
                        if &skill.category != cat {
                            return false;
                        }
                    }

                    // Filter by skill name if specified
                    if let Some(name) = &query.skill_name {
                        if &skill.skill_name != name {
                            return false;
                        }
                    }

                    true
                })
                .collect();

            Ok(filtered)
        } else if let Some(skill_name) = &query.skill_name {
            // Find people with this skill
            let _person_ids = self.queries.find_people_with_skill(skill_name).await;

            // For each person, get their skill summary for this specific skill
            // (This is a simplified implementation)
            Ok(Vec::new()) // Would need to aggregate from multiple sources
        } else {
            Ok(Vec::new())
        }
    }

    // --- Network Queries ---

    /// Query person network/connections
    pub async fn query_network(&self, query: &NetworkQuery) -> DomainResult<Vec<PersonRelationship>> {
        let mut connections = if query.include_outgoing {
            self.queries.get_person_connections(&query.person_id).await
        } else {
            Vec::new()
        };

        if query.include_incoming {
            let incoming = self.queries.get_incoming_connections(&query.person_id).await;
            connections.extend(incoming);
        }

        // Apply relationship type filter
        if let Some(rel_type) = &query.relationship_type {
            connections = connections.into_iter()
                .filter(|conn| &conn.relationship_type == rel_type)
                .collect();
        }

        Ok(connections)
    }

    /// Get network statistics for a person
    pub async fn get_network_stats(&self, person_id: &PersonId) -> DomainResult<NetworkStats> {
        Ok(self.queries.get_network_stats(person_id).await)
    }

    /// Find shortest path between two people
    pub async fn find_path(
        &self,
        from: &PersonId,
        to: &PersonId,
    ) -> DomainResult<Option<Vec<PersonId>>> {
        Ok(self.queries.find_shortest_path(from, to).await)
    }

    // --- Timeline Queries ---

    /// Query person timeline
    pub async fn query_timeline(&self, query: &TimelineQuery) -> DomainResult<Vec<TimelineEntry>> {
        let timeline = if let (Some(start), Some(end)) = (query.start_date, query.end_date) {
            self.queries.get_timeline_range(&query.person_id, start, end).await
        } else {
            self.queries.get_person_timeline(&query.person_id, query.limit).await
        };

        // Apply event type filter
        let filtered = if let Some(types) = &query.event_types {
            timeline.into_iter()
                .filter(|entry| types.contains(&entry.event_type))
                .collect()
        } else {
            timeline
        };

        // Apply sort order
        let mut sorted = filtered;
        sorted.sort_by(|a, b| {
            if query.ascending {
                a.timestamp.cmp(&b.timestamp)
            } else {
                b.timestamp.cmp(&a.timestamp)
            }
        });

        Ok(sorted)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Paginate results (pure function)
    fn paginate<T>(items: Vec<T>, page: usize, page_size: usize) -> Vec<T> {
        let start = page * page_size;
        let end = std::cmp::min(start + page_size, items.len());

        if start >= items.len() {
            Vec::new()
        } else {
            items.into_iter().skip(start).take(end - start).collect()
        }
    }
}

// ============================================================================
// Marker Traits for Type Safety
// ============================================================================

/// Marker trait for command operations (writes)
///
/// This ensures commands can never be confused with queries at compile time.
pub trait CommandOperation {
    type Result;
}

impl CommandOperation for PersonCommand {
    type Result = CommandResult;
}

/// Marker trait for query operations (reads)
///
/// This ensures queries can never be confused with commands at compile time.
pub trait QueryOperation {
    type Result;
}

impl QueryOperation for PersonSummaryQuery {
    type Result = Vec<PersonSummary>;
}

impl QueryOperation for PersonSearchQuery {
    type Result = Vec<PersonSearchResult>;
}

impl QueryOperation for SkillsQuery {
    type Result = Vec<SkillSummary>;
}

impl QueryOperation for NetworkQuery {
    type Result = Vec<PersonRelationship>;
}

impl QueryOperation for TimelineQuery {
    type Result = Vec<TimelineEntry>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // Page 0, size 3
        let page0 = PersonService::paginate(items.clone(), 0, 3);
        assert_eq!(page0, vec![1, 2, 3]);

        // Page 1, size 3
        let page1 = PersonService::paginate(items.clone(), 1, 3);
        assert_eq!(page1, vec![4, 5, 6]);

        // Page 3, size 3
        let page3 = PersonService::paginate(items.clone(), 3, 3);
        assert_eq!(page3, vec![10]);

        // Page beyond bounds
        let page_beyond = PersonService::paginate(items.clone(), 10, 3);
        assert_eq!(page_beyond, Vec::<i32>::new());
    }
}
