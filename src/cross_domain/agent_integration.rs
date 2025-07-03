//! Integration with Agent domain for AI agent associations

use crate::aggregate::PersonId;
use crate::events::PersonEvent;
use crate::infrastructure::PersonRepository;
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};

/// Events from Agent domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentDomainEvent {
    AgentCreated {
        agent_id: String,
        agent_type: AgentType,
        name: String,
        capabilities: Vec<AgentCapability>,
        created_at: DateTime<Utc>,
    },
    AgentAssignedToPerson {
        agent_id: String,
        person_id: PersonId,
        assignment_type: AssignmentType,
        permissions: Vec<AgentPermission>,
        assigned_at: DateTime<Utc>,
    },
    AgentUnassignedFromPerson {
        agent_id: String,
        person_id: PersonId,
        reason: Option<String>,
        unassigned_at: DateTime<Utc>,
    },
    AgentInteractionCompleted {
        agent_id: String,
        person_id: PersonId,
        interaction_type: InteractionType,
        outcome: InteractionOutcome,
        duration_seconds: u64,
        timestamp: DateTime<Utc>,
    },
    AgentLearningFromPerson {
        agent_id: String,
        person_id: PersonId,
        learning_type: LearningType,
        data_points: u32,
        improvement_metrics: Option<ImprovementMetrics>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    PersonalAssistant,
    ResearchAssistant,
    CodingAssistant,
    WritingAssistant,
    DataAnalyst,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentCapability {
    NaturalLanguageProcessing,
    CodeGeneration,
    DataAnalysis,
    Research,
    Scheduling,
    EmailManagement,
    DocumentProcessing,
    Translation,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentType {
    Exclusive,    // Agent works only for this person
    Shared,       // Agent can work for multiple people
    Temporary,    // Time-limited assignment
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentPermission {
    ReadPersonalData,
    WritePersonalData,
    AccessCalendar,
    SendEmails,
    AccessDocuments,
    MakeDecisions,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    Query,
    Task,
    Conversation,
    Learning,
    Feedback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionOutcome {
    Successful { satisfaction_score: Option<f32> },
    Failed { error: String },
    Partial { completed_percentage: f32 },
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningType {
    Preferences,
    Communication,
    TaskPatterns,
    DecisionMaking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementMetrics {
    pub accuracy_improvement: f32,
    pub response_time_improvement: f32,
    pub satisfaction_improvement: f32,
}

/// Handler for Agent domain events
pub struct AgentEventHandler {
    person_repository: Arc<PersonRepository>,
}

impl AgentEventHandler {
    pub fn new(person_repository: Arc<PersonRepository>) -> Self {
        Self { person_repository }
    }
    
    /// Process an event from the Agent domain
    pub async fn handle_event(&self, event: AgentDomainEvent) -> DomainResult<Vec<PersonEvent>> {
        match event {
            AgentDomainEvent::AgentAssignedToPerson {
                person_id,
                agent_id,
                ..
            } => {
                self.handle_agent_assigned(person_id, agent_id).await
            }
            AgentDomainEvent::AgentUnassignedFromPerson {
                person_id,
                ..
            } => {
                self.handle_agent_unassigned(person_id).await
            }
            _ => Ok(vec![]),
        }
    }
    
    async fn handle_agent_assigned(
        &self,
        person_id: PersonId,
        agent_id: String,
    ) -> DomainResult<Vec<PersonEvent>> {
        // Verify person exists
        let person = self.person_repository.load(person_id).await?;
        if person.is_none() {
            return Err(DomainError::AggregateNotFound(format!("Person {person_id}")));
        }
        
        // For now, we'll just log this
        // In a real implementation, this might create an agent component
        tracing::info!("Agent {} assigned to person {}", agent_id, person_id);
        
        Ok(vec![])
    }
    
    async fn handle_agent_unassigned(
        &self,
        person_id: PersonId,
    ) -> DomainResult<Vec<PersonEvent>> {
        // Log the unassignment
        tracing::info!("Agent unassigned from person {}", person_id);
        
        Ok(vec![])
    }
}

/// Commands to send to Agent domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentDomainCommand {
    /// Request an agent for a person
    RequestAgent {
        person_id: PersonId,
        agent_type: AgentType,
        required_capabilities: Vec<AgentCapability>,
        assignment_type: AssignmentType,
    },
    
    /// Configure agent permissions
    ConfigureAgentPermissions {
        person_id: PersonId,
        agent_id: String,
        permissions: Vec<AgentPermission>,
    },
    
    /// Train agent on person's preferences
    TrainAgentOnPerson {
        person_id: PersonId,
        agent_id: String,
        training_data: TrainingDataType,
    },
    
    /// Get agent performance metrics
    GetAgentMetrics {
        person_id: PersonId,
        agent_id: String,
        time_period: TimePeriod,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrainingDataType {
    CommunicationStyle,
    TaskPreferences,
    DecisionPatterns,
    WorkSchedule,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Service for agent-related operations
pub struct AgentIntegrationService {
    person_repository: Arc<PersonRepository>,
}

impl AgentIntegrationService {
    pub fn new(person_repository: Arc<PersonRepository>) -> Self {
        Self { person_repository }
    }
    
    /// Check if a person has any assigned agents
    pub async fn person_has_agents(&self, _person_id: PersonId) -> DomainResult<bool> {
        // This would check if the person has any agent components
        // For now, return false
        Ok(false)
    }
    
    /// Get suitable agent type based on person's role
    pub async fn suggest_agent_type(&self, _person_id: PersonId) -> DomainResult<AgentType> {
        // This would analyze the person's components to suggest an agent type
        // For now, return PersonalAssistant as default
        Ok(AgentType::PersonalAssistant)
    }
} 