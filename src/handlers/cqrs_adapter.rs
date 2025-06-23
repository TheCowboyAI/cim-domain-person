//! CQRS Adapters for Person Domain
//!
//! This module provides adapters that implement the standard CQRS CommandHandler
//! and QueryHandler traits while delegating to the existing person domain handlers.
//! This allows the Person domain to participate in correlation/causation tracking.

use cim_domain::{
    Command, CommandEnvelope, CommandHandler, CommandAcknowledgment, CommandStatus,
    Query, QueryEnvelope, QueryHandler, QueryResponse,
    EntityId, DomainResult,
};
use serde::{Deserialize, Serialize};
use crate::{
    PersonId, PersonCommand, PersonEvent, PersonQuery,
    aggregate::Person,
};
use super::command_handlers::handle_person_command;
use super::query_handlers::PersonQueryHandler as QueryHandlerImpl;

/// Wrapper for PersonCommand that implements the Command trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonCommandWrapper {
    pub person_id: PersonId,
    pub command: PersonCommand,
}

impl Command for PersonCommandWrapper {
    type Aggregate = Person;
    
    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        // PersonId doesn't directly convert to EntityId, so we return None
        // The actual ID is handled by the wrapper
        None
    }
}

/// CQRS-compliant command handler for Person domain
pub struct PersonCommandHandler {
    // In a real implementation, this would have a repository
}

impl PersonCommandHandler {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Handle a command and return events (for compatibility with existing code)
    pub async fn handle_command(
        &self,
        aggregate: &mut Person,
        command: PersonCommand,
    ) -> DomainResult<Vec<PersonEvent>> {
        handle_person_command(aggregate, command).await
    }
}

impl CommandHandler<PersonCommandWrapper> for PersonCommandHandler {
    fn handle(&mut self, envelope: CommandEnvelope<PersonCommandWrapper>) -> CommandAcknowledgment {
        let command_id = envelope.id;
        let correlation_id = envelope.identity.correlation_id.clone();
        let _wrapper = envelope.command;
        
        // In a real implementation, we would:
        // 1. Load the aggregate from repository
        // 2. Apply the command
        // 3. Save the aggregate
        // 4. Publish events
        
        // For now, we just acknowledge the command
        CommandAcknowledgment {
            command_id,
            correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS adapter for PersonQueryHandler
pub struct PersonQueryHandlerAdapter {
    inner: QueryHandlerImpl,
}

impl PersonQueryHandlerAdapter {
    pub fn new() -> Self {
        Self {
            inner: QueryHandlerImpl::new(),
        }
    }
}

impl QueryHandler<PersonQuery> for PersonQueryHandlerAdapter {
    fn handle(&self, envelope: QueryEnvelope<PersonQuery>) -> QueryResponse {
        let query_id = envelope.id;
        let correlation_id = envelope.identity.correlation_id.clone();
        
        // Process the query synchronously (blocking on async)
        let runtime = tokio::runtime::Handle::current();
        let result = runtime.block_on(async {
            self.inner.handle_query(envelope.query).await
        });
        
        match result {
            Ok(value) => QueryResponse {
                query_id: envelope.identity.message_id,
                correlation_id,
                result: serde_json::to_value(value).unwrap_or_else(|e| {
                    serde_json::json!({
                        "error": format!("Failed to serialize result: {}", e)
                    })
                }),
            },
            Err(error) => QueryResponse {
                query_id: envelope.identity.message_id,
                correlation_id,
                result: serde_json::json!({
                    "error": error.to_string()
                }),
            },
        }
    }
}

impl Query for PersonQuery {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::NameComponent;
    
    #[test]
    fn test_person_command_wrapper() {
        let person_id = PersonId::new();
        let command = PersonCommand::UpdateName {
            person_id: *person_id.as_uuid(),
            name: NameComponent::simple("John".to_string(), "Doe".to_string()),
        };
        let wrapper = PersonCommandWrapper { person_id, command };
        
        // Test serialization
        let serialized = serde_json::to_string(&wrapper).unwrap();
        let deserialized: PersonCommandWrapper = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.person_id, person_id);
    }
    
    #[test]
    fn test_person_query_trait() {
        let query = PersonQuery::GetPersonById { 
            person_id: PersonId::new() 
        };
        
        // Verify Query trait is implemented
        let _: &dyn Query = &query;
    }
} 