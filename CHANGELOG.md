# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2025-11-07 - NATS Service Deployment and Production Readiness

### Added
- **Service Binary**: `person-service` - Long-running daemon that responds to NATS commands
- **Systemd Deployment**: Complete systemd service configuration with security hardening
  - Service unit file with extensive security restrictions
  - Environment configuration template
  - Automated installation script
- **NixOS Module**: Declarative NixOS configuration for deployment
  - Full NixOS module with configuration options
  - Nix flake for easy deployment
  - Comprehensive NixOS documentation
- **NATS Integration Tests**: Comprehensive test suite for NATS cluster
  - `nats_simple_test.rs` - Basic connectivity and pub/sub
  - `nats_jetstream_test.rs` - JetStream functionality verification
  - `nats_cluster_test.rs` - Full Person domain integration
- **Deployment Documentation**: Complete deployment guides
  - Systemd deployment guide
  - NixOS deployment guide
  - NATS cluster test results
  - Production deployment checklist

### Changed
- **NatsEventStore**: Improved stream creation to handle existing streams gracefully
- **Service Architecture**: Person domain now deployable as standalone NATS-responding service
- **Documentation**: Enhanced with production deployment instructions

### Infrastructure
- **Command Handler**: PersonCommandHandler listens on `person.commands.>` subject
- **Event Publishing**: Events published to `person.events.>` subject
- **JetStream**: Durable event storage via NATS JetStream
- **Snapshots**: In-memory snapshot store for performance

### Security
- Service runs as unprivileged user (cim-person)
- Read-only filesystem except data directory
- Restricted system calls and network access
- Private /tmp directory
- No new privileges capability
- Memory execution protection

### Testing
- ✅ Verified against NATS cluster at 10.0.0.41:4222
- ✅ Basic pub/sub functionality
- ✅ JetStream stream creation and message persistence
- ✅ Event sourcing workflow
- ✅ Command handling and event publishing

## [0.7.8] - 2025-01-15 - FRP/CT Compliance and Documentation Cleanup

### Added
- **Attribute filtering methods**: `physical_attributes()`, `demographic_attributes()` on PersonAttributeSet
- **Comprehensive documentation**: Complete README rewrite matching current implementation
- **Documentation status tracking**: DOCUMENTATION_STATUS.md and CLEANUP_PLAN.md

### Changed
- **README.md**: Complete rewrite to accurately reflect pure functional event sourcing architecture
- **Documentation structure**: Consolidated /docs/ into /doc/ directory
- **Category Theory**: Enhanced attribute predicates (`is_physical()`, `is_demographic()`)

### Removed
- Duplicate documentation files (changelog.md, readme.md)
- Temporary status files (cleanup-complete.md, implementation-complete.md, phase*.md)
- Outdated documentation directory (/docs/)
- Outdated components.md (referenced non-existent ECS components)

### Fixed
- All compiler warnings resolved (zero warnings build)
- Example code compilation errors fixed
- Documentation accurately matches implementation

## [0.7.0] - 2025-01-14 - Category Theory Implementation

### Added
- **Formal Category Theory traits**: Functor, Monad, Applicative, Coalgebra, NaturalTransformation
- **Pure projection functions**: (State, Event) → NewState pattern with zero side effects
- **Explicit CQRS API**: PersonService with compile-time command/query separation
- **Query specifications**: PersonSummaryQuery, PersonSearchQuery, SkillsQuery, NetworkQuery, TimelineQuery
- **Category theory module**: `src/category_theory.rs` with formal trait definitions
- **Pure projections module**: `src/projections/pure_projections.rs`
- **Marker traits**: CommandOperation and QueryOperation for type safety

### Changed
- **Projection pattern**: Infrastructure adapters now use pure projection functions
- **PersonAttributeSet**: Implements Monad for compositional operations
- **PersonAttribute**: Implements Functor for structure-preserving transformations
- **FRP compliance**: Achieved 100% FRP/Category Theory compliance

### Documentation
- **FRP-CT-COMPLIANCE.md**: Updated to 100% compliance status
- **person-attributes-category-theory.md**: Complete mathematical foundations
- **person-attributes-design.md**: EAV pattern and design decisions
- **person-names-design.md**: International name handling

## [0.6.0] - 2024-12-XX - EAV Pattern and Attribute System

### Added
- **PersonAttributeSet**: Collection-based attribute management with filtering
- **Attribute categories**: Identifying, Physical, Healthcare, Demographic, Custom
- **Temporal validity**: Time-bounded attribute values
- **Provenance tracking**: Full data lineage with transformation history
- **Confidence levels**: Certain, Likely, Possible, Speculative
- **Attribute sources**: DocumentVerified, Measured, SelfReported, Imported, etc.

### Changed
- **Person aggregate**: Simplified to core_identity + attributes pattern
- **Event sourcing**: Pure functional `apply_event_pure()` method
- **Component model**: Replaced with flexible attribute-value pairs

### Removed
- Individual component types (EmailComponent, PhoneComponent, etc.)
- Component registry system
- Hard-coded person properties

## [0.5.0] - 2024-11-XX - Event Sourcing Refinement

### Added
- **MealyStateMachine integration**: Pure functional state transitions
- **PersonLifecycle**: Active, Deactivated, Deceased, Merged states
- **RecordAttribute command**: Add/update attributes via event sourcing
- **AttributeRecorded event**: Track attribute changes
- **Pure functional patterns**: Zero mutation in domain logic

### Changed
- **Command processing**: Switched to MealyStateMachine::output pattern
- **Event application**: Pure functions returning new state
- **Aggregate structure**: Minimalist core with extensible attributes

## [0.4.0] - 2024-01-XX - Pure Event-Driven Architecture

### Added

#### Infrastructure
- NATS JetStream integration for event persistence and streaming
- Async command processor with streaming results
- Retry handler with exponential backoff and circuit breaker
- Dead letter queue support for failed events
- Event metadata with correlation and causation tracking

#### State Machines
- Generic state machine framework for aggregates
- Onboarding workflow as state machine example
- Guard conditions for state transitions
- Entry/exit actions for states

#### Event Versioning
- Event versioning framework with automatic migration
- Migration registry for schema evolution
- V1 → V2 → V3 migration chain support
- Backward compatibility maintained

#### Policy Engine
- Simple policy engine for reactive business rules
- Default policies: WelcomeEmail, SkillRecommendation, DataQuality, AutoArchive
- Composable policy system
- Async policy evaluation

#### Testing & Tools
- Comprehensive integration test suite (5 modules)
- Performance benchmarks for all major operations
- Migration scripts with dry-run support
- Migration verification tool

### Changed

#### Architecture
- All command processing is now async with streaming support
- Events flow through NATS JetStream by default
- Sagas replaced with state machine aggregates
- Synchronous handlers converted to async
- Event structure enhanced with metadata

#### API
- `CommandHandler` → `AsyncCommandProcessor`
- `handle()` → `process()` (async)
- `PersonEvent` → `PersonEventV2` with metadata
- Command results now include optional event streams

### Migration Guide

1. **Update Dependencies**
   ```toml
   cim-domain-person = "0.4.0"
   ```

2. **Update Command Processing**
   ```rust
   // Old
   let events = handler.handle(command)?;
   
   // New
   let result = processor.process(command).await?;
   let events = result.events;
   ```

3. **Handle Event Streams**
   ```rust
   if let Some(mut stream) = result.event_stream {
       while let Some(event) = stream.next().await {
           // Process streamed events
       }
   }
   ```

4. **Use Policy Engine**
   ```rust
   let policy_engine = create_default_policy_engine();
   let commands = policy_engine.evaluate(&event).await;
   ```

5. **Run Migration**
   ```bash
   cargo run --bin migrate_events --features migration
   ```

### Performance Improvements
- Event processing: ~50μs per event
- Command handling: ~200μs per command (async)
- Policy evaluation: ~100μs per event
- Concurrent processing scales linearly

### Documentation
- Pure event-driven architecture guide
- State machine patterns documentation
- Policy engine usage examples
- Migration tooling documentation

## [0.3.0] - Previous Release

### Added
- ECS architecture implementation
- Component system for person attributes
- Cross-domain relationships
- Basic event sourcing

### Changed
- Moved from traditional aggregate to ECS model
- Simplified person core to ID + name only
- Components handle all other attributes

## [0.2.0] - Initial Event Sourcing

### Added
- Basic event sourcing implementation
- In-memory event store
- Command/event separation
- Simple projections

## [0.1.0] - Initial Release

### Added
- Basic person domain model
- CRUD operations
- Value objects for name, email, etc.