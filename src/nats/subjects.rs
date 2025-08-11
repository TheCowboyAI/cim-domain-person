//! Subject algebra for Person domain NATS messaging
//!
//! Provides structured subject patterns for events, commands, and queries
//! following CIM domain conventions and enabling efficient wildcard subscriptions.

use std::fmt;
use serde::{Deserialize, Serialize};

/// Root-level subject categories for Person domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonSubjectRoot {
    Events,
    Commands,
    Queries,
}

/// Person domain aggregates for subject routing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonAggregate {
    Person,
    Identity,
    Employment,
    Skills,
    Network,
    Preferences,
    Demographics,
    Contact,
}

/// Event types for person aggregate
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonEventType {
    // Core person lifecycle
    Created,
    Updated,
    Archived,
    Reactivated,
    Merged,
    Split,
    
    // Identity events
    NameUpdated,
    BirthDateSet,
    DeathRecorded,
    IdentifierAdded,
    IdentifierRemoved,
    
    // Employment events
    EmploymentAdded,
    EmploymentUpdated,
    EmploymentEnded,
    RoleChanged,
    OrganizationChanged,
    
    // Skills events
    SkillAdded,
    SkillUpdated,
    SkillRemoved,
    SkillEndorsed,
    CertificationAdded,
    CertificationExpired,
    
    // Network events
    ConnectionRequested,
    ConnectionAccepted,
    ConnectionRejected,
    ConnectionRemoved,
    NetworkUpdated,
    
    // Contact events
    ContactAdded,
    ContactUpdated,
    ContactRemoved,
    ContactVerified,
    
    // Component events
    ComponentRegistered,
    ComponentUnregistered,
    ComponentDataUpdated,
    
    // Privacy events
    PrivacySettingsUpdated,
    ConsentGiven,
    ConsentRevoked,
    DataExportRequested,
    DataDeletionRequested,
}

/// Command types for person aggregate
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonCommandType {
    // Core person commands
    CreatePerson,
    UpdatePerson,
    ArchivePerson,
    ReactivatePerson,
    MergePerson,
    
    // Identity commands
    UpdateName,
    SetBirthDate,
    RecordDeath,
    AddIdentifier,
    RemoveIdentifier,
    
    // Employment commands
    AddEmployment,
    UpdateEmployment,
    EndEmployment,
    ChangeRole,
    
    // Skills commands
    AddSkill,
    UpdateSkill,
    RemoveSkill,
    EndorseSkill,
    AddCertification,
    
    // Network commands
    RequestConnection,
    AcceptConnection,
    RejectConnection,
    RemoveConnection,
    
    // Contact commands
    AddContact,
    UpdateContact,
    RemoveContact,
    VerifyContact,
    
    // Component commands
    RegisterComponent,
    UnregisterComponent,
    UpdateComponentData,
    
    // Privacy commands
    UpdatePrivacySettings,
    GiveConsent,
    RevokeConsent,
    RequestDataExport,
    RequestDataDeletion,
}

/// Query types for person aggregate
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonQueryType {
    // Person queries
    GetPerson,
    ListPersons,
    SearchPersons,
    GetPersonHistory,
    
    // Identity queries
    FindByIdentifier,
    GetIdentityHistory,
    
    // Employment queries
    GetEmploymentHistory,
    FindByEmployer,
    FindByRole,
    
    // Skills queries
    GetSkills,
    SearchBySkill,
    GetSkillEndorsements,
    GetCertifications,
    
    // Network queries
    GetConnections,
    GetConnectionRequests,
    FindMutualConnections,
    GetNetworkAnalysis,
    
    // Contact queries
    GetContacts,
    FindByContact,
    VerifyContactReachability,
    
    // Privacy queries
    GetPrivacySettings,
    GetConsentHistory,
    GetDataExportStatus,
}

/// Scoping mechanisms for subject hierarchies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonScope {
    /// Global scope - no specific scoping
    Global,
    /// User-scoped subjects for user-specific operations
    User(String),
    /// Organization-scoped subjects for company-specific operations
    Organization(String),
    /// Team-scoped subjects for team-specific operations
    Team(String),
    /// Region-scoped subjects for geographic operations
    Region(String),
    /// Department-scoped subjects for department-specific operations
    Department(String),
}

/// Main subject structure for Person domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonSubject {
    /// Namespace for multi-tenancy support
    pub namespace: Option<String>,
    /// Subject root category
    pub root: PersonSubjectRoot,
    /// Domain identifier
    pub domain: String,
    /// Aggregate type
    pub aggregate: PersonAggregate,
    /// Scope for hierarchical organization
    pub scope: PersonScope,
    /// Operation (event, command, or query type)
    pub operation: Option<String>,
    /// Entity ID for specific targeting
    pub entity_id: Option<String>,
}

impl PersonSubject {
    /// Create new person subject with default values
    pub fn new() -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Events,
            domain: "person".to_string(),
            aggregate: PersonAggregate::Person,
            scope: PersonScope::Global,
            operation: None,
            entity_id: None,
        }
    }
    
    /// Create event subject
    pub fn event(aggregate: PersonAggregate, event_type: PersonEventType, entity_id: &str) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Events,
            domain: "person".to_string(),
            aggregate,
            scope: PersonScope::Global,
            operation: Some(event_type.to_string()),
            entity_id: Some(entity_id.to_string()),
        }
    }
    
    /// Create command subject
    pub fn command(aggregate: PersonAggregate, command_type: PersonCommandType, entity_id: &str) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Commands,
            domain: "person".to_string(),
            aggregate,
            scope: PersonScope::Global,
            operation: Some(command_type.to_string()),
            entity_id: Some(entity_id.to_string()),
        }
    }
    
    /// Create query subject
    pub fn query(aggregate: PersonAggregate, query_type: PersonQueryType) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Queries,
            domain: "person".to_string(),
            aggregate,
            scope: PersonScope::Global,
            operation: Some(query_type.to_string()),
            entity_id: None,
        }
    }
    
    /// Set namespace for multi-tenancy
    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);
        self
    }
    
    /// Set scope
    pub fn with_scope(mut self, scope: PersonScope) -> Self {
        self.scope = scope;
        self
    }
    
    /// Set operation
    pub fn with_operation(mut self, operation: String) -> Self {
        self.operation = Some(operation);
        self
    }
    
    /// Set entity ID
    pub fn with_entity_id(mut self, entity_id: String) -> Self {
        self.entity_id = Some(entity_id);
        self
    }
    
    /// Create wildcard subject for subscribing to all events of an aggregate
    pub fn events_wildcard(aggregate: PersonAggregate) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Events,
            domain: "person".to_string(),
            aggregate,
            scope: PersonScope::Global,
            operation: Some("*".to_string()),
            entity_id: Some("*".to_string()),
        }
    }
    
    /// Create wildcard subject for subscribing to all commands of an aggregate
    pub fn commands_wildcard(aggregate: PersonAggregate) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Commands,
            domain: "person".to_string(),
            aggregate,
            scope: PersonScope::Global,
            operation: Some("*".to_string()),
            entity_id: Some("*".to_string()),
        }
    }
    
    /// Create user-scoped event subject
    pub fn user_event(
        user_id: &str, 
        aggregate: PersonAggregate, 
        event_type: PersonEventType, 
        entity_id: &str
    ) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Events,
            domain: "person".to_string(),
            aggregate,
            scope: PersonScope::User(user_id.to_string()),
            operation: Some(event_type.to_string()),
            entity_id: Some(entity_id.to_string()),
        }
    }
    
    /// Create organization-scoped event subject
    pub fn org_event(
        org_id: &str, 
        aggregate: PersonAggregate, 
        event_type: PersonEventType, 
        entity_id: &str
    ) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Events,
            domain: "person".to_string(),
            aggregate,
            scope: PersonScope::Organization(org_id.to_string()),
            operation: Some(event_type.to_string()),
            entity_id: Some(entity_id.to_string()),
        }
    }
    
    /// Create team-scoped event subject
    pub fn team_event(
        team_id: &str, 
        aggregate: PersonAggregate, 
        event_type: PersonEventType, 
        entity_id: &str
    ) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Events,
            domain: "person".to_string(),
            aggregate,
            scope: PersonScope::Team(team_id.to_string()),
            operation: Some(event_type.to_string()),
            entity_id: Some(entity_id.to_string()),
        }
    }
    
    /// Create cross-domain integration subject for person-document relationships
    pub fn person_document_event(person_id: &str, document_cid: &str, event_type: &str) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Events,
            domain: "person".to_string(),
            aggregate: PersonAggregate::Person,
            scope: PersonScope::Global,
            operation: Some(format!("document.{}.{}", document_cid, event_type)),
            entity_id: Some(person_id.to_string()),
        }
    }
    
    /// Create cross-domain integration subject for person-location relationships
    pub fn person_location_event(person_id: &str, location_id: &str, event_type: &str) -> Self {
        Self {
            namespace: None,
            root: PersonSubjectRoot::Events,
            domain: "person".to_string(),
            aggregate: PersonAggregate::Person,
            scope: PersonScope::Global,
            operation: Some(format!("location.{}.{}", location_id, event_type)),
            entity_id: Some(person_id.to_string()),
        }
    }
}

impl Default for PersonSubject {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PersonSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        
        // Add namespace if present
        if let Some(namespace) = &self.namespace {
            parts.push(namespace.clone());
        }
        
        // Add root
        parts.push(self.root.to_string());
        
        // Add domain
        parts.push(self.domain.clone());
        
        // Add aggregate
        parts.push(self.aggregate.to_string());
        
        // Add scope if not global
        if self.scope != PersonScope::Global {
            parts.push(self.scope.to_string());
        }
        
        // Add operation if present
        if let Some(operation) = &self.operation {
            parts.push(operation.clone());
        }
        
        // Add entity ID if present
        if let Some(entity_id) = &self.entity_id {
            parts.push(entity_id.clone());
        }
        
        write!(f, "{}", parts.join("."))
    }
}

impl fmt::Display for PersonSubjectRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonSubjectRoot::Events => write!(f, "events"),
            PersonSubjectRoot::Commands => write!(f, "commands"),
            PersonSubjectRoot::Queries => write!(f, "queries"),
        }
    }
}

impl fmt::Display for PersonAggregate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonAggregate::Person => write!(f, "person"),
            PersonAggregate::Identity => write!(f, "identity"),
            PersonAggregate::Employment => write!(f, "employment"),
            PersonAggregate::Skills => write!(f, "skills"),
            PersonAggregate::Network => write!(f, "network"),
            PersonAggregate::Preferences => write!(f, "preferences"),
            PersonAggregate::Demographics => write!(f, "demographics"),
            PersonAggregate::Contact => write!(f, "contact"),
        }
    }
}

impl fmt::Display for PersonEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonEventType::Created => write!(f, "created"),
            PersonEventType::Updated => write!(f, "updated"),
            PersonEventType::Archived => write!(f, "archived"),
            PersonEventType::Reactivated => write!(f, "reactivated"),
            PersonEventType::Merged => write!(f, "merged"),
            PersonEventType::Split => write!(f, "split"),
            PersonEventType::NameUpdated => write!(f, "name_updated"),
            PersonEventType::BirthDateSet => write!(f, "birth_date_set"),
            PersonEventType::DeathRecorded => write!(f, "death_recorded"),
            PersonEventType::IdentifierAdded => write!(f, "identifier_added"),
            PersonEventType::IdentifierRemoved => write!(f, "identifier_removed"),
            PersonEventType::EmploymentAdded => write!(f, "employment_added"),
            PersonEventType::EmploymentUpdated => write!(f, "employment_updated"),
            PersonEventType::EmploymentEnded => write!(f, "employment_ended"),
            PersonEventType::RoleChanged => write!(f, "role_changed"),
            PersonEventType::OrganizationChanged => write!(f, "organization_changed"),
            PersonEventType::SkillAdded => write!(f, "skill_added"),
            PersonEventType::SkillUpdated => write!(f, "skill_updated"),
            PersonEventType::SkillRemoved => write!(f, "skill_removed"),
            PersonEventType::SkillEndorsed => write!(f, "skill_endorsed"),
            PersonEventType::CertificationAdded => write!(f, "certification_added"),
            PersonEventType::CertificationExpired => write!(f, "certification_expired"),
            PersonEventType::ConnectionRequested => write!(f, "connection_requested"),
            PersonEventType::ConnectionAccepted => write!(f, "connection_accepted"),
            PersonEventType::ConnectionRejected => write!(f, "connection_rejected"),
            PersonEventType::ConnectionRemoved => write!(f, "connection_removed"),
            PersonEventType::NetworkUpdated => write!(f, "network_updated"),
            PersonEventType::ContactAdded => write!(f, "contact_added"),
            PersonEventType::ContactUpdated => write!(f, "contact_updated"),
            PersonEventType::ContactRemoved => write!(f, "contact_removed"),
            PersonEventType::ContactVerified => write!(f, "contact_verified"),
            PersonEventType::ComponentRegistered => write!(f, "component_registered"),
            PersonEventType::ComponentUnregistered => write!(f, "component_unregistered"),
            PersonEventType::ComponentDataUpdated => write!(f, "component_data_updated"),
            PersonEventType::PrivacySettingsUpdated => write!(f, "privacy_settings_updated"),
            PersonEventType::ConsentGiven => write!(f, "consent_given"),
            PersonEventType::ConsentRevoked => write!(f, "consent_revoked"),
            PersonEventType::DataExportRequested => write!(f, "data_export_requested"),
            PersonEventType::DataDeletionRequested => write!(f, "data_deletion_requested"),
        }
    }
}

impl fmt::Display for PersonCommandType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonCommandType::CreatePerson => write!(f, "create_person"),
            PersonCommandType::UpdatePerson => write!(f, "update_person"),
            PersonCommandType::ArchivePerson => write!(f, "archive_person"),
            PersonCommandType::ReactivatePerson => write!(f, "reactivate_person"),
            PersonCommandType::MergePerson => write!(f, "merge_person"),
            PersonCommandType::UpdateName => write!(f, "update_name"),
            PersonCommandType::SetBirthDate => write!(f, "set_birth_date"),
            PersonCommandType::RecordDeath => write!(f, "record_death"),
            PersonCommandType::AddIdentifier => write!(f, "add_identifier"),
            PersonCommandType::RemoveIdentifier => write!(f, "remove_identifier"),
            PersonCommandType::AddEmployment => write!(f, "add_employment"),
            PersonCommandType::UpdateEmployment => write!(f, "update_employment"),
            PersonCommandType::EndEmployment => write!(f, "end_employment"),
            PersonCommandType::ChangeRole => write!(f, "change_role"),
            PersonCommandType::AddSkill => write!(f, "add_skill"),
            PersonCommandType::UpdateSkill => write!(f, "update_skill"),
            PersonCommandType::RemoveSkill => write!(f, "remove_skill"),
            PersonCommandType::EndorseSkill => write!(f, "endorse_skill"),
            PersonCommandType::AddCertification => write!(f, "add_certification"),
            PersonCommandType::RequestConnection => write!(f, "request_connection"),
            PersonCommandType::AcceptConnection => write!(f, "accept_connection"),
            PersonCommandType::RejectConnection => write!(f, "reject_connection"),
            PersonCommandType::RemoveConnection => write!(f, "remove_connection"),
            PersonCommandType::AddContact => write!(f, "add_contact"),
            PersonCommandType::UpdateContact => write!(f, "update_contact"),
            PersonCommandType::RemoveContact => write!(f, "remove_contact"),
            PersonCommandType::VerifyContact => write!(f, "verify_contact"),
            PersonCommandType::RegisterComponent => write!(f, "register_component"),
            PersonCommandType::UnregisterComponent => write!(f, "unregister_component"),
            PersonCommandType::UpdateComponentData => write!(f, "update_component_data"),
            PersonCommandType::UpdatePrivacySettings => write!(f, "update_privacy_settings"),
            PersonCommandType::GiveConsent => write!(f, "give_consent"),
            PersonCommandType::RevokeConsent => write!(f, "revoke_consent"),
            PersonCommandType::RequestDataExport => write!(f, "request_data_export"),
            PersonCommandType::RequestDataDeletion => write!(f, "request_data_deletion"),
        }
    }
}

impl fmt::Display for PersonQueryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonQueryType::GetPerson => write!(f, "get_person"),
            PersonQueryType::ListPersons => write!(f, "list_persons"),
            PersonQueryType::SearchPersons => write!(f, "search_persons"),
            PersonQueryType::GetPersonHistory => write!(f, "get_person_history"),
            PersonQueryType::FindByIdentifier => write!(f, "find_by_identifier"),
            PersonQueryType::GetIdentityHistory => write!(f, "get_identity_history"),
            PersonQueryType::GetEmploymentHistory => write!(f, "get_employment_history"),
            PersonQueryType::FindByEmployer => write!(f, "find_by_employer"),
            PersonQueryType::FindByRole => write!(f, "find_by_role"),
            PersonQueryType::GetSkills => write!(f, "get_skills"),
            PersonQueryType::SearchBySkill => write!(f, "search_by_skill"),
            PersonQueryType::GetSkillEndorsements => write!(f, "get_skill_endorsements"),
            PersonQueryType::GetCertifications => write!(f, "get_certifications"),
            PersonQueryType::GetConnections => write!(f, "get_connections"),
            PersonQueryType::GetConnectionRequests => write!(f, "get_connection_requests"),
            PersonQueryType::FindMutualConnections => write!(f, "find_mutual_connections"),
            PersonQueryType::GetNetworkAnalysis => write!(f, "get_network_analysis"),
            PersonQueryType::GetContacts => write!(f, "get_contacts"),
            PersonQueryType::FindByContact => write!(f, "find_by_contact"),
            PersonQueryType::VerifyContactReachability => write!(f, "verify_contact_reachability"),
            PersonQueryType::GetPrivacySettings => write!(f, "get_privacy_settings"),
            PersonQueryType::GetConsentHistory => write!(f, "get_consent_history"),
            PersonQueryType::GetDataExportStatus => write!(f, "get_data_export_status"),
        }
    }
}

impl fmt::Display for PersonScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonScope::Global => write!(f, "global"),
            PersonScope::User(id) => write!(f, "user.{}", id),
            PersonScope::Organization(id) => write!(f, "org.{}", id),
            PersonScope::Team(id) => write!(f, "team.{}", id),
            PersonScope::Region(id) => write!(f, "region.{}", id),
            PersonScope::Department(id) => write!(f, "dept.{}", id),
        }
    }
}

/// Builder for constructing PersonSubject instances
#[derive(Debug, Default)]
pub struct PersonSubjectBuilder {
    subject: PersonSubject,
}

impl PersonSubjectBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn namespace(mut self, namespace: String) -> Self {
        self.subject.namespace = Some(namespace);
        self
    }
    
    pub fn events(mut self) -> Self {
        self.subject.root = PersonSubjectRoot::Events;
        self
    }
    
    pub fn commands(mut self) -> Self {
        self.subject.root = PersonSubjectRoot::Commands;
        self
    }
    
    pub fn queries(mut self) -> Self {
        self.subject.root = PersonSubjectRoot::Queries;
        self
    }
    
    pub fn aggregate(mut self, aggregate: PersonAggregate) -> Self {
        self.subject.aggregate = aggregate;
        self
    }
    
    pub fn scope(mut self, scope: PersonScope) -> Self {
        self.subject.scope = scope;
        self
    }
    
    pub fn operation(mut self, operation: String) -> Self {
        self.subject.operation = Some(operation);
        self
    }
    
    pub fn entity_id(mut self, entity_id: String) -> Self {
        self.subject.entity_id = Some(entity_id);
        self
    }
    
    pub fn build(self) -> PersonSubject {
        self.subject
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_subject_construction() {
        let subject = PersonSubject::event(
            PersonAggregate::Person,
            PersonEventType::Created,
            "person123"
        );
        
        assert_eq!(subject.to_string(), "events.person.person.created.person123");
    }
    
    #[test]
    fn test_namespaced_subject() {
        let subject = PersonSubject::event(
            PersonAggregate::Person,
            PersonEventType::Created,
            "person123"
        ).with_namespace("tenant1".to_string());
        
        assert_eq!(subject.to_string(), "tenant1.events.person.person.created.person123");
    }
    
    #[test]
    fn test_user_scoped_subject() {
        let subject = PersonSubject::user_event(
            "user456",
            PersonAggregate::Skills,
            PersonEventType::SkillAdded,
            "person123"
        );
        
        assert_eq!(subject.to_string(), "events.person.skills.user.user456.skill_added.person123");
    }
    
    #[test]
    fn test_organization_scoped_subject() {
        let subject = PersonSubject::org_event(
            "org789",
            PersonAggregate::Employment,
            PersonEventType::EmploymentAdded,
            "person123"
        );
        
        assert_eq!(subject.to_string(), "events.person.employment.org.org789.employment_added.person123");
    }
    
    #[test]
    fn test_wildcard_subjects() {
        let events_wildcard = PersonSubject::events_wildcard(PersonAggregate::Person);
        assert_eq!(events_wildcard.to_string(), "events.person.person.*.">);
        
        let commands_wildcard = PersonSubject::commands_wildcard(PersonAggregate::Skills);
        assert_eq!(commands_wildcard.to_string(), "commands.person.skills.*.*");
    }
    
    #[test]
    fn test_cross_domain_subjects() {
        let person_doc_subject = PersonSubject::person_document_event(
            "person123",
            "Qm...",
            "viewed"
        );
        
        assert_eq!(person_doc_subject.to_string(), "events.person.person.document.Qm....viewed.person123");
    }
    
    #[test]
    fn test_command_subject() {
        let subject = PersonSubject::command(
            PersonAggregate::Person,
            PersonCommandType::UpdatePerson,
            "person123"
        );
        
        assert_eq!(subject.to_string(), "commands.person.person.update_person.person123");
    }
    
    #[test]
    fn test_query_subject() {
        let subject = PersonSubject::query(
            PersonAggregate::Person,
            PersonQueryType::GetPerson
        );
        
        assert_eq!(subject.to_string(), "queries.person.person.get_person");
    }
    
    #[test]
    fn test_subject_builder() {
        let subject = PersonSubjectBuilder::new()
            .namespace("tenant1".to_string())
            .events()
            .aggregate(PersonAggregate::Skills)
            .scope(PersonScope::User("user123".to_string()))
            .operation("skill_added".to_string())
            .entity_id("person456".to_string())
            .build();
        
        assert_eq!(subject.to_string(), "tenant1.events.person.skills.user.user123.skill_added.person456");
    }
    
    #[test]
    fn test_employment_workflow_subjects() {
        // Employment lifecycle events
        let hire_event = PersonSubject::event(
            PersonAggregate::Employment,
            PersonEventType::EmploymentAdded,
            "person123"
        );
        assert_eq!(hire_event.to_string(), "events.person.employment.employment_added.person123");
        
        let role_change_event = PersonSubject::event(
            PersonAggregate::Employment,
            PersonEventType::RoleChanged,
            "person123"
        );
        assert_eq!(role_change_event.to_string(), "events.person.employment.role_changed.person123");
        
        let termination_event = PersonSubject::event(
            PersonAggregate::Employment,
            PersonEventType::EmploymentEnded,
            "person123"
        );
        assert_eq!(termination_event.to_string(), "events.person.employment.employment_ended.person123");
    }
    
    #[test]
    fn test_skills_certification_subjects() {
        // Skills and certification events
        let skill_add = PersonSubject::event(
            PersonAggregate::Skills,
            PersonEventType::SkillAdded,
            "person123"
        );
        assert_eq!(skill_add.to_string(), "events.person.skills.skill_added.person123");
        
        let certification_add = PersonSubject::event(
            PersonAggregate::Skills,
            PersonEventType::CertificationAdded,
            "person123"
        );
        assert_eq!(certification_add.to_string(), "events.person.skills.certification_added.person123");
        
        let skill_endorsement = PersonSubject::event(
            PersonAggregate::Skills,
            PersonEventType::SkillEndorsed,
            "person123"
        );
        assert_eq!(skill_endorsement.to_string(), "events.person.skills.skill_endorsed.person123");
    }
    
    #[test]
    fn test_network_connection_subjects() {
        // Network and connection events
        let connection_request = PersonSubject::event(
            PersonAggregate::Network,
            PersonEventType::ConnectionRequested,
            "person123"
        );
        assert_eq!(connection_request.to_string(), "events.person.network.connection_requested.person123");
        
        let connection_accepted = PersonSubject::event(
            PersonAggregate::Network,
            PersonEventType::ConnectionAccepted,
            "person123"
        );
        assert_eq!(connection_accepted.to_string(), "events.person.network.connection_accepted.person123");
    }
}