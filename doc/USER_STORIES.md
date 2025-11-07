# Person Domain User Stories

This document defines the functional requirements for the Person domain through user stories.

## Epic 1: Core Identity Management

### US-1.1: Create Person with Culturally-Aware Name

**As a** system administrator
**I want** to create a person record with culturally-appropriate name structure
**So that** names from any culture are represented accurately and respectfully

**Acceptance Criteria:**
- ✓ Can create PersonName with given names and family names as lists
- ✓ Support Western convention (Given Family)
- ✓ Support Spanish convention (Given Paternal Maternal)
- ✓ Support East Asian convention (Family Given)
- ✓ Support Patronymic convention (Given Patronymic)
- ✓ Support Mononymic convention (single name)
- ✓ PersonName is immutable after creation

**Test Scenarios:**
- Create Western name: "John Michael Doe"
- Create Spanish name: "María García López"
- Create East Asian name: "山田 太郎" (Yamada Taro)
- Create Patronymic name: "Ivan Petrovich"
- Create Mononymic name: "Prince"

---

### US-1.2: Display Names with Cultural Conventions

**As a** UI developer
**I want** to display names according to cultural conventions
**So that** names appear correctly for the person's culture

**Acceptance Criteria:**
- ✓ Western: "Given Family" (e.g., "John Doe")
- ✓ Spanish: "Given Paternal Maternal" (e.g., "María García López")
- ✓ EastAsian: "Family Given" (e.g., "山田 太郎")
- ✓ Patronymic: "Given Patronymic" (e.g., "Ivan Petrovich")
- ✓ Mononymic: "Given" (e.g., "Prince")
- ✓ Can override display policy per-use-case

**Test Scenarios:**
- Display same person with different policies
- Verify correct ordering for each convention
- Test formal vs informal display

---

### US-1.3: Manage Person Titles

**As a** system administrator
**I want** to record professional and honorific titles with temporal validity
**So that** titles are accurate for specific time periods

**Acceptance Criteria:**
- ✓ Support professional titles (Dr., Prof., Rev.)
- ✓ Support honorific titles (Sir, Dame, Lord)
- ✓ Support nobility titles (Duke, Count)
- ✓ Titles have start dates
- ✓ Titles can have end dates (optional)
- ✓ Can query titles valid at specific date

**Test Scenarios:**
- Add "Dr." title with medical degree date
- Add "Prof." title with tenure date
- Query titles valid in 2020 vs 2024
- Handle person with multiple concurrent titles

---

### US-1.4: Track Name Changes Over Time

**As a** compliance officer
**I want** to track all historical names for a person
**So that** we maintain audit trail and can find persons by previous names

**Acceptance Criteria:**
- ✓ NameUpdated event captures old and new names
- ✓ Event includes reason for change (marriage, legal change, etc.)
- ✓ Event includes timestamp
- ✓ Cannot modify historical name change events
- ✓ Can reconstruct name at any point in time

**Test Scenarios:**
- Record name change due to marriage
- Record name change due to legal process
- Query person's name in 2020 (before change)
- Query person's name in 2024 (after change)

---

## Epic 2: Extensible Attribute System

### US-2.1: Record Identifying Attributes

**As a** data analyst
**I want** to record identifying attributes with provenance
**So that** we can disambiguate between persons with similar names

**Acceptance Criteria:**
- ✓ Can record birth datetime with precision level (exact, year, decade, century)
- ✓ Can record birth place (cross-domain reference to location)
- ✓ Can record national ID numbers
- ✓ Each attribute has source (system, user, import, integration)
- ✓ Each attribute has confidence level (high, medium, low)
- ✓ Can track transformation trace for derived attributes

**Test Scenarios:**
- Record exact birth datetime with hospital source
- Record approximate birth year (1950s) with family history source
- Record national ID from government integration
- Trace SSN derived from tax system

---

### US-2.2: Record Physical Attributes

**As a** healthcare provider
**I want** to record physical characteristics with temporal validity
**So that** we track changes over time (growth, aging)

**Acceptance Criteria:**
- ✓ Can record height with unit (meters)
- ✓ Can record weight with unit (kilograms)
- ✓ Can record blood type (A+, B-, etc.)
- ✓ Can record eye color, hair color
- ✓ Can record biological sex, handedness
- ✓ Each attribute has valid_from and valid_until dates
- ✓ Can query attributes valid at specific date

**Test Scenarios:**
- Record child's height measurements over years
- Record adult's weight changes
- Query person's height in 2020 vs 2024
- Filter attributes valid on specific date

---

### US-2.3: Record Healthcare Attributes

**As a** healthcare administrator
**I want** to record healthcare identifiers
**So that** we can integrate with external healthcare systems

**Acceptance Criteria:**
- ✓ Can record medical record number (MRN)
- ✓ Can record insurance ID
- ✓ Can record organ donor status
- ✓ Each has provenance tracking
- ✓ Each has temporal validity
- ✓ Can mark attributes as invalidated with reason

**Test Scenarios:**
- Record MRN from hospital system
- Record insurance ID with policy dates
- Update organ donor status
- Invalidate old insurance ID when changed

---

### US-2.4: Query Attributes by Category and Time

**As a** application developer
**I want** to filter attributes by type and temporal validity
**So that** I only work with relevant current attributes

**Acceptance Criteria:**
- ✓ Can get all identifying attributes
- ✓ Can get all physical attributes
- ✓ Can get all healthcare attributes
- ✓ Can get all demographic attributes
- ✓ Can filter to currently valid attributes only
- ✓ Can filter to attributes valid on specific date
- ✓ Can filter by attribute type (e.g., only blood type)

**Test Scenarios:**
- Get all identifying attributes for disambiguation
- Get current physical attributes for display
- Get healthcare attributes valid during 2023
- Get person's blood type (most recent)

---

### US-2.5: Transform and Derive Attributes

**As a** data integration specialist
**I want** to map attributes using functors with provenance tracking
**So that** derived attributes maintain trace to source

**Acceptance Criteria:**
- ✓ PersonAttribute implements Functor (map operation)
- ✓ map operation preserves attribute type
- ✓ map operation preserves temporal validity
- ✓ Transformation added to provenance trace
- ✓ Can compose multiple transformations
- ✓ Transformation trace is immutable

**Test Scenarios:**
- Convert height from inches to meters
- Normalize name capitalization
- Compose multiple transformations
- Verify provenance trace includes all steps

---

## Epic 3: Temporal State Management

### US-3.1: Observe Person State at Point in Time

**As a** compliance auditor
**I want** to observe person's complete state at any historical date
**So that** I can verify what was known at that time

**Acceptance Criteria:**
- ✓ Can query person's name at historical date
- ✓ Can query person's attributes valid at date
- ✓ Can query person's lifecycle status at date
- ✓ Result excludes attributes not yet valid
- ✓ Result excludes attributes already invalid
- ✓ Result is read-only (observation, not mutation)

**Test Scenarios:**
- Observe person state on 2020-01-01
- Observe person state on 2024-01-01
- Compare differences between dates
- Verify attributes match temporal validity

---

### US-3.2: Track Temporal Validity Lifecycle

**As a** data steward
**I want** attributes to have explicit validity periods
**So that** outdated information is not used incorrectly

**Acceptance Criteria:**
- ✓ Attribute has recorded_at timestamp
- ✓ Attribute has valid_from date
- ✓ Attribute has optional valid_until date
- ✓ Can mark attribute as currently valid (no end date)
- ✓ Can invalidate attribute with reason
- ✓ Temporal queries respect validity periods

**Test Scenarios:**
- Record attribute valid from today
- Record attribute valid from past date
- Invalidate attribute with "data correction" reason
- Query currently valid vs all historical attributes

---

## Epic 4: Event Sourcing Workflows

### US-4.1: Create Person via Command

**As a** application service
**I want** to create person via CreatePerson command
**So that** creation is properly event-sourced

**Acceptance Criteria:**
- ✓ CreatePerson command includes name and source
- ✓ Command generates PersonCreated event
- ✓ Event includes correlation ID for tracing
- ✓ Event includes timestamp
- ✓ Aggregate reconstructed from event stream
- ✓ Cannot create person with same ID twice

**Test Scenarios:**
- Send CreatePerson command
- Verify PersonCreated event emitted
- Verify aggregate state matches event
- Verify event stored in event store

---

### US-4.2: Record Attribute via Command

**As a** application service
**I want** to record attributes via RecordAttribute command
**So that** attribute changes are properly event-sourced

**Acceptance Criteria:**
- ✓ RecordAttribute command includes person ID and attribute
- ✓ Command generates AttributeRecorded event
- ✓ Event includes full attribute with provenance
- ✓ Event includes timestamp
- ✓ Person aggregate updated with new attribute
- ✓ Can record multiple attributes sequentially

**Test Scenarios:**
- Send RecordAttribute for birth date
- Verify AttributeRecorded event emitted
- Verify attribute added to aggregate
- Record multiple attributes in sequence

---

### US-4.3: Update Attribute via Command

**As a** application service
**I want** to update existing attributes via UpdateAttribute command
**So that** corrections maintain audit trail

**Acceptance Criteria:**
- ✓ UpdateAttribute command includes person ID, attribute type, new value
- ✓ Command generates AttributeUpdated event
- ✓ Event includes old and new attribute values
- ✓ Event includes timestamp
- ✓ Person aggregate replaces old with new
- ✓ Original attribute preserved in event history

**Test Scenarios:**
- Update person's height measurement
- Verify AttributeUpdated event emitted
- Verify old value captured in event
- Verify aggregate has new value
- Reconstruct historical state shows old value

---

### US-4.4: Invalidate Attribute via Command

**As a** data quality manager
**I want** to invalidate incorrect attributes
**So that** bad data is marked but preserved for audit

**Acceptance Criteria:**
- ✓ InvalidateAttribute command includes person ID, attribute type, reason
- ✓ Command generates AttributeInvalidated event
- ✓ Event includes invalidation reason
- ✓ Event includes timestamp
- ✓ Attribute marked as invalid (valid_until set)
- ✓ Attribute excluded from current queries
- ✓ Attribute still available in historical queries

**Test Scenarios:**
- Invalidate incorrect birth date with "data entry error" reason
- Verify AttributeInvalidated event emitted
- Verify currently_valid() excludes it
- Verify historical queries include it

---

## Epic 5: Category Theory Compliance

### US-5.1: Functor Laws for PersonAttribute

**As a** domain architect
**I want** PersonAttribute to satisfy functor laws
**So that** transformations are mathematically sound

**Acceptance Criteria:**
- ✓ Identity law: F.map(id) = F
- ✓ Composition law: F.map(f ∘ g) = F.map(g).map(f)
- ✓ Structure preservation: temporal validity unchanged
- ✓ Provenance tracking: transformations recorded
- ✓ Type safety: compile-time guarantees

**Test Scenarios:**
- Verify identity law with various attributes
- Verify composition law with multiple transforms
- Verify temporal structure preserved
- Verify provenance trace maintained

---

### US-5.2: Monad Laws for TemporalValidity

**As a** domain architect
**I want** TemporalValidity to satisfy monad laws
**So that** temporal composition is mathematically sound

**Acceptance Criteria:**
- ✓ Left identity: return a >>= f ≡ f a
- ✓ Right identity: m >>= return ≡ m
- ✓ Associativity: (m >>= f) >>= g ≡ m >>= (λx. f x >>= g)
- ✓ Temporal coherence: composition maintains validity
- ✓ Type safety: compile-time guarantees

**Test Scenarios:**
- Verify left identity law
- Verify right identity law
- Verify associativity law
- Verify temporal consistency

---

### US-5.3: Monoid Laws for PersonAttributeSet

**As a** domain architect
**I want** PersonAttributeSet to satisfy monoid laws
**So that** attribute composition is mathematically sound

**Acceptance Criteria:**
- ✓ Left identity: empty + a = a
- ✓ Right identity: a + empty = a
- ✓ Associativity: (a + b) + c = a + (b + c)
- ✓ Composition preserves all attributes
- ✓ Type safety: compile-time guarantees

**Test Scenarios:**
- Verify left identity law
- Verify right identity law
- Verify associativity law
- Verify attribute preservation

---

### US-5.4: Coalgebra for Person Unfold

**As a** domain architect
**I want** Person aggregate to implement coalgebra unfold
**So that** state observation is mathematically sound

**Acceptance Criteria:**
- ✓ unfold: Person → PersonAttributeSet
- ✓ Temporal coherence: observe_at(date) ⊆ unfold()
- ✓ Immutability: unfold doesn't modify Person
- ✓ Completeness: unfold returns all attributes
- ✓ Type safety: compile-time guarantees

**Test Scenarios:**
- Verify unfold returns all attributes
- Verify temporal filtering subset of unfold
- Verify multiple unfolds are identical (pure)
- Verify aggregate not modified

---

## Epic 6: Lifecycle Management

### US-6.1: Deactivate Person

**As a** system administrator
**I want** to deactivate person records with reason
**So that** inactive persons are excluded from active queries

**Acceptance Criteria:**
- ✓ DeactivatePerson command includes reason
- ✓ PersonDeactivated event emitted
- ✓ Person lifecycle status = Inactive
- ✓ Active queries exclude deactivated persons
- ✓ Historical queries include deactivated persons
- ✓ Can reactivate later

**Test Scenarios:**
- Deactivate person with "account closed" reason
- Verify PersonDeactivated event
- Verify active queries exclude person
- Verify historical queries include person

---

### US-6.2: Merge Duplicate Persons

**As a** data quality manager
**I want** to merge duplicate person records
**So that** we maintain single source of truth

**Acceptance Criteria:**
- ✓ MergePerson command includes source and target IDs
- ✓ PersonMergedInto event emitted
- ✓ Event includes merge reason (duplicate, confirmed_match, etc.)
- ✓ Source person marked as merged
- ✓ Target person preserves all attributes
- ✓ Queries redirect from source to target

**Test Scenarios:**
- Merge duplicate created from different sources
- Verify PersonMergedInto event
- Verify source marked as merged
- Verify queries redirect appropriately

---

## Epic 7: Cross-Domain Integration

### US-7.1: Link Person to Location Domain

**As a** integration developer
**I want** to reference locations from Person domain
**So that** addresses are managed by location domain

**Acceptance Criteria:**
- ✓ BirthPlace stored as LocationId reference
- ✓ No duplication of location data in Person
- ✓ PersonToLocationFunctor preserves structure
- ✓ Natural transformation properties satisfied
- ✓ Events reference location IDs, not location data

**Test Scenarios:**
- Record birth place as LocationId
- Verify no embedded location data
- Verify functor transformation
- Query location details separately

---

### US-7.2: Link Person to Organization Domain

**As a** integration developer
**I want** to reference organizations from Person domain
**So that** employment is managed as relationship

**Acceptance Criteria:**
- ✓ Employment stored as PersonOrganization link
- ✓ Link includes role, start date, end date
- ✓ Link references OrganizationId
- ✓ No duplication of organization data
- ✓ Events reference organization IDs

**Test Scenarios:**
- Create employment link
- Verify OrganizationId reference
- Verify role and dates
- Update employment dates

---

## Epic 8: Data Quality and Privacy

### US-8.1: Validate Attribute Quality

**As a** data quality manager
**I want** attributes to include confidence levels
**So that** we know reliability of data

**Acceptance Criteria:**
- ✓ Attribute provenance includes confidence (High, Medium, Low)
- ✓ Can filter attributes by minimum confidence
- ✓ Confidence level immutable after recording
- ✓ Derived attributes inherit source confidence
- ✓ Multiple sources can be compared

**Test Scenarios:**
- Record high-confidence government source
- Record low-confidence user-entered data
- Filter to high-confidence only
- Compare conflicting attributes by confidence

---

### US-8.2: Track Data Lineage

**As a** compliance officer
**I want** complete provenance tracking for all attributes
**So that** we can audit data sources and transformations

**Acceptance Criteria:**
- ✓ Attribute provenance includes source type
- ✓ Attribute provenance includes source identifier
- ✓ Attribute provenance includes confidence
- ✓ Attribute provenance includes transformation trace
- ✓ Provenance immutable after recording
- ✓ Can query all attributes from specific source

**Test Scenarios:**
- Record attribute from external system
- Apply transformation
- Verify complete provenance chain
- Query all attributes from source "IRS"

---

## Success Metrics

### Functional Coverage
- ✓ All user stories have passing tests
- ✓ All acceptance criteria are testable
- ✓ Tests verify actual behavior, not trivial pass
- ✓ Integration tests cover end-to-end workflows

### Code Quality
- ✓ 0 compilation errors
- ✓ 0 compilation warnings
- ✓ All Category Theory laws verified
- ✓ All public APIs documented

### Domain Purity
- ✓ No CRUD operations (event sourcing only)
- ✓ All state changes via immutable events
- ✓ All aggregates reconstructible from events
- ✓ UUID v7 used throughout

---

## Test Scenarios Summary

Each user story above includes specific test scenarios. The test implementation should:

1. **Test actual behavior** - Not just `assert!(true)` or trivial checks
2. **Use real data** - Realistic names, dates, attributes
3. **Verify events** - Check event content, not just that it was emitted
4. **Test edge cases** - Empty sets, boundary dates, invalid inputs
5. **Test invariants** - Category Theory laws, temporal consistency
6. **Test integration** - Commands → Events → Aggregate state changes
7. **Use property-based testing** where appropriate (quickcheck)

## Non-Functional Requirements

### Performance
- Temporal queries should be O(n) where n = number of attributes
- Event replay should be optimized for large event streams
- Attribute filtering should avoid unnecessary allocations

### Security
- PII attributes should be identifiable for GDPR compliance
- Provenance tracking enables audit trails
- Immutable events prevent tampering

### Maintainability
- Category Theory compliance provides formal guarantees
- Pure functional design enables local reasoning
- Event sourcing enables temporal debugging
