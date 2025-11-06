//! Workflow definitions for Person domain
//!
//! Defines the structure and behavior of person-related workflows including
//! state machines, transitions, and workflow metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Unique identifier for workflows
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(pub Uuid);

impl WorkflowId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for WorkflowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Workflow states for person-related processes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowState {
    /// Workflow is pending initiation
    Pending,
    /// Workflow is actively running
    Running,
    /// Workflow is waiting for external input
    Waiting,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with errors
    Failed,
    /// Workflow was cancelled
    Cancelled,
    /// Workflow was suspended
    Suspended,
}

/// Types of workflows supported in Person domain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonWorkflowType {
    /// Person onboarding workflow
    PersonOnboarding,
    /// Identity verification workflow
    IdentityVerification,
    /// Employment lifecycle workflow
    EmploymentLifecycle,
    /// Skills certification workflow
    SkillsCertification,
    /// Network connection workflow
    NetworkConnection,
    /// Privacy compliance workflow
    PrivacyCompliance,
    /// Data migration workflow
    DataMigration,
    /// Account deactivation workflow
    AccountDeactivation,
}

/// Workflow node representing a step in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// Unique identifier for the node
    pub id: String,
    /// Human-readable name of the node
    pub name: String,
    /// Type of node (action, condition, etc.)
    pub node_type: NodeType,
    /// Configuration for the node
    pub configuration: NodeConfiguration,
    /// Timeout for the node execution
    pub timeout: Option<std::time::Duration>,
    /// Retry policy for the node
    pub retry_policy: Option<RetryPolicy>,
}

/// Types of workflow nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// Service invocation node
    ServiceInvocation {
        service: String,
        operation: String,
        input_mapping: Option<HashMap<String, String>>,
        output_mapping: Option<HashMap<String, String>>,
    },
    /// Decision gateway node
    DecisionGateway {
        condition: ConditionExpression,
        branches: Vec<Branch>,
    },
    /// Parallel execution gateway
    ParallelGateway {
        branches: Vec<ParallelBranch>,
    },
    /// Wait node for external events
    WaitForEvent {
        event_pattern: String,
        timeout: std::time::Duration,
    },
    /// Human task node
    HumanTask {
        assignee: Option<String>,
        form_definition: Option<String>,
        due_date: Option<DateTime<Utc>>,
    },
    /// Script execution node
    Script {
        script_type: ScriptType,
        script_content: String,
    },
}

/// Configuration for workflow nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfiguration {
    /// Input parameters for the node
    pub inputs: HashMap<String, serde_json::Value>,
    /// Output parameters from the node
    pub outputs: HashMap<String, serde_json::Value>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for NodeConfiguration {
    fn default() -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Condition expressions for decision nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionExpression {
    /// Simple boolean expression
    Boolean(bool),
    /// Comparison expression
    Comparison {
        left: String,
        operator: ComparisonOperator,
        right: serde_json::Value,
    },
    /// Logical expression
    Logical {
        operator: LogicalOperator,
        operands: Vec<Box<ConditionExpression>>,
    },
    /// Script-based condition
    Script {
        script_type: ScriptType,
        script_content: String,
    },
}

/// Comparison operators for conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
}

/// Logical operators for conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Branch in a decision gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub condition: Option<ConditionExpression>,
    pub target_node_id: String,
}

/// Branch in a parallel gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelBranch {
    pub name: String,
    pub nodes: Vec<String>,
}

/// Script types supported in workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptType {
    /// JavaScript
    JavaScript,
    /// Python
    Python,
    /// Rust expression
    RustExpression,
}

/// Retry policy for workflow nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff: std::time::Duration,
    /// Maximum backoff duration
    pub max_backoff: std::time::Duration,
    /// Backoff multiplier
    pub multiplier: f64,
    /// Conditions that trigger retries
    pub retry_conditions: Vec<RetryCondition>,
}

/// Conditions that determine if a retry should occur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    /// Retry on any error
    AnyError,
    /// Retry on specific error types
    ErrorType(String),
    /// Retry on timeout
    Timeout,
    /// Retry on custom condition
    Custom(ConditionExpression),
}

/// Workflow transition between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Condition for the transition
    pub condition: Option<ConditionExpression>,
    /// Priority of the transition (higher = higher priority)
    pub priority: i32,
}

/// Complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Unique identifier for the workflow
    pub id: WorkflowId,
    /// Human-readable name
    pub name: String,
    /// Version of the workflow
    pub version: String,
    /// Workflow type
    pub workflow_type: PersonWorkflowType,
    /// Description of the workflow
    pub description: Option<String>,
    /// List of workflow nodes
    pub nodes: Vec<WorkflowNode>,
    /// List of transitions between nodes
    pub transitions: Vec<WorkflowTransition>,
    /// Starting node ID
    pub start_node_id: String,
    /// End node IDs
    pub end_node_ids: Vec<String>,
    /// Global workflow configuration
    pub global_config: WorkflowGlobalConfig,
    /// Workflow metadata
    pub metadata: WorkflowMetadata,
}

/// Global configuration for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGlobalConfig {
    /// Maximum execution time for the entire workflow
    pub max_execution_time: Option<std::time::Duration>,
    /// Variables available throughout the workflow
    pub variables: HashMap<String, serde_json::Value>,
    /// Error handling strategy
    pub error_handling: ErrorHandlingStrategy,
    /// Logging configuration
    pub logging_config: LoggingConfig,
}

impl Default for WorkflowGlobalConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Some(std::time::Duration::from_secs(3600)), // 1 hour
            variables: HashMap::new(),
            error_handling: ErrorHandlingStrategy::FailFast,
            logging_config: LoggingConfig::default(),
        }
    }
}

/// Error handling strategies for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandlingStrategy {
    /// Fail the entire workflow on first error
    FailFast,
    /// Continue execution and collect errors
    ContinueOnError,
    /// Retry failed nodes according to their retry policy
    RetryOnError,
    /// Route to error handling nodes
    RouteToErrorHandler,
}

/// Logging configuration for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,
    /// Log node inputs
    pub log_inputs: bool,
    /// Log node outputs
    pub log_outputs: bool,
    /// Log transitions
    pub log_transitions: bool,
    /// Log performance metrics
    pub log_performance: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            log_inputs: false,
            log_outputs: false,
            log_transitions: true,
            log_performance: true,
        }
    }
}

/// Log levels for workflow logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Workflow metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Creator of the workflow
    pub created_by: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Additional metadata
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

impl Default for WorkflowMetadata {
    fn default() -> Self {
        Self {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "system".to_string(),
            tags: Vec::new(),
            custom_metadata: HashMap::new(),
        }
    }
}

/// Current state of a workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    /// Unique identifier for the instance
    pub instance_id: Uuid,
    /// Workflow definition ID
    pub workflow_id: WorkflowId,
    /// Current state
    pub state: WorkflowState,
    /// Current node ID
    pub current_node_id: Option<String>,
    /// Execution context
    pub context: WorkflowContext,
    /// Execution history
    pub execution_history: Vec<WorkflowExecution>,
    /// Start time
    pub started_at: Option<DateTime<Utc>>,
    /// End time
    pub ended_at: Option<DateTime<Utc>>,
    /// Error information if failed
    pub error: Option<WorkflowError>,
}

/// Execution context for workflow instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Input data for the workflow
    pub input_data: HashMap<String, serde_json::Value>,
    /// Current variables in the workflow
    pub variables: HashMap<String, serde_json::Value>,
    /// Output data from the workflow
    pub output_data: HashMap<String, serde_json::Value>,
    /// External correlation ID
    pub correlation_id: String,
    /// Actor initiating the workflow
    pub actor: String,
}

/// Record of workflow node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Node that was executed
    pub node_id: String,
    /// Execution start time
    pub started_at: DateTime<Utc>,
    /// Execution end time
    pub ended_at: Option<DateTime<Utc>>,
    /// Execution status
    pub status: ExecutionStatus,
    /// Input data for the node
    pub input_data: HashMap<String, serde_json::Value>,
    /// Output data from the node
    pub output_data: HashMap<String, serde_json::Value>,
    /// Error information if failed
    pub error: Option<String>,
    /// Performance metrics
    pub metrics: ExecutionMetrics,
}

/// Status of node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Skipped,
    Retrying,
}

/// Performance metrics for node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: Option<u64>,
    /// CPU usage percentage
    pub cpu_usage_percent: Option<f64>,
    /// Number of retry attempts
    pub retry_count: u32,
}

/// Workflow error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Node where error occurred
    pub node_id: Option<String>,
    /// Stack trace or additional details
    pub details: Option<String>,
    /// Timestamp when error occurred
    pub timestamp: DateTime<Utc>,
    /// Whether the error is recoverable
    pub recoverable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_id_creation() {
        let id1 = WorkflowId::new();
        let id2 = WorkflowId::new();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_workflow_definition_creation() {
        let workflow = WorkflowDefinition {
            id: WorkflowId::new(),
            name: "Test Workflow".to_string(),
            version: "1.0".to_string(),
            workflow_type: PersonWorkflowType::PersonOnboarding,
            description: Some("Test workflow for person onboarding".to_string()),
            nodes: vec![],
            transitions: vec![],
            start_node_id: "start".to_string(),
            end_node_ids: vec!["end".to_string()],
            global_config: WorkflowGlobalConfig::default(),
            metadata: WorkflowMetadata::default(),
        };
        
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.version, "1.0");
        assert_eq!(workflow.workflow_type, PersonWorkflowType::PersonOnboarding);
    }
    
    #[test]
    fn test_condition_expression() {
        let condition = ConditionExpression::Comparison {
            left: "age".to_string(),
            operator: ComparisonOperator::GreaterThan,
            right: serde_json::json!(18),
        };
        
        if let ConditionExpression::Comparison { left, operator, right } = condition {
            assert_eq!(left, "age");
            assert!(matches!(operator, ComparisonOperator::GreaterThan));
            assert_eq!(right, serde_json::json!(18));
        } else {
            panic!("Expected comparison condition");
        }
    }
    
    #[test]
    fn test_workflow_instance() {
        let mut instance = WorkflowInstance {
            instance_id: Uuid::now_v7(),
            workflow_id: WorkflowId::new(),
            state: WorkflowState::Pending,
            current_node_id: None,
            context: WorkflowContext {
                input_data: HashMap::new(),
                variables: HashMap::new(),
                output_data: HashMap::new(),
                correlation_id: "test-correlation".to_string(),
                actor: "test-user".to_string(),
            },
            execution_history: Vec::new(),
            started_at: None,
            ended_at: None,
            error: None,
        };
        
        assert_eq!(instance.state, WorkflowState::Pending);
        assert!(instance.started_at.is_none());
        
        // Start the workflow
        instance.state = WorkflowState::Running;
        instance.started_at = Some(Utc::now());
        instance.current_node_id = Some("start".to_string());
        
        assert_eq!(instance.state, WorkflowState::Running);
        assert!(instance.started_at.is_some());
        assert_eq!(instance.current_node_id, Some("start".to_string()));
    }
}