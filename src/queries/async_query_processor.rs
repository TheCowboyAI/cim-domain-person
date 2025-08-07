//! Async query processor for streaming responses

use async_trait::async_trait;
use cim_domain::DomainResult;
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, debug};

use crate::aggregate::PersonId;
use crate::projections::*;

/// Person network view
#[derive(Debug, Clone)]
pub struct PersonNetworkView {
    pub person_id: PersonId,
    pub person_name: String,
    pub connections: Vec<NetworkConnection>,
    pub total_connections: usize,
    pub depth_reached: usize,
}

#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub person_id: PersonId,
    pub person_name: String,
    pub connection_type: String,
    pub strength: f32,
}

/// Query result with optional streaming
pub enum QueryResult<T> {
    /// Single result
    Single(T),
    /// Multiple results
    Multiple(Vec<T>),
    /// Streaming results
    Stream(Pin<Box<dyn Stream<Item = T> + Send>>),
}

/// Async query processor trait
#[async_trait]
pub trait AsyncQueryProcessor: Send + Sync {
    /// Get person by ID
    async fn get_person(&self, person_id: PersonId) -> DomainResult<Option<PersonSummary>>;
    
    /// Search persons with streaming results
    async fn search_persons(
        &self, 
        criteria: SearchCriteria,
    ) -> DomainResult<QueryResult<PersonSearchResult>>;
    
    /// Get person network with depth
    async fn get_person_network(
        &self,
        person_id: PersonId,
        depth: usize,
    ) -> DomainResult<Option<PersonNetworkView>>;
    
    /// Get person timeline events
    async fn get_person_timeline(
        &self,
        person_id: PersonId,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> DomainResult<QueryResult<TimelineEvent>>;
    
    /// Subscribe to person updates
    async fn subscribe_to_updates(
        &self,
        person_id: PersonId,
    ) -> DomainResult<Pin<Box<dyn Stream<Item = PersonUpdate> + Send>>>;
}

/// Person update notification
#[derive(Debug, Clone)]
pub struct PersonUpdate {
    pub person_id: PersonId,
    pub update_type: UpdateType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum UpdateType {
    NameChanged,
    ComponentAdded,
    ComponentUpdated,
    ComponentRemoved,
    StatusChanged,
    RelationshipChanged,
}

/// Search criteria for person queries
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    pub name_pattern: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub skills: Vec<String>,
    pub location_id: Option<uuid::Uuid>,
    pub organization_id: Option<uuid::Uuid>,
    pub active_only: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

// PersonSearchResult is defined in projections::mod

/// Timeline event
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub event_id: uuid::Uuid,
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub metadata: serde_json::Value,
}

/// Implementation of async query processor
pub struct PersonQueryProcessor {
    // In a real implementation, these would be projection stores
    summary_store: Arc<dyn SummaryProjectionStore>,
    search_store: Arc<dyn SearchProjectionStore>,
    network_store: Arc<dyn NetworkProjectionStore>,
    timeline_store: Arc<dyn TimelineProjectionStore>,
}

/// Trait for projection stores
#[async_trait]
pub trait SummaryProjectionStore: Send + Sync {
    async fn get_summary(&self, person_id: PersonId) -> DomainResult<Option<PersonSummary>>;
}

#[async_trait]
pub trait SearchProjectionStore: Send + Sync {
    async fn search(&self, criteria: &SearchCriteria) -> DomainResult<Vec<PersonSearchResult>>;
    async fn search_stream(&self, criteria: &SearchCriteria) 
        -> DomainResult<Pin<Box<dyn Stream<Item = PersonSearchResult> + Send>>>;
}

#[async_trait]
pub trait NetworkProjectionStore: Send + Sync {
    async fn get_network(&self, person_id: PersonId, depth: usize) 
        -> DomainResult<Option<PersonNetworkView>>;
}

#[async_trait]
pub trait TimelineProjectionStore: Send + Sync {
    async fn get_timeline(
        &self,
        person_id: PersonId,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> DomainResult<Vec<TimelineEvent>>;
    
    async fn get_timeline_stream(
        &self,
        person_id: PersonId,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> DomainResult<Pin<Box<dyn Stream<Item = TimelineEvent> + Send>>>;
}

impl PersonQueryProcessor {
    pub fn new(
        summary_store: Arc<dyn SummaryProjectionStore>,
        search_store: Arc<dyn SearchProjectionStore>,
        network_store: Arc<dyn NetworkProjectionStore>,
        timeline_store: Arc<dyn TimelineProjectionStore>,
    ) -> Self {
        Self {
            summary_store,
            search_store,
            network_store,
            timeline_store,
        }
    }
}

#[async_trait]
impl AsyncQueryProcessor for PersonQueryProcessor {
    async fn get_person(&self, person_id: PersonId) -> DomainResult<Option<PersonSummary>> {
        debug!("Getting person summary for {}", person_id);
        self.summary_store.get_summary(person_id).await
    }
    
    async fn search_persons(
        &self,
        criteria: SearchCriteria,
    ) -> DomainResult<QueryResult<PersonSearchResult>> {
        info!("Searching persons with criteria: {:?}", criteria);
        
        // For large result sets, use streaming
        if criteria.limit.is_none() || criteria.limit.unwrap_or(0) > 100 {
            let stream = self.search_store.search_stream(&criteria).await?;
            Ok(QueryResult::Stream(stream))
        } else {
            let results = self.search_store.search(&criteria).await?;
            Ok(QueryResult::Multiple(results))
        }
    }
    
    async fn get_person_network(
        &self,
        person_id: PersonId,
        depth: usize,
    ) -> DomainResult<Option<PersonNetworkView>> {
        debug!("Getting person network for {} with depth {}", person_id, depth);
        self.network_store.get_network(person_id, depth).await
    }
    
    async fn get_person_timeline(
        &self,
        person_id: PersonId,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> DomainResult<QueryResult<TimelineEvent>> {
        debug!("Getting timeline for person {}", person_id);
        
        // Use streaming for open-ended queries
        if from.is_none() && to.is_none() {
            let stream = self.timeline_store.get_timeline_stream(person_id, from, to).await?;
            Ok(QueryResult::Stream(stream))
        } else {
            let events = self.timeline_store.get_timeline(person_id, from, to).await?;
            Ok(QueryResult::Multiple(events))
        }
    }
    
    async fn subscribe_to_updates(
        &self,
        person_id: PersonId,
    ) -> DomainResult<Pin<Box<dyn Stream<Item = PersonUpdate> + Send>>> {
        info!("Subscribing to updates for person {}", person_id);
        
        // Create a channel for updates
        let (tx, rx) = mpsc::channel(100);
        
        // In a real implementation, this would subscribe to NATS events
        // For now, return an empty stream
        tokio::spawn(async move {
            // Subscribe to person.events.{person_id}.*
            // Convert events to PersonUpdate and send through channel
            drop(tx); // Close channel when done
        });
        
        Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }
}

/// Helper to consume query results
pub async fn consume_query_result<T>(result: QueryResult<T>) -> Vec<T> 
where
    T: Send + 'static,
{
    match result {
        QueryResult::Single(item) => vec![item],
        QueryResult::Multiple(items) => items,
        QueryResult::Stream(mut stream) => {
            let mut items = Vec::new();
            while let Some(item) = stream.next().await {
                items.push(item);
            }
            items
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_consume_query_result() {
        let single = QueryResult::Single(42);
        assert_eq!(consume_query_result(single).await, vec![42]);
        
        let multiple = QueryResult::Multiple(vec![1, 2, 3]);
        assert_eq!(consume_query_result(multiple).await, vec![1, 2, 3]);
    }
}