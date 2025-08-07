# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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