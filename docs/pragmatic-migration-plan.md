# Pragmatic Migration Plan - Person Domain to Pure Event-Driven

## Executive Summary

This plan outlines a practical 6-8 week migration from the current architecture (which is already 80% event-driven) to a pure event-driven system aligned with cim-graph patterns. The approach emphasizes incremental changes, preserving working code, and minimal disruption.

## Migration Principles

1. **Evolutionary, not Revolutionary**: Enhance existing patterns rather than rewrite
2. **Backward Compatible**: Maintain existing APIs during transition
3. **Incremental Delivery**: Ship improvements weekly
4. **Test-Driven**: Every change validated by tests
5. **Zero Downtime**: Migration transparent to users

## Phase-by-Phase Plan

### Phase 1: Foundation Enhancement (Week 1)
**Goal**: Upgrade NATS integration and add streaming support

#### Tasks
1. **Enhance NATS Configuration**
   ```rust
   // Update infrastructure/nats_integration.rs
   - Add streaming configuration
   - Configure consumer groups
   - Set up dead letter queues
   ```

2. **Add Event Metadata**
   ```rust
   // Extend events/mod.rs
   pub struct EventMetadata {
       version: String,
       correlation_id: Uuid,
       causation_id: Option<Uuid>,
       timestamp: DateTime<Utc>,
   }
   ```

3. **Implement Retry Policies**
   - Exponential backoff
   - Circuit breaker pattern
   - Dead letter queue routing

#### Deliverables
- Enhanced NATS module with streaming
- Event metadata on all events
- Retry mechanism for failed events

### Phase 2: Async-First Refactoring (Week 2)
**Goal**: Remove all synchronous command processing

#### Tasks
1. **Convert Sync Handlers**
   ```rust
   // Before
   pub fn handle_create_person(cmd: CreatePerson) -> Result<PersonId>
   
   // After
   pub async fn handle_create_person(cmd: CreatePerson) -> Result<EventStream>
   ```

2. **Update Command Bus**
   - Make all command processing async
   - Add streaming response support
   - Remove blocking calls

3. **Async Projection Updates**
   - Convert projection handlers to async
   - Add concurrent update support

#### Deliverables
- Fully async command processing
- Streaming command responses
- Non-blocking projection updates

### Phase 3: Process Management (Weeks 3-4)
**Goal**: Add saga support for complex workflows

#### Tasks
1. **Create Process Manager Framework**
   ```rust
   // New: src/process_manager/mod.rs
   pub trait ProcessManager {
       async fn handle_command(&self, cmd: Command) -> Result<Vec<Event>>;
       async fn handle_event(&self, event: Event) -> Result<Vec<Command>>;
   }
   ```

2. **Implement Core Sagas**
   - PersonOnboardingSaga
   - PersonArchivalSaga
   - CrossDomainSyncSaga

3. **Add Compensation Logic**
   - Rollback mechanisms
   - Compensation events
   - Failure handling

#### Deliverables
- Working process manager
- 3 core sagas implemented
- Compensation for failed operations

### Phase 4: Event Evolution (Week 5)
**Goal**: Add versioning and migration support

#### Tasks
1. **Event Versioning System**
   ```rust
   // Add to events/mod.rs
   #[derive(EventVersion)]
   #[version("1.0")]
   pub struct PersonCreatedV1 { ... }
   
   #[derive(EventVersion)]
   #[version("2.0")]
   pub struct PersonCreatedV2 { ... }
   ```

2. **Migration Framework**
   - Event upcasting
   - Schema evolution
   - Backward compatibility

3. **Update Event Store**
   - Version-aware deserialization
   - Migration on read
   - Version tracking

#### Deliverables
- Event versioning implemented
- Migration framework
- Backward compatible events

### Phase 5: Enhanced Features (Week 6)
**Goal**: Add policy engine and advanced features

#### Tasks
1. **Policy Engine**
   ```rust
   // New: src/policies/mod.rs
   pub trait Policy {
       async fn evaluate(&self, event: &Event) -> Vec<Command>;
   }
   ```

2. **Core Policies**
   - AutoArchiveInactivePersons
   - SkillRecommendations
   - RelationshipCascades

3. **Monitoring Integration**
   - OpenTelemetry setup
   - Event flow tracing
   - Performance metrics

#### Deliverables
- Working policy engine
- 3 core policies
- Basic observability

### Phase 6: Migration & Optimization (Weeks 7-8)
**Goal**: Complete migration and optimize performance

#### Tasks
1. **Data Migration**
   - Event replay scripts
   - Projection rebuilding
   - Consistency validation

2. **Performance Tuning**
   - Benchmark critical paths
   - Optimize hot spots
   - Cache frequently accessed data

3. **Documentation & Training**
   - Update API docs
   - Migration guide
   - Team training

#### Deliverables
- Migrated data
- Performance benchmarks
- Complete documentation

## Implementation Timeline

```
Week 1: Foundation Enhancement
Week 2: Async-First Refactoring
Week 3-4: Process Management
Week 5: Event Evolution
Week 6: Enhanced Features
Week 7-8: Migration & Optimization
```

## Risk Mitigation

### Technical Risks
1. **Data Loss**
   - Mitigation: Dual-write during migration
   - Rollback: Keep old system running

2. **Performance Degradation**
   - Mitigation: Incremental rollout
   - Monitoring: Real-time metrics

3. **API Breaking Changes**
   - Mitigation: Compatibility layer
   - Versioning: API v1 maintained

### Operational Risks
1. **Team Availability**
   - Buffer: 2-week contingency
   - Cross-training: Knowledge sharing

2. **Integration Issues**
   - Testing: Integration test suite
   - Coordination: Weekly sync meetings

## Success Metrics

### Week-by-Week Goals
- Week 1: NATS streaming operational
- Week 2: 100% async processing
- Week 3-4: 3 sagas in production
- Week 5: Event versioning live
- Week 6: Policy engine running
- Week 7-8: Full migration complete

### Final Success Criteria
1. **Performance**: <10ms p95 event processing
2. **Reliability**: 99.9% message delivery
3. **Scalability**: 10k events/second
4. **Compatibility**: Zero breaking changes
5. **Testing**: >90% code coverage

## Budget & Resources

### Development Effort
- 1-2 senior developers
- 6-8 weeks duration
- ~320 hours total effort

### Infrastructure
- NATS cluster (existing)
- Monitoring stack (minimal additions)
- No new infrastructure required

## Post-Migration Roadmap

1. **Advanced Features** (Month 2)
   - Complex event processing
   - Machine learning policies
   - Real-time analytics

2. **Scale Optimization** (Month 3)
   - Horizontal scaling
   - Multi-region support
   - Performance optimization

3. **Domain Evolution** (Ongoing)
   - New component types
   - Enhanced projections
   - Cross-domain workflows

## Conclusion

This pragmatic approach delivers a pure event-driven architecture in 6-8 weeks by:
- Building on existing strengths
- Making incremental improvements
- Maintaining backward compatibility
- Focusing on high-value changes

The result is a modern, scalable, event-driven person domain that aligns with cim-graph patterns while preserving the investment in existing code.