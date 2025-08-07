# Integration Guide

## Overview

The Person domain integrates with other CIM domains and external systems through well-defined interfaces and event-driven communication patterns.

## Cross-Domain Integration

### Agent Domain Integration

The Person domain delegates tasks to the Agent domain for autonomous operations.

```rust
use cim_domain_person::cross_domain::agent_integration::AgentIntegration;

// Delegate task to agent
let integration = AgentIntegration::new(agent_client);

let task = AgentTask {
    task_type: TaskType::DataEnrichment,
    target_person: person_id,
    parameters: json!({
        "sources": ["linkedin", "github"],
        "fields": ["skills", "experience"]
    }),
};

let result = integration.delegate_task(task).await?;
```

**Integration Points:**
- Data enrichment tasks
- Automated outreach
- Research and analysis
- Workflow automation

### Location Domain Integration

Validates and enriches address components.

```rust
use cim_domain_person::cross_domain::location_integration::LocationIntegration;

let integration = LocationIntegration::new(location_client);

// Validate address
let validated = integration
    .validate_address(&address_component.address)
    .await?;

// Geocode address
let coordinates = integration
    .geocode_address(&address_component.address)
    .await?;

// Find nearby persons
let nearby = integration
    .find_persons_near(coordinates, radius_km)
    .await?;
```

**Integration Points:**
- Address validation
- Geocoding/reverse geocoding
- Proximity searches
- Region-based queries

### Identity Domain Integration

Handles authentication and authorization.

```rust
use cim_domain_person::cross_domain::identity_integration::IdentityIntegration;

let integration = IdentityIntegration::new(identity_client);

// Create identity for person
let identity = integration
    .create_identity(person_id, credentials)
    .await?;

// Verify permissions
let can_edit = integration
    .check_permission(actor_id, person_id, Permission::Edit)
    .await?;

// Link external identity
integration
    .link_external_identity(person_id, provider, external_id)
    .await?;
```

**Integration Points:**
- Identity creation
- Authentication flows
- Permission checks
- SSO integration

### Git Domain Integration

Version control for person data and history.

```rust
use cim_domain_person::cross_domain::git_integration::GitIntegration;

let integration = GitIntegration::new(git_client);

// Track person changes
let commit = integration
    .commit_person_changes(person_id, changes, message)
    .await?;

// View history
let history = integration
    .get_person_history(person_id, limit)
    .await?;

// Revert changes
integration
    .revert_to_commit(person_id, commit_hash)
    .await?;
```

**Integration Points:**
- Change tracking
- Audit history
- Rollback capability
- Branch management

## Event-Based Integration

### Publishing Events

All person domain events are published to NATS JetStream for consumption by other domains.

```rust
use cim_domain_person::infrastructure::nats_integration::NatsPublisher;

let publisher = NatsPublisher::new(nats_client).await?;

// Publish person created event
publisher
    .publish("person.created", &event)
    .await?;

// Publish with specific subject hierarchy
publisher
    .publish_to_subject(
        "cim.person.component.email.added",
        &component_event
    )
    .await?;
```

### Subscribing to Events

Listen to events from other domains.

```rust
use cim_domain_person::infrastructure::subscriptions::EventSubscriber;

let subscriber = EventSubscriber::new(nats_client).await?;

// Subscribe to organization events
subscriber
    .subscribe("organization.member.added", |event| async {
        // Create relationship component
        handle_organization_member_added(event).await
    })
    .await?;

// Subscribe with queue group for load balancing
subscriber
    .queue_subscribe("identity.created", "person-workers", |event| async {
        // Link identity to person
        handle_identity_created(event).await
    })
    .await?;
```

### Event Routing

Configure event routing rules for cross-domain communication.

```yaml
# event-routing.yaml
routes:
  - source: person.created
    destinations:
      - identity.provision
      - notification.welcome
      - analytics.track
  
  - source: person.skill.added
    destinations:
      - recommendation.update
      - search.index
  
  - source: person.relationship.created
    destinations:
      - graph.update
      - notification.connection
```

## API Integration

### REST API Adapter

```rust
use cim_domain_person::handlers::cqrs_adapter::RestAdapter;

let adapter = RestAdapter::new(command_processor, query_processor);

// Handle HTTP request
let response = adapter
    .handle_request(HttpRequest {
        method: Method::POST,
        path: "/persons",
        body: json!({
            "firstName": "Alice",
            "lastName": "Smith",
            "email": "alice@example.com"
        }),
    })
    .await?;
```

### GraphQL Integration

```rust
use cim_domain_person::handlers::graphql_adapter::GraphQLAdapter;

let adapter = GraphQLAdapter::new(command_processor, query_processor);

// Execute GraphQL query
let result = adapter
    .execute(
        r#"
        query GetPerson($id: ID!) {
            person(id: $id) {
                id
                name {
                    firstName
                    lastName
                }
                emails {
                    address
                    isPrimary
                }
                skills {
                    name
                    proficiency
                }
            }
        }
        "#,
        variables,
    )
    .await?;
```

### gRPC Integration

```proto
// person.proto
service PersonService {
    rpc CreatePerson(CreatePersonRequest) returns (PersonResponse);
    rpc GetPerson(GetPersonRequest) returns (PersonResponse);
    rpc UpdatePerson(UpdatePersonRequest) returns (PersonResponse);
    rpc StreamPersonEvents(StreamRequest) returns (stream PersonEvent);
}
```

```rust
use cim_domain_person::grpc::PersonServiceImpl;

let service = PersonServiceImpl::new(command_processor, query_processor);

// Start gRPC server
Server::builder()
    .add_service(PersonServiceServer::new(service))
    .serve(addr)
    .await?;
```

## External System Integration

### Database Integration

```rust
use cim_domain_person::infrastructure::persistence::DatabaseAdapter;

let adapter = DatabaseAdapter::new(connection_pool);

// Store projection
adapter
    .store_projection("person_summary", &summary_projection)
    .await?;

// Query with SQL
let results = adapter
    .query_raw(
        "SELECT * FROM person_projections WHERE skills @> ?",
        &[&["Rust", "Go"]]
    )
    .await?;
```

### Search Engine Integration

```rust
use cim_domain_person::infrastructure::search::ElasticsearchAdapter;

let search = ElasticsearchAdapter::new(es_client);

// Index person for search
search
    .index_person(person_view)
    .await?;

// Full-text search
let results = search
    .search(SearchQuery {
        text: "rust developer",
        filters: vec![
            Filter::Status(PersonLifecycle::Active),
            Filter::HasSkill("Rust"),
        ],
        limit: 20,
    })
    .await?;
```

### Message Queue Integration

```rust
use cim_domain_person::infrastructure::queue::QueueAdapter;

let queue = QueueAdapter::new(queue_client);

// Publish to queue
queue
    .publish("person-updates", &update_message)
    .await?;

// Consume from queue
queue
    .consume("person-commands", |message| async {
        process_command(message).await
    })
    .await?;
```

## Integration Patterns

### Saga Pattern

Coordinate multi-domain transactions.

```rust
pub struct PersonOnboardingSaga {
    person_domain: PersonDomain,
    identity_domain: IdentityDomain,
    notification_domain: NotificationDomain,
}

impl PersonOnboardingSaga {
    pub async fn execute(&self, request: OnboardingRequest) -> Result<(), SagaError> {
        // Step 1: Create person
        let person = self.person_domain
            .create_person(request.person_data)
            .await
            .map_err(|e| self.compensate_person_creation(e))?;
        
        // Step 2: Create identity
        let identity = self.identity_domain
            .create_identity(person.id, request.credentials)
            .await
            .map_err(|e| self.compensate_identity_creation(person.id, e))?;
        
        // Step 3: Send welcome notification
        self.notification_domain
            .send_welcome(person.id, person.email)
            .await
            .map_err(|e| self.compensate_notification(person.id, identity.id, e))?;
        
        Ok(())
    }
}
```

### CQRS with Event Sourcing

Separate read and write models with event sourcing.

```rust
// Write side - Commands produce events
let events = command_processor
    .process(CreatePerson { ... })
    .await?;

// Events stored in event store
event_store.append(person_id, events).await?;

// Read side - Events update projections
for event in events {
    projection_updater.update(&event).await?;
}

// Query from optimized read model
let person_view = query_processor
    .process(GetPersonById { ... })
    .await?;
```

### Circuit Breaker Pattern

Protect against cascading failures.

```rust
use cim_domain_person::infrastructure::retry::CircuitBreaker;

let breaker = CircuitBreaker::new(
    failure_threshold: 5,
    success_threshold: 2,
    timeout: Duration::from_secs(30),
);

// Wrap external calls
let result = breaker
    .execute(|| async {
        external_service.call().await
    })
    .await?;
```

## Testing Integration

### Integration Test Setup

```rust
#[tokio::test]
async fn test_cross_domain_integration() {
    // Setup test environment
    let test_env = IntegrationTestEnvironment::new().await;
    
    // Create person in person domain
    let person = test_env
        .person_domain
        .create_person(test_data::person())
        .await?;
    
    // Verify identity created in identity domain
    let identity = test_env
        .identity_domain
        .get_identity_for_person(person.id)
        .await?;
    assert!(identity.is_some());
    
    // Verify notification sent
    let notifications = test_env
        .notification_domain
        .get_sent_notifications(person.id)
        .await?;
    assert_eq!(notifications.len(), 1);
}
```

### Mock External Services

```rust
use mockito::{mock, server_url};

#[tokio::test]
async fn test_external_api_integration() {
    // Mock external API
    let _m = mock("POST", "/validate-address")
        .with_status(200)
        .with_body(r#"{"valid": true, "normalized": "123 Main St"}"#)
        .create();
    
    // Configure integration to use mock
    let integration = LocationIntegration::new(
        LocationClient::with_base_url(&server_url())
    );
    
    // Test integration
    let result = integration
        .validate_address(&test_address())
        .await?;
    assert!(result.valid);
}
```

## Monitoring Integration

### Metrics Collection

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref INTEGRATION_CALLS: Counter = Counter::new(
        "person_integration_calls_total",
        "Total integration calls"
    ).unwrap();
    
    static ref INTEGRATION_DURATION: Histogram = Histogram::new(
        "person_integration_duration_seconds",
        "Integration call duration"
    ).unwrap();
}

// Track integration metrics
let timer = INTEGRATION_DURATION.start_timer();
let result = external_service.call().await;
timer.observe_duration();
INTEGRATION_CALLS.inc();
```

### Distributed Tracing

```rust
use opentelemetry::trace::{Tracer, SpanKind};

let tracer = global::tracer("person-domain");

let span = tracer
    .span_builder("cross-domain-call")
    .with_kind(SpanKind::Client)
    .with_attributes(vec![
        KeyValue::new("domain", "identity"),
        KeyValue::new("operation", "create_identity"),
    ])
    .start(&tracer);

let result = identity_domain
    .create_identity(person_id)
    .await;

span.end();
```