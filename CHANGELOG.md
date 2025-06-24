# Changelog

## [0.3.0] - 2025-01-24

### Changed - BREAKING CHANGES
- **Complete refactoring to ECS (Entity Component System) architecture**
  - Person aggregate now focuses only on core identity (ID, name, birth/death dates)
  - All other data (addresses, employment, skills, etc.) are now ECS components
  - Removed direct storage of complex value objects from Person aggregate

### Added
- `components` module with ECS-oriented component definitions:
  - `EmailComponent`, `PhoneComponent` for contact information
  - `SkillComponent`, `CertificationComponent` for professional capabilities
  - `CommunicationPreferencesComponent`, `PrivacyPreferencesComponent` for preferences
- `cross_domain` module for managing relationships with other domains:
  - `person_location` for person-address relationships (uses location domain)
  - `person_organization` for employment relationships (uses organization domain)
- Component registration tracking in Person aggregate
- ECS-focused projections (`PersonView`, `PersonListItem`, `PersonStatistics`)

### Removed
- `event_store.rs` - moved to shared infrastructure in `cim-domain`
- Old value objects that violated domain boundaries:
  - `PhysicalAddress` - use `cim-domain-location::Address`
  - `Employment` - use cross-domain relationships
  - Complex structs like `Skill`, `Certification`, etc. - now ECS components
- Old commands and events for managing non-core data

### Deprecated
- Legacy command and event stubs marked as deprecated for migration period
- Will be removed in v0.4.0

### Migration Guide
To migrate from v0.2.x to v0.3.0:

1. **Addresses**: Use location domain to create addresses, then associate with person
2. **Employment**: Use cross-domain service to manage person-organization relationships
3. **Skills/Certifications**: Create as separate ECS components and attach to person entity
4. **Contact Info**: Use `EmailComponent` and `PhoneComponent` instead of direct storage

## [0.2.0] - Previous version
- Traditional DDD aggregate with all data stored directly 