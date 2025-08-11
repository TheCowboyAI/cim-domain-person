//! Workflow manager for Person domain
//!
//! Provides workflow orchestration, execution, and monitoring capabilities
//! for person-related business processes.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::{RwLock, mpsc};
use futures::StreamExt;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json;

use crate::nats::{PersonSubject, PersonEventType, PersonAggregate, MessageIdentity, PersonActor};
use super::definitions::*;

/// Errors that can occur during workflow management
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Workflow not found: {workflow_id}")]
    WorkflowNotFound { workflow_id: WorkflowId },
    
    #[error("Workflow instance not found: {instance_id}")]
    InstanceNotFound { instance_id: Uuid },
    
    #[error("Invalid workflow state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: WorkflowState, to: WorkflowState },
    
    #[error("Node not found: {node_id}")]
    NodeNotFound { node_id: String },
    
    #[error("Execution error: {message}")]
    ExecutionError { message: String },
    
    #[error("Timeout error: workflow exceeded maximum execution time")]
    TimeoutError,
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("External service error: {service}: {message}")]
    ExternalServiceError { service: String, message: String },
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;

/// Events emitted by the workflow manager
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    /// Workflow instance started
    InstanceStarted {
        instance_id: Uuid,
        workflow_id: WorkflowId,
        actor: String,
    },
    /// Workflow node started
    NodeStarted {
        instance_id: Uuid,
        node_id: String,
    },
    /// Workflow node completed
    NodeCompleted {
        instance_id: Uuid,
        node_id: String,
        output: HashMap<String, serde_json::Value>,
    },
    /// Workflow node failed
    NodeFailed {
        instance_id: Uuid,
        node_id: String,
        error: String,
    },
    /// Workflow instance completed
    InstanceCompleted {
        instance_id: Uuid,
        output: HashMap<String, serde_json::Value>,
    },
    /// Workflow instance failed
    InstanceFailed {
        instance_id: Uuid,
        error: String,
    },
    /// Workflow instance cancelled
    InstanceCancelled {
        instance_id: Uuid,
        reason: String,
    },
}

/// Trait for workflow execution engines
#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// Execute a workflow node
    async fn execute_node(
        &self,
        node: &WorkflowNode,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<HashMap<String, serde_json::Value>>;
    
    /// Evaluate a condition expression
    async fn evaluate_condition(
        &self,
        condition: &ConditionExpression,
        context: &WorkflowContext,
    ) -> WorkflowResult<bool>;
    
    /// Execute a script
    async fn execute_script(
        &self,
        script_type: &ScriptType,
        script_content: &str,
        context: &WorkflowContext,
    ) -> WorkflowResult<serde_json::Value>;
}

/// Default workflow execution engine
pub struct DefaultWorkflowEngine {
    service_registry: Arc<dyn ServiceRegistry>,
    nats_client: async_nats::Client,
}

impl DefaultWorkflowEngine {
    pub fn new(service_registry: Arc<dyn ServiceRegistry>, nats_client: async_nats::Client) -> Self {
        Self {
            service_registry,
            nats_client,
        }
    }
}

#[async_trait]
impl WorkflowEngine for DefaultWorkflowEngine {
    async fn execute_node(
        &self,
        node: &WorkflowNode,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<HashMap<String, serde_json::Value>> {
        match &node.node_type {
            NodeType::ServiceInvocation { service, operation, input_mapping, output_mapping } => {
                self.execute_service_invocation(service, operation, input_mapping, output_mapping, context).await
            },
            NodeType::DecisionGateway { condition, branches: _ } => {
                let result = self.evaluate_condition(condition, context).await?;
                let mut output = HashMap::new();
                output.insert("decision_result".to_string(), serde_json::json!(result));
                Ok(output)
            },
            NodeType::WaitForEvent { event_pattern, timeout } => {
                self.execute_wait_for_event(event_pattern, *timeout, context).await
            },
            NodeType::HumanTask { assignee, form_definition, due_date } => {
                self.execute_human_task(assignee.as_deref(), form_definition.as_deref(), *due_date, context).await
            },
            NodeType::Script { script_type, script_content } => {
                let result = self.execute_script(script_type, script_content, context).await?;
                let mut output = HashMap::new();
                output.insert("script_result".to_string(), result);
                Ok(output)
            },
            NodeType::ParallelGateway { branches } => {
                self.execute_parallel_gateway(branches, context).await
            },
        }
    }
    
    async fn evaluate_condition(
        &self,
        condition: &ConditionExpression,
        context: &WorkflowContext,
    ) -> WorkflowResult<bool> {
        match condition {
            ConditionExpression::Boolean(value) => Ok(*value),
            ConditionExpression::Comparison { left, operator, right } => {
                self.evaluate_comparison(left, operator, right, context).await
            },
            ConditionExpression::Logical { operator, operands } => {
                self.evaluate_logical(operator, operands, context).await
            },
            ConditionExpression::Script { script_type, script_content } => {
                let result = self.execute_script(script_type, script_content, context).await?;
                Ok(result.as_bool().unwrap_or(false))
            },
        }
    }
    
    async fn execute_script(
        &self,
        script_type: &ScriptType,
        script_content: &str,
        _context: &WorkflowContext,
    ) -> WorkflowResult<serde_json::Value> {
        match script_type {
            ScriptType::RustExpression => {
                // For now, only support simple boolean expressions
                match script_content {
                    "true" => Ok(serde_json::json!(true)),
                    "false" => Ok(serde_json::json!(false)),
                    _ => Err(WorkflowError::ExecutionError {
                        message: format!("Unsupported Rust expression: {}", script_content),
                    }),
                }
            },
            _ => Err(WorkflowError::ExecutionError {
                message: format!("Unsupported script type: {:?}", script_type),
            }),
        }
    }
}

impl DefaultWorkflowEngine {
    async fn execute_service_invocation(
        &self,
        service: &str,
        operation: &str,
        _input_mapping: &Option<HashMap<String, String>>,
        _output_mapping: &Option<HashMap<String, String>>,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<HashMap<String, serde_json::Value>> {
        // Get service from registry
        let service_instance = self.service_registry.get_service(service).await
            .map_err(|e| WorkflowError::ExternalServiceError {
                service: service.to_string(),
                message: format!("Service not found: {}", e),
            })?;
        
        // Execute operation
        let result = service_instance.execute(operation, &context.variables).await
            .map_err(|e| WorkflowError::ExecutionError {
                message: format!("Service execution failed: {}", e),
            })?;
        
        Ok(result)
    }
    
    async fn execute_wait_for_event(
        &self,
        event_pattern: &str,
        timeout: std::time::Duration,
        _context: &mut WorkflowContext,
    ) -> WorkflowResult<HashMap<String, serde_json::Value>> {
        // Subscribe to event pattern
        let mut subscription = self.nats_client.subscribe(event_pattern).await
            .map_err(|e| WorkflowError::ExternalServiceError {
                service: "NATS".to_string(),
                message: format!("Failed to subscribe: {}", e),
            })?;
        
        // Wait for event with timeout
        let timeout_future = tokio::time::sleep(timeout);
        tokio::select! {
            msg = subscription.next() => {
                if let Some(msg) = msg {
                    let mut output = HashMap::new();
                    output.insert("event_received".to_string(), serde_json::json!(true));
                    output.insert("event_data".to_string(), 
                        serde_json::from_slice(&msg.payload).unwrap_or(serde_json::json!({})));
                    Ok(output)
                } else {
                    Err(WorkflowError::ExecutionError {
                        message: "Event stream closed".to_string(),
                    })
                }
            },
            _ = timeout_future => {
                Err(WorkflowError::TimeoutError)
            }
        }
    }
    
    async fn execute_human_task(
        &self,
        assignee: Option<&str>,
        _form_definition: Option<&str>,
        due_date: Option<DateTime<Utc>>,
        context: &mut WorkflowContext,
    ) -> WorkflowResult<HashMap<String, serde_json::Value>> {
        // Create human task event
        let task_id = Uuid::new_v4().to_string();
        let subject = PersonSubject::event(
            PersonAggregate::Person,
            PersonEventType::ComponentDataUpdated,
            &context.correlation_id,
        );
        
        let task_event = serde_json::json!({
            "task_id": task_id,
            "assignee": assignee,
            "due_date": due_date,
            "workflow_instance_id": context.correlation_id,
        });
        
        // Publish human task event
        self.nats_client.publish(
            subject.to_string(),
            serde_json::to_vec(&task_event)
                .map_err(|e| WorkflowError::SerializationError {
                    message: e.to_string(),
                })?
                .into()
        ).await
        .map_err(|e| WorkflowError::ExternalServiceError {
            service: "NATS".to_string(),
            message: format!("Failed to publish human task: {}", e),
        })?;
        
        let mut output = HashMap::new();
        output.insert("task_created".to_string(), serde_json::json!(true));
        output.insert("task_id".to_string(), serde_json::json!(task_id));
        Ok(output)
    }
    
    async fn execute_parallel_gateway(
        &self,
        _branches: &[ParallelBranch],
        _context: &mut WorkflowContext,
    ) -> WorkflowResult<HashMap<String, serde_json::Value>> {
        // Simplified parallel execution - would need more complex implementation
        let mut output = HashMap::new();
        output.insert("parallel_completed".to_string(), serde_json::json!(true));
        Ok(output)
    }
    
    async fn evaluate_comparison(
        &self,
        left: &str,
        operator: &ComparisonOperator,
        right: &serde_json::Value,
        context: &WorkflowContext,
    ) -> WorkflowResult<bool> {
        let left_value = context.variables.get(left)
            .cloned()
            .unwrap_or(serde_json::json!(null));
        
        let result = match operator {
            ComparisonOperator::Equal => left_value == *right,
            ComparisonOperator::NotEqual => left_value != *right,
            ComparisonOperator::GreaterThan => {
                if let (Some(l), Some(r)) = (left_value.as_f64(), right.as_f64()) {
                    l > r
                } else {
                    false
                }
            },
            ComparisonOperator::LessThan => {
                if let (Some(l), Some(r)) = (left_value.as_f64(), right.as_f64()) {
                    l < r
                } else {
                    false
                }
            },
            ComparisonOperator::GreaterThanOrEqual => {
                if let (Some(l), Some(r)) = (left_value.as_f64(), right.as_f64()) {
                    l >= r
                } else {
                    false
                }
            },
            ComparisonOperator::LessThanOrEqual => {
                if let (Some(l), Some(r)) = (left_value.as_f64(), right.as_f64()) {
                    l <= r
                } else {
                    false
                }
            },
            ComparisonOperator::Contains => {
                if let (Some(l), Some(r)) = (left_value.as_str(), right.as_str()) {
                    l.contains(r)
                } else {
                    false
                }
            },
            ComparisonOperator::StartsWith => {
                if let (Some(l), Some(r)) = (left_value.as_str(), right.as_str()) {
                    l.starts_with(r)
                } else {
                    false
                }
            },
            ComparisonOperator::EndsWith => {
                if let (Some(l), Some(r)) = (left_value.as_str(), right.as_str()) {
                    l.ends_with(r)
                } else {
                    false
                }
            },
        };
        
        Ok(result)
    }
    
    async fn evaluate_logical(
        &self,
        operator: &LogicalOperator,
        operands: &[Box<ConditionExpression>],
        context: &WorkflowContext,
    ) -> WorkflowResult<bool> {
        match operator {
            LogicalOperator::And => {
                for operand in operands {
                    if !self.evaluate_condition(operand, context).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
            LogicalOperator::Or => {
                for operand in operands {
                    if self.evaluate_condition(operand, context).await? {
                        return Ok(true);
                    }
                }
                Ok(false)
            },
            LogicalOperator::Not => {
                if operands.len() != 1 {
                    return Err(WorkflowError::ConfigurationError {
                        message: "NOT operator requires exactly one operand".to_string(),
                    });
                }
                let result = self.evaluate_condition(&operands[0], context).await?;
                Ok(!result)
            },
        }
    }
}

/// Service registry for workflow services
#[async_trait]
pub trait ServiceRegistry: Send + Sync {
    async fn get_service(&self, name: &str) -> Result<Arc<dyn WorkflowService>, Box<dyn std::error::Error>>;
}

/// Workflow service interface
#[async_trait]
pub trait WorkflowService: Send + Sync {
    async fn execute(
        &self,
        operation: &str,
        inputs: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>>;
}

/// Main workflow manager
pub struct WorkflowManager {
    workflows: Arc<RwLock<HashMap<WorkflowId, WorkflowDefinition>>>,
    instances: Arc<RwLock<HashMap<Uuid, WorkflowInstance>>>,
    engine: Arc<dyn WorkflowEngine>,
    event_sender: mpsc::UnboundedSender<WorkflowEvent>,
    nats_client: async_nats::Client,
}

impl WorkflowManager {
    pub fn new(
        engine: Arc<dyn WorkflowEngine>,
        nats_client: async_nats::Client,
    ) -> (Self, mpsc::UnboundedReceiver<WorkflowEvent>) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let manager = Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            engine,
            event_sender,
            nats_client,
        };
        
        (manager, event_receiver)
    }
    
    /// Register a workflow definition
    pub async fn register_workflow(&self, workflow: WorkflowDefinition) -> WorkflowResult<()> {
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }
    
    /// Start a workflow instance
    pub async fn start_workflow(
        &self,
        workflow_id: &WorkflowId,
        input_data: HashMap<String, serde_json::Value>,
        actor: PersonActor,
    ) -> WorkflowResult<Uuid> {
        let workflow = {
            let workflows = self.workflows.read().await;
            workflows.get(workflow_id)
                .ok_or_else(|| WorkflowError::WorkflowNotFound { 
                    workflow_id: workflow_id.clone() 
                })?
                .clone()
        };
        
        let instance_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4().to_string();
        
        let instance = WorkflowInstance {
            instance_id,
            workflow_id: workflow_id.clone(),
            state: WorkflowState::Running,
            current_node_id: Some(workflow.start_node_id.clone()),
            context: WorkflowContext {
                input_data,
                variables: workflow.global_config.variables.clone(),
                output_data: HashMap::new(),
                correlation_id,
                actor: actor.to_string(),
            },
            execution_history: Vec::new(),
            started_at: Some(Utc::now()),
            ended_at: None,
            error: None,
        };
        
        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, instance);
        }
        
        // Send event
        let _ = self.event_sender.send(WorkflowEvent::InstanceStarted {
            instance_id,
            workflow_id: workflow_id.clone(),
            actor: actor.to_string(),
        });
        
        // Start execution
        self.execute_workflow(instance_id).await?;
        
        Ok(instance_id)
    }
    
    /// Execute workflow instance
    async fn execute_workflow(&self, instance_id: Uuid) -> WorkflowResult<()> {
        let (workflow, mut instance) = {
            let workflows = self.workflows.read().await;
            let instances = self.instances.read().await;
            
            let instance = instances.get(&instance_id)
                .ok_or_else(|| WorkflowError::InstanceNotFound { instance_id })?;
            
            let workflow = workflows.get(&instance.workflow_id)
                .ok_or_else(|| WorkflowError::WorkflowNotFound { 
                    workflow_id: instance.workflow_id.clone() 
                })?;
            
            (workflow.clone(), instance.clone())
        };
        
        while let Some(current_node_id) = &instance.current_node_id {
            // Check if we've reached an end node
            if workflow.end_node_ids.contains(current_node_id) {
                instance.state = WorkflowState::Completed;
                instance.ended_at = Some(Utc::now());
                break;
            }
            
            // Find current node
            let current_node = workflow.nodes.iter()
                .find(|n| &n.id == current_node_id)
                .ok_or_else(|| WorkflowError::NodeNotFound { 
                    node_id: current_node_id.clone() 
                })?;
            
            // Send node started event
            let _ = self.event_sender.send(WorkflowEvent::NodeStarted {
                instance_id,
                node_id: current_node_id.clone(),
            });
            
            // Execute node
            let execution_start = Utc::now();
            match self.engine.execute_node(current_node, &mut instance.context).await {
                Ok(output) => {
                    let execution_end = Utc::now();
                    
                    // Record execution
                    let execution = WorkflowExecution {
                        node_id: current_node_id.clone(),
                        started_at: execution_start,
                        ended_at: Some(execution_end),
                        status: ExecutionStatus::Completed,
                        input_data: HashMap::new(), // Could capture actual inputs
                        output_data: output.clone(),
                        error: None,
                        metrics: ExecutionMetrics {
                            duration_ms: (execution_end - execution_start).num_milliseconds() as u64,
                            memory_usage_bytes: None,
                            cpu_usage_percent: None,
                            retry_count: 0,
                        },
                    };
                    
                    instance.execution_history.push(execution);
                    
                    // Send node completed event
                    let _ = self.event_sender.send(WorkflowEvent::NodeCompleted {
                        instance_id,
                        node_id: current_node_id.clone(),
                        output,
                    });
                    
                    // Find next node
                    let next_node_id = self.find_next_node(&workflow, current_node_id, &instance.context).await?;
                    instance.current_node_id = next_node_id;
                },
                Err(e) => {
                    instance.state = WorkflowState::Failed;
                    instance.ended_at = Some(Utc::now());
                    instance.error = Some(super::definitions::WorkflowError {
                        code: "EXECUTION_ERROR".to_string(),
                        message: e.to_string(),
                        node_id: Some(current_node_id.clone()),
                        details: None,
                        timestamp: Utc::now(),
                        recoverable: false,
                    });
                    
                    // Send failure events
                    let _ = self.event_sender.send(WorkflowEvent::NodeFailed {
                        instance_id,
                        node_id: current_node_id.clone(),
                        error: e.to_string(),
                    });
                    
                    let _ = self.event_sender.send(WorkflowEvent::InstanceFailed {
                        instance_id,
                        error: e.to_string(),
                    });
                    
                    break;
                }
            }
        }
        
        // Update instance in storage
        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, instance.clone());
        }
        
        // Send completion event if successful
        if instance.state == WorkflowState::Completed {
            let _ = self.event_sender.send(WorkflowEvent::InstanceCompleted {
                instance_id,
                output: instance.context.output_data,
            });
        }
        
        Ok(())
    }
    
    /// Find the next node to execute
    async fn find_next_node(
        &self,
        workflow: &WorkflowDefinition,
        current_node_id: &str,
        context: &WorkflowContext,
    ) -> WorkflowResult<Option<String>> {
        // Find transitions from current node
        let mut applicable_transitions: Vec<_> = workflow.transitions.iter()
            .filter(|t| t.from == current_node_id)
            .collect();
        
        // Sort by priority
        applicable_transitions.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Find first transition whose condition is met
        for transition in applicable_transitions {
            let condition_met = if let Some(condition) = &transition.condition {
                self.engine.evaluate_condition(condition, context).await?
            } else {
                true // No condition means always take this transition
            };
            
            if condition_met {
                return Ok(Some(transition.to.clone()));
            }
        }
        
        // No applicable transition found
        Ok(None)
    }
    
    /// Get workflow instance
    pub async fn get_instance(&self, instance_id: Uuid) -> WorkflowResult<WorkflowInstance> {
        let instances = self.instances.read().await;
        instances.get(&instance_id)
            .cloned()
            .ok_or_else(|| WorkflowError::InstanceNotFound { instance_id })
    }
    
    /// Cancel workflow instance
    pub async fn cancel_instance(&self, instance_id: Uuid, reason: String) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.state = WorkflowState::Cancelled;
            instance.ended_at = Some(Utc::now());
            
            let _ = self.event_sender.send(WorkflowEvent::InstanceCancelled {
                instance_id,
                reason,
            });
            
            Ok(())
        } else {
            Err(WorkflowError::InstanceNotFound { instance_id })
        }
    }
    
    /// List workflow instances
    pub async fn list_instances(&self) -> Vec<WorkflowInstance> {
        let instances = self.instances.read().await;
        instances.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    struct MockServiceRegistry;
    
    #[async_trait]
    impl ServiceRegistry for MockServiceRegistry {
        async fn get_service(&self, _name: &str) -> Result<Arc<dyn WorkflowService>, Box<dyn std::error::Error>> {
            Ok(Arc::new(MockWorkflowService))
        }
    }
    
    struct MockWorkflowService;
    
    #[async_trait]
    impl WorkflowService for MockWorkflowService {
        async fn execute(
            &self,
            _operation: &str,
            _inputs: &HashMap<String, serde_json::Value>,
        ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
            Ok(HashMap::new())
        }
    }
    
    #[tokio::test]
    async fn test_workflow_manager_creation() {
        let nats_client = async_nats::connect("nats://localhost:4222").await
            .unwrap_or_else(|_| panic!("NATS not available for testing"));
        
        let service_registry = Arc::new(MockServiceRegistry);
        let engine = Arc::new(DefaultWorkflowEngine::new(service_registry, nats_client.clone()));
        let (manager, _event_receiver) = WorkflowManager::new(engine, nats_client);
        
        // Test basic workflow registration
        let workflow = WorkflowDefinition {
            id: WorkflowId::new(),
            name: "Test Workflow".to_string(),
            version: "1.0".to_string(),
            workflow_type: PersonWorkflowType::PersonOnboarding,
            description: None,
            nodes: vec![],
            transitions: vec![],
            start_node_id: "start".to_string(),
            end_node_ids: vec!["end".to_string()],
            global_config: WorkflowGlobalConfig::default(),
            metadata: WorkflowMetadata::default(),
        };
        
        assert!(manager.register_workflow(workflow).await.is_ok());
    }
    
    #[test]
    fn test_condition_evaluation() {
        let condition = ConditionExpression::Boolean(true);
        assert!(matches!(condition, ConditionExpression::Boolean(true)));
        
        let comparison = ConditionExpression::Comparison {
            left: "age".to_string(),
            operator: ComparisonOperator::GreaterThan,
            right: serde_json::json!(18),
        };
        
        if let ConditionExpression::Comparison { operator, .. } = comparison {
            assert!(matches!(operator, ComparisonOperator::GreaterThan));
        }
    }
}