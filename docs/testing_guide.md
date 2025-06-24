# Person Domain Testing Guide

## Overview

This guide covers testing strategies, patterns, and best practices for the Person domain. All tests follow TDD principles and include Mermaid diagrams documenting test scenarios.

## Test Categories

### 1. Unit Tests

#### Aggregate Tests
```rust
#[cfg(test)]
mod person_aggregate_tests {
    use super::*;
    
    /// Test basic person creation
    /// 
    /// ```mermaid
    /// graph LR
    ///     A[Create Command] --> B[New Person]
    ///     B --> C[PersonCreated Event]
    ///     C --> D[Active State]
    /// ```
    #[test]
    fn test_create_person() {
        // Arrange
        let person_id = PersonId::new();
        let name = PersonName::new("Alice", "Johnson");
        
        // Act
        let person = Person::new(person_id, name.clone());
        
        // Assert
        assert_eq!(person.id(), person_id);
        assert_eq!(person.core_identity().legal_name, name.display_name());
        assert!(matches!(person.lifecycle(), PersonLifecycle::Active));
        assert_eq!(person.component_count(), 0);
    }
    
    /// Test lifecycle transitions
    /// 
    /// ```mermaid
    /// stateDiagram-v2
    ///     [*] --> Active
    ///     Active --> Deactivated: Deactivate
    ///     Active --> Deceased: Record Death
    ///     Active --> MergedInto: Merge
    ///     Deactivated --> Active: Reactivate
    ///     Deactivated --> Deceased: Record Death
    ///     Deceased --> [*]
    ///     MergedInto --> [*]
    /// ```
    #[test]
    fn test_lifecycle_transitions() {
        let mut person = create_test_person();
        
        // Test deactivation
        let result = person.deactivate("Test reason", "system");
        assert!(result.is_ok());
        assert!(matches!(
            person.lifecycle(),
            PersonLifecycle::Deactivated { .. }
        ));
        
        // Cannot modify deactivated person
        let name_update = person.update_name(
            PersonName::new("New", "Name"),
            "Test"
        );
        assert!(matches!(
            name_update,
            Err(PersonDomainError::CannotModifyInactivePerson)
        ));
    }
}
```

#### Component Tests
```rust
#[cfg(test)]
mod component_tests {
    /// Test component registration
    /// 
    /// ```mermaid
    /// graph TB
    ///     A[Person] --> B{Register Component}
    ///     B -->|Success| C[Component Registered]
    ///     B -->|Already Exists| D[Error]
    ///     C --> E[ComponentRegistered Event]
    /// ```
    #[test]
    fn test_component_registration() {
        let mut person = create_test_person();
        
        // Register email component
        let result = person.register_component(
            ComponentType::EmailAddress,
            "test_system"
        );
        assert!(result.is_ok());
        
        // Verify event generated
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            PersonEvent::ComponentRegistered { .. }
        ));
        
        // Cannot register same component twice
        let duplicate = person.register_component(
            ComponentType::EmailAddress,
            "test_system"
        );
        assert!(matches!(
            duplicate,
            Err(PersonDomainError::ComponentAlreadyRegistered(_))
        ));
    }
}
```

### 2. Integration Tests

#### Cross-Domain Tests
```rust
#[tokio::test]
async fn test_person_organization_integration() {
    /// Test employment relationship creation
    /// 
    /// ```mermaid
    /// sequenceDiagram
    ///     participant P as Person Domain
    ///     participant O as Organization Domain
    ///     participant E as Event Bus
    ///     
    ///     P->>E: PersonCreated
    ///     O->>E: OrganizationCreated
    ///     P->>O: EstablishEmployment
    ///     O->>E: EmploymentEstablished
    ///     P->>P: RegisterComponent(Employment)
    ///     P->>E: ComponentRegistered
    /// ```
    
    // Setup
    let person_id = PersonId::new();
    let org_id = OrganizationId::new();
    let event_bus = create_test_event_bus();
    
    // Create person
    let person_events = create_person(person_id, "John Doe").await?;
    event_bus.publish_all(person_events).await?;
    
    // Create organization (simulated)
    event_bus.publish(OrganizationCreated {
        organization_id: org_id,
        name: "TechCorp".to_string(),
    }).await?;
    
    // Establish employment
    let employment = EstablishEmployment {
        person_id,
        organization_id: org_id,
        role: "Software Engineer".to_string(),
        start_date: today(),
    };
    
    let result = cross_domain_handler
        .handle_employment(employment)
        .await?;
    
    // Verify events
    assert!(result.events.iter().any(|e| matches!(
        e,
        DomainEvent::EmploymentEstablished { .. }
    )));
    
    // Verify person has employment component registered
    let person = load_person(person_id).await?;
    assert!(person.has_component(&ComponentType::Employment));
}
```

#### Network Analysis Tests
```rust
#[tokio::test]
async fn test_influence_calculation() {
    /// Test influence score calculation
    /// 
    /// ```mermaid
    /// graph TB
    ///     A[Central Person] --> B[Person 1]
    ///     A --> C[Person 2]
    ///     A --> D[Person 3]
    ///     B --> E[Person 4]
    ///     C --> E
    ///     D --> F[Person 5]
    ///     
    ///     style A fill:#f9f,stroke:#333,stroke-width:4px
    /// ```
    
    // Build test network
    let network = TestNetworkBuilder::new()
        .add_person("central", "Central Person")
        .add_person("p1", "Person 1")
        .add_person("p2", "Person 2")
        .add_person("p3", "Person 3")
        .add_person("p4", "Person 4")
        .add_person("p5", "Person 5")
        .connect("central", "p1", RelationshipType::Colleague)
        .connect("central", "p2", RelationshipType::Colleague)
        .connect("central", "p3", RelationshipType::Colleague)
        .connect("p1", "p4", RelationshipType::Manager)
        .connect("p2", "p4", RelationshipType::Mentor)
        .connect("p3", "p5", RelationshipType::Partner)
        .build()
        .await?;
    
    // Calculate influence scores
    let analyzer = InfluenceAnalyzer::new(&network);
    let central_score = analyzer.calculate_influence_score("central");
    let p1_score = analyzer.calculate_influence_score("p1");
    let p5_score = analyzer.calculate_influence_score("p5");
    
    // Central person should have highest influence
    assert!(central_score.overall > p1_score.overall);
    assert!(central_score.overall > p5_score.overall);
    
    // Verify betweenness centrality
    assert!(central_score.betweenness_centrality > 0.0);
}
```

### 3. Performance Tests

```rust
#[test]
fn test_large_network_query_performance() {
    /// Test query performance on large networks
    /// 
    /// ```mermaid
    /// graph LR
    ///     A[Generate 10K People] --> B[Create Relationships]
    ///     B --> C[Build Indices]
    ///     C --> D[Execute Queries]
    ///     D --> E{< 10ms?}
    ///     E -->|Yes| F[Pass]
    ///     E -->|No| G[Fail]
    /// ```
    
    // Generate large dataset
    let people = generate_people(10_000);
    let relationships = generate_random_relationships(&people, 50_000);
    
    // Build indices
    let index = NetworkIndex::build(&people, &relationships);
    
    // Test query performance
    let start = Instant::now();
    let results = index.find_people_by_skill("Rust", Some(ProficiencyLevel::Expert));
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 10);
    assert!(!results.is_empty());
}
```

### 4. Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    /// Test that merging preserves all components
    /// 
    /// ```mermaid
    /// graph LR
    ///     A[Source Person] --> B{Merge}
    ///     C[Target Person] --> B
    ///     B --> D[Merged Person]
    ///     E[Components A] --> D
    ///     F[Components B] --> D
    /// ```
    #[test]
    fn test_merge_preserves_components(
        source_components in prop::collection::vec(component_type_strategy(), 0..10),
        target_components in prop::collection::vec(component_type_strategy(), 0..10)
    ) {
        let mut source = create_test_person();
        let mut target = create_test_person();
        
        // Add components
        for comp in &source_components {
            source.register_component(comp.clone(), "test").ok();
        }
        for comp in &target_components {
            target.register_component(comp.clone(), "test").ok();
        }
        
        // Merge
        let merge_result = merge_persons(source, target);
        
        // All components should be noted for migration
        let all_components: HashSet<_> = source_components
            .iter()
            .chain(target_components.iter())
            .cloned()
            .collect();
        
        assert_eq!(
            merge_result.components_to_migrate.len(),
            source_components.len()
        );
    }
}
```

## Test Data Builders

```rust
/// Builder for creating test persons with specific attributes
pub struct TestPersonBuilder {
    name: PersonName,
    lifecycle: PersonLifecycle,
    components: Vec<ComponentType>,
}

impl TestPersonBuilder {
    pub fn new() -> Self {
        Self {
            name: PersonName::new("Test", "Person"),
            lifecycle: PersonLifecycle::Active,
            components: Vec::new(),
        }
    }
    
    pub fn with_name(mut self, given: &str, family: &str) -> Self {
        self.name = PersonName::new(given, family);
        self
    }
    
    pub fn deactivated(mut self, reason: &str) -> Self {
        self.lifecycle = PersonLifecycle::Deactivated {
            reason: reason.to_string(),
            since: Utc::now(),
        };
        self
    }
    
    pub fn with_components(mut self, components: Vec<ComponentType>) -> Self {
        self.components = components;
        self
    }
    
    pub fn build(self) -> Person {
        let mut person = Person::new(PersonId::new(), self.name);
        person.lifecycle = self.lifecycle;
        
        for component in self.components {
            person.register_component(component, "builder").ok();
        }
        
        person
    }
}
```

## Testing Best Practices

### 1. Test Organization

```
tests/
├── unit/
│   ├── aggregate_tests.rs
│   ├── component_tests.rs
│   └── value_object_tests.rs
├── integration/
│   ├── cross_domain_tests.rs
│   ├── network_tests.rs
│   └── event_flow_tests.rs
├── performance/
│   └── benchmarks.rs
└── common/
    ├── builders.rs
    └── fixtures.rs
```

### 2. Test Naming Conventions

- Unit tests: `test_<what>_<condition>_<expected_result>`
- Integration tests: `test_<scenario>_integration`
- Performance tests: `bench_<operation>_<scale>`

### 3. Assertion Patterns

```rust
// Use specific error matching
assert!(matches!(
    result,
    Err(PersonDomainError::InvalidLifecycleTransition { .. })
));

// Verify event contents
assert_event!(
    events[0],
    PersonEvent::ComponentRegistered {
        component_type: ComponentType::EmailAddress,
        ..
    }
);

// Check multiple conditions
assert_person_state!(
    person,
    lifecycle: Active,
    components: [EmailAddress, PhoneNumber],
    name: "John Doe"
);
```

### 4. Test Coverage Requirements

- Minimum 95% code coverage
- All commands must have tests
- All event handlers must have tests
- All error paths must be tested
- All cross-domain interactions must have integration tests

## Continuous Integration

```yaml
# .github/workflows/test.yml
name: Person Domain Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Run unit tests
        run: cargo test --lib
        
      - name: Run integration tests
        run: cargo test --test '*'
        
      - name: Run benchmarks
        run: cargo bench --no-run
        
      - name: Check coverage
        run: cargo tarpaulin --min-coverage 95
        
      - name: Generate test report
        run: cargo test -- --format json > test-results.json
``` 