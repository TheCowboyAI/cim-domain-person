//! Performance benchmarks for event processing

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use cim_domain_person::{
    events::{PersonEventV2, EventMetadata, create_event_registry},
    aggregate::PersonId,
    value_objects::PersonName,
    policies::create_default_policy_engine,
    handlers::AsyncCommandProcessor,
    infrastructure::{InMemoryEventStore, InMemorySnapshotStore, InMemoryComponentStore},
    commands::{PersonCommand, CreatePerson},
};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_event_versioning(c: &mut Criterion) {
    let registry = create_event_registry();
    
    // V1 event for migration
    let v1_event = serde_json::json!({
        "version": "1.0",
        "person_id": "550e8400-e29b-41d4-a716-446655440000",
        "name": {
            "first_name": "Bench",
            "middle_name": "Mark",
            "last_name": "Test"
        },
        "source": "benchmark",
        "created_at": "2023-01-01T00:00:00Z"
    });
    
    c.bench_function("migrate_v1_to_v2", |b| {
        b.iter(|| {
            let event = v1_event.clone();
            let migrated = registry.migrate_to_current("PersonCreated", event).unwrap();
            black_box(migrated);
        })
    });
    
    // Benchmark serialization
    let person_id = PersonId::new();
    let event = PersonEventV2::Created {
        person_id,
        name: PersonName::new("Bench", None, "Test").unwrap(),
        source: "test".to_string(),
        metadata: EventMetadata::new(),
    };
    
    c.bench_function("serialize_event_v2", |b| {
        b.iter(|| {
            let json = serde_json::to_string(&event).unwrap();
            black_box(json);
        })
    });
    
    let json = serde_json::to_string(&event).unwrap();
    c.bench_function("deserialize_event_v2", |b| {
        b.iter(|| {
            let event: PersonEventV2 = serde_json::from_str(&json).unwrap();
            black_box(event);
        })
    });
}

fn bench_policy_engine(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let engine = create_default_policy_engine();
    
    let person_id = PersonId::new();
    let event = PersonEventV2::Created {
        person_id,
        name: PersonName::new("Policy", None, "Bench").unwrap(),
        source: "benchmark".to_string(),
        metadata: EventMetadata::new(),
    };
    
    c.bench_function("evaluate_policies", |b| {
        b.iter(|| {
            rt.block_on(async {
                let commands = engine.evaluate(&event).await;
                black_box(commands);
            })
        })
    });
}

fn bench_async_command_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Setup infrastructure
    let event_store = Arc::new(InMemoryEventStore::new());
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let component_store = Arc::new(InMemoryComponentStore::new());
    
    let processor = AsyncCommandProcessor::new(
        event_store,
        snapshot_store,
        component_store,
    );
    
    let mut group = c.benchmark_group("command_processing");
    
    for size in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    for i in 0..size {
                        let person_id = PersonId::new();
                        let command = PersonCommand::CreatePerson(CreatePerson {
                            person_id,
                            name: PersonName::new(&format!("Person{}", i), None, "Test").unwrap(),
                            source: "benchmark".to_string(),
                        });
                        
                        let result = processor.process(command).await.unwrap();
                        black_box(result);
                    }
                })
            });
        });
    }
    
    group.finish();
}

fn bench_state_machine(c: &mut Criterion) {
    use cim_domain_person::aggregate::{
        person_onboarding::{OnboardingAggregate, OnboardingCommand},
    };
    
    c.bench_function("state_machine_transitions", |b| {
        b.iter(|| {
            let person_id = PersonId::new();
            let mut aggregate = OnboardingAggregate::new(
                person_id,
                PersonName::new("State", None, "Machine").unwrap(),
            );
            
            // Run through complete workflow
            aggregate.handle(OnboardingCommand::AddEmail {
                email: "test@example.com".to_string(),
            }).unwrap();
            
            aggregate.handle(OnboardingCommand::VerifyEmail {
                token: "test-token".to_string(),
            }).unwrap();
            
            aggregate.handle(OnboardingCommand::AddSkills {
                skills: vec!["Rust".to_string(), "Event Sourcing".to_string()],
            }).unwrap();
            
            aggregate.handle(OnboardingCommand::CompleteOnboarding).unwrap();
            
            black_box(aggregate);
        })
    });
}

criterion_group!(
    benches,
    bench_event_versioning,
    bench_policy_engine,
    bench_async_command_processing,
    bench_state_machine
);
criterion_main!(benches);