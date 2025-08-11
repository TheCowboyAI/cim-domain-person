//! Message identity and correlation support for Person domain NATS messaging
//!
//! Provides message correlation, causation tracking, and actor identification
//! following CIM messaging conventions for distributed tracing and debugging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a message
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for MessageId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<MessageId> for Uuid {
    fn from(id: MessageId) -> Self {
        id.0
    }
}

/// Correlation identifier for tracking related messages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CorrelationId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for CorrelationId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl From<Uuid> for CorrelationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }
}

/// Causation identifier for tracking message chains
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausationId(pub String);

impl CausationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    pub fn from_string(id: String) -> Self {
        Self(id)
    }
    
    pub fn from_message_id(message_id: &MessageId) -> Self {
        Self(message_id.to_string())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CausationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CausationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CausationId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for CausationId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl From<MessageId> for CausationId {
    fn from(id: MessageId) -> Self {
        Self(id.to_string())
    }
}

/// Actor types that can initiate person-related actions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonActor {
    /// Human user (person themselves)
    User(String),
    /// System service
    System(String),
    /// External API client
    ApiClient(String),
    /// Background job or process
    Job(String),
    /// HR management system
    HrSystem(String),
    /// Identity provider
    IdentityProvider(String),
    /// Skills assessment system
    SkillsSystem(String),
    /// Networking platform
    NetworkSystem(String),
    /// Employment verification system
    EmploymentSystem(String),
    /// Contact verification service
    ContactSystem(String),
    /// Data export/import system
    DataSystem(String),
    /// Privacy management system
    PrivacySystem(String),
    /// Unknown or unspecified actor
    Unknown,
}

impl PersonActor {
    pub fn user(id: &str) -> Self {
        Self::User(id.to_string())
    }
    
    pub fn system(name: &str) -> Self {
        Self::System(name.to_string())
    }
    
    pub fn api_client(id: &str) -> Self {
        Self::ApiClient(id.to_string())
    }
    
    pub fn job(name: &str) -> Self {
        Self::Job(name.to_string())
    }
    
    pub fn hr_system(id: &str) -> Self {
        Self::HrSystem(id.to_string())
    }
    
    pub fn identity_provider(name: &str) -> Self {
        Self::IdentityProvider(name.to_string())
    }
    
    pub fn skills_system(name: &str) -> Self {
        Self::SkillsSystem(name.to_string())
    }
    
    pub fn network_system(name: &str) -> Self {
        Self::NetworkSystem(name.to_string())
    }
    
    pub fn employment_system(name: &str) -> Self {
        Self::EmploymentSystem(name.to_string())
    }
    
    pub fn contact_system(name: &str) -> Self {
        Self::ContactSystem(name.to_string())
    }
}

impl std::fmt::Display for PersonActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonActor::User(id) => write!(f, "user:{}", id),
            PersonActor::System(name) => write!(f, "system:{}", name),
            PersonActor::ApiClient(id) => write!(f, "api:{}", id),
            PersonActor::Job(name) => write!(f, "job:{}", name),
            PersonActor::HrSystem(id) => write!(f, "hr:{}", id),
            PersonActor::IdentityProvider(name) => write!(f, "idp:{}", name),
            PersonActor::SkillsSystem(name) => write!(f, "skills:{}", name),
            PersonActor::NetworkSystem(name) => write!(f, "network:{}", name),
            PersonActor::EmploymentSystem(name) => write!(f, "employment:{}", name),
            PersonActor::ContactSystem(name) => write!(f, "contact:{}", name),
            PersonActor::DataSystem(name) => write!(f, "data:{}", name),
            PersonActor::PrivacySystem(name) => write!(f, "privacy:{}", name),
            PersonActor::Unknown => write!(f, "unknown"),
        }
    }
}

impl Default for PersonActor {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Complete message identity for tracking and correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageIdentity {
    /// Unique identifier for this message
    pub message_id: MessageId,
    /// Correlation identifier for grouping related messages
    pub correlation_id: CorrelationId,
    /// Causation identifier linking to the causing message
    pub causation_id: CausationId,
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    /// Actor that initiated the message
    pub actor: PersonActor,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl MessageIdentity {
    /// Create new message identity with default values
    pub fn new() -> Self {
        let message_id = MessageId::new();
        Self {
            correlation_id: CorrelationId::from(message_id.as_uuid()),
            causation_id: CausationId::from(message_id.clone()),
            message_id,
            timestamp: Utc::now(),
            actor: PersonActor::Unknown,
            metadata: HashMap::new(),
        }
    }
    
    /// Create message identity with specific actor
    pub fn with_actor(actor: PersonActor) -> Self {
        Self {
            actor,
            ..Self::new()
        }
    }
    
    /// Create message identity for user action
    pub fn for_user(user_id: &str) -> Self {
        Self::with_actor(PersonActor::user(user_id))
    }
    
    /// Create message identity for system action
    pub fn for_system(system_name: &str) -> Self {
        Self::with_actor(PersonActor::system(system_name))
    }
    
    /// Create message identity for API client
    pub fn for_api_client(client_id: &str) -> Self {
        Self::with_actor(PersonActor::api_client(client_id))
    }
    
    /// Create message identity for HR system action
    pub fn for_hr_system(system_id: &str) -> Self {
        Self::with_actor(PersonActor::hr_system(system_id))
    }
    
    /// Set correlation ID (for grouping related messages)
    pub fn with_correlation_id(mut self, correlation_id: impl Into<CorrelationId>) -> Self {
        self.correlation_id = correlation_id.into();
        self
    }
    
    /// Set causation ID (for linking to causing message)
    pub fn with_causation_id(mut self, causation_id: impl Into<CausationId>) -> Self {
        self.causation_id = causation_id.into();
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Set custom timestamp
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
    
    /// Create child message identity (inherits correlation, sets causation)
    pub fn create_child(&self) -> Self {
        Self {
            message_id: MessageId::new(),
            correlation_id: self.correlation_id.clone(),
            causation_id: CausationId::from(self.message_id.clone()),
            timestamp: Utc::now(),
            actor: self.actor.clone(),
            metadata: self.metadata.clone(),
        }
    }
    
    /// Create related message identity (same correlation, new causation chain)
    pub fn create_related(&self) -> Self {
        Self {
            message_id: MessageId::new(),
            correlation_id: self.correlation_id.clone(),
            causation_id: CausationId::new(),
            timestamp: Utc::now(),
            actor: self.actor.clone(),
            metadata: self.metadata.clone(),
        }
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Check if this message was caused by another specific message
    pub fn is_caused_by(&self, other_message_id: &MessageId) -> bool {
        self.causation_id.as_str() == other_message_id.to_string()
    }
    
    /// Check if this message is in the same correlation group
    pub fn is_correlated_with(&self, other: &MessageIdentity) -> bool {
        self.correlation_id == other.correlation_id
    }
}

impl Default for MessageIdentity {
    fn default() -> Self {
        Self::new()
    }
}

/// Message envelope containing identity and payload information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonMessageEnvelope<T> {
    /// Message identity for tracking
    pub identity: MessageIdentity,
    /// The actual message payload
    pub payload: T,
    /// Subject the message was sent to
    pub subject: String,
    /// Message headers for additional metadata
    pub headers: HashMap<String, String>,
}

impl<T> PersonMessageEnvelope<T> {
    /// Create new message envelope
    pub fn new(payload: T, subject: String, identity: MessageIdentity) -> Self {
        Self {
            identity,
            payload,
            subject,
            headers: HashMap::new(),
        }
    }
    
    /// Add header
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Get header value
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    
    /// Extract message ID from envelope
    pub fn message_id(&self) -> &MessageId {
        &self.identity.message_id
    }
    
    /// Extract correlation ID from envelope
    pub fn correlation_id(&self) -> &CorrelationId {
        &self.identity.correlation_id
    }
    
    /// Extract causation ID from envelope
    pub fn causation_id(&self) -> &CausationId {
        &self.identity.causation_id
    }
    
    /// Extract actor from envelope
    pub fn actor(&self) -> &PersonActor {
        &self.identity.actor
    }
    
    /// Check if message is from a specific actor type
    pub fn is_from_user(&self) -> bool {
        matches!(self.identity.actor, PersonActor::User(_))
    }
    
    pub fn is_from_system(&self) -> bool {
        matches!(self.identity.actor, PersonActor::System(_))
    }
    
    pub fn is_from_hr_system(&self) -> bool {
        matches!(self.identity.actor, PersonActor::HrSystem(_))
    }
    
    /// Create reply envelope with same correlation
    pub fn create_reply<R>(&self, reply_payload: R, reply_subject: String) -> PersonMessageEnvelope<R> {
        let reply_identity = self.identity.create_child();
        PersonMessageEnvelope::new(reply_payload, reply_subject, reply_identity)
    }
}

/// Tracing context for message processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonTracingContext {
    /// Trace identifier for distributed tracing
    pub trace_id: String,
    /// Span identifier within the trace
    pub span_id: String,
    /// Parent span identifier
    pub parent_span_id: Option<String>,
    /// Trace sampling decision
    pub sampled: bool,
    /// Additional tracing flags
    pub flags: u8,
}

impl PersonTracingContext {
    /// Create new tracing context
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            sampled: true,
            flags: 0,
        }
    }
    
    /// Create child span context
    pub fn create_child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            sampled: self.sampled,
            flags: self.flags,
        }
    }
    
    /// Convert to tracing headers
    pub fn to_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("x-trace-id".to_string(), self.trace_id.clone());
        headers.insert("x-span-id".to_string(), self.span_id.clone());
        if let Some(parent_span_id) = &self.parent_span_id {
            headers.insert("x-parent-span-id".to_string(), parent_span_id.clone());
        }
        headers.insert("x-sampled".to_string(), self.sampled.to_string());
        headers.insert("x-flags".to_string(), self.flags.to_string());
        headers
    }
    
    /// Create from tracing headers
    pub fn from_headers(headers: &HashMap<String, String>) -> Option<Self> {
        let trace_id = headers.get("x-trace-id")?.clone();
        let span_id = headers.get("x-span-id")?.clone();
        let parent_span_id = headers.get("x-parent-span-id").cloned();
        let sampled = headers.get("x-sampled")
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);
        let flags = headers.get("x-flags")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        Some(Self {
            trace_id,
            span_id,
            parent_span_id,
            sampled,
            flags,
        })
    }
}

impl Default for PersonTracingContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_id_creation() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();
        assert_ne!(id1, id2);
        
        let uuid = Uuid::new_v4();
        let id3 = MessageId::from_uuid(uuid);
        assert_eq!(id3.as_uuid(), uuid);
    }
    
    #[test]
    fn test_correlation_id() {
        let corr_id = CorrelationId::new();
        assert!(!corr_id.as_str().is_empty());
        
        let custom_id = CorrelationId::from_string("custom-correlation".to_string());
        assert_eq!(custom_id.as_str(), "custom-correlation");
    }
    
    #[test]
    fn test_causation_id_from_message_id() {
        let message_id = MessageId::new();
        let causation_id = CausationId::from_message_id(&message_id);
        assert_eq!(causation_id.as_str(), message_id.to_string());
    }
    
    #[test]
    fn test_person_actor_display() {
        assert_eq!(PersonActor::user("123").to_string(), "user:123");
        assert_eq!(PersonActor::system("hr-system").to_string(), "system:hr-system");
        assert_eq!(PersonActor::hr_system("hr-1").to_string(), "hr:hr-1");
        assert_eq!(PersonActor::Unknown.to_string(), "unknown");
    }
    
    #[test]
    fn test_message_identity_creation() {
        let identity = MessageIdentity::for_user("user123");
        assert!(matches!(identity.actor, PersonActor::User(ref id) if id == "user123"));
        
        let system_identity = MessageIdentity::for_system("person-service");
        assert!(matches!(system_identity.actor, PersonActor::System(ref name) if name == "person-service"));
    }
    
    #[test]
    fn test_message_identity_correlation() {
        let parent = MessageIdentity::new();
        let child = parent.create_child();
        
        assert_eq!(parent.correlation_id, child.correlation_id);
        assert_eq!(child.causation_id.as_str(), parent.message_id.to_string());
        assert!(child.is_caused_by(&parent.message_id));
        assert!(child.is_correlated_with(&parent));
    }
    
    #[test]
    fn test_message_envelope() {
        let identity = MessageIdentity::for_user("user123");
        let envelope = PersonMessageEnvelope::new(
            "test payload".to_string(),
            "events.person.person.created.person123".to_string(),
            identity
        ).with_header("content-type", "application/json");
        
        assert_eq!(envelope.payload, "test payload");
        assert_eq!(envelope.subject, "events.person.person.created.person123");
        assert_eq!(envelope.get_header("content-type"), Some(&"application/json".to_string()));
        assert!(envelope.is_from_user());
        assert!(!envelope.is_from_system());
    }
    
    #[test]
    fn test_tracing_context() {
        let context = PersonTracingContext::new();
        let child_context = context.create_child_span();
        
        assert_eq!(context.trace_id, child_context.trace_id);
        assert_ne!(context.span_id, child_context.span_id);
        assert_eq!(child_context.parent_span_id, Some(context.span_id.clone()));
        
        let headers = context.to_headers();
        assert!(headers.contains_key("x-trace-id"));
        assert!(headers.contains_key("x-span-id"));
        
        let restored_context = PersonTracingContext::from_headers(&headers);
        assert!(restored_context.is_some());
        let restored = restored_context.unwrap();
        assert_eq!(restored.trace_id, context.trace_id);
        assert_eq!(restored.span_id, context.span_id);
    }
    
    #[test]
    fn test_employment_workflow_tracing() {
        // Test employment workflow message tracing
        let hr_identity = MessageIdentity::for_hr_system("workday-hr")
            .with_correlation_id("employment-workflow-123");
        
        // Employment added event
        let employment_added_identity = hr_identity.create_child();
        assert!(employment_added_identity.is_correlated_with(&hr_identity));
        assert!(employment_added_identity.is_caused_by(&hr_identity.message_id));
        
        // Role change event (related to same workflow)
        let role_change_identity = hr_identity.create_related();
        assert!(role_change_identity.is_correlated_with(&hr_identity));
        assert!(!role_change_identity.is_caused_by(&hr_identity.message_id));
    }
    
    #[test]
    fn test_skills_endorsement_workflow() {
        // Test skills endorsement message chain
        let user_identity = MessageIdentity::for_user("user456")
            .with_correlation_id("skills-endorsement-789");
        
        let skills_system_identity = MessageIdentity::for_system("linkedin-skills")
            .with_correlation_id(user_identity.correlation_id.clone())
            .with_causation_id(CausationId::from(user_identity.message_id.clone()));
        
        assert!(skills_system_identity.is_correlated_with(&user_identity));
        assert!(skills_system_identity.is_caused_by(&user_identity.message_id));
    }
    
    #[test]
    fn test_privacy_workflow_tracing() {
        // Test GDPR data request workflow
        let user_identity = MessageIdentity::for_user("user789")
            .with_correlation_id("gdpr-data-request-456")
            .with_metadata("request_type", "data_export")
            .with_metadata("jurisdiction", "EU");
        
        let privacy_system_identity = user_identity.create_child();
        let data_system_identity = privacy_system_identity.create_child();
        
        // All should be correlated
        assert!(privacy_system_identity.is_correlated_with(&user_identity));
        assert!(data_system_identity.is_correlated_with(&user_identity));
        assert!(data_system_identity.is_correlated_with(&privacy_system_identity));
        
        // Check causation chain
        assert!(privacy_system_identity.is_caused_by(&user_identity.message_id));
        assert!(data_system_identity.is_caused_by(&privacy_system_identity.message_id));
        
        // Check metadata inheritance
        assert_eq!(user_identity.get_metadata("request_type"), Some(&"data_export".to_string()));
        assert_eq!(user_identity.get_metadata("jurisdiction"), Some(&"EU".to_string()));
    }
}