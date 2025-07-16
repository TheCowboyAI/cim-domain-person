# Changelog

## [0.3.0] - 2024-01-20

- **Complete refactoring to ECS (Entity Component System) architecture**
- Minimal Person aggregate with only core identity
- Component Registry for managing person data as components
- Lifecycle management (Active, Suspended, Deactivated, Merged)
- Event sourcing with CID chains
- Full test coverage with 83 tests passing

### Added
- ECS-aligned Person aggregate
- Component Registry system
- PersonLifecycle state machine
- Command and Event handlers
- Value objects: PersonName, EmailAddress, PhoneNumber
- Infrastructure: Event store, persistence, NATS integration
- Cross-domain integration framework
- Query service with 5 projection types
- Comprehensive test suite

### Changed
- Person aggregate now only contains core identity
- All other data managed as components
- Events follow past-tense naming convention
- Commands use imperative naming

## [0.2.0] - Previous version
- Traditional aggregate design
- All person data in single aggregate 