<!--
Copyright 2025 Cowboy AI, LLC.
SPDX-License-Identifier: MIT
-->

# CIM Person Domain NATS API Documentation

## Table of Contents

- [Overview](#overview)
- [NATS Subject Architecture](#nats-subject-architecture)
- [Command Messages](#command-messages)
- [Event Messages](#event-messages)
- [Query Messages](#query-messages)
- [Message Patterns](#message-patterns)
- [Error Handling](#error-handling)
- [Subject Algebra](#subject-algebra)

## Overview

The CIM Person Domain provides a NATS-only API for managing person profiles, identities, employment records, skills, and relationships within the CIM ecosystem. All interactions occur through NATS message passing with mandatory correlation/causation tracking. The API follows CIM Subject Algebra patterns for consistent message routing and processing.

## NATS Subject Architecture

The Person Domain follows CIM Subject Algebra with hierarchical subject structure:

```
cim.person.{version}.{operation}.{entity}.{action}
```

### Subject Hierarchy

- **Domain**: `cim.person`
- **Version**: `v1` (current version)
- **Operations**: `command`, `event`, `query`
- **Entities**: `person`, `identity`, `employment`, `skills`, `network`, `contact`, `preferences`, `demographics`
- **Actions**: Specific operations (create, update, verify, etc.)

### Example Subjects

```
cim.person.v1.command.person.create
cim.person.v1.command.identity.verify
cim.person.v1.command.employment.update
cim.person.v1.event.person.created
cim.person.v1.event.skills.certified
cim.person.v1.query.person.get
```

## Command Messages

All commands are sent as NATS messages with JSON payloads and mandatory CIM message envelope.

### Message Envelope

Every message includes the CIM message envelope:

```json
{
  "correlation_id": "uuid-v4",
  "causation_id": "uuid-v4", 
  "message_id": "uuid-v4",
  "timestamp": "2025-01-15T10:30:00Z",
  "version": "1.0",
  "subject": "cim.person.v1.command.person.create",
  "payload": { ... }
}
```

### Person Commands

#### Create Person
**Subject**: `cim.person.v1.command.person.create`
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "given_name": "John",
  "family_name": "Doe",
  "display_name": "John Doe",
  "email": "john.doe@example.com",
  "date_of_birth": "1985-06-15",
  "organization_id": "org-456",
  "metadata": {
    "source": "hr_system",
    "employee_id": "EMP-12345"
  }
}
```

#### Update Person Profile
**Subject**: `cim.person.v1.command.person.update`
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "updates": {
    "display_name": "John A. Doe",
    "phone": "+1-555-123-4567",
    "location": {
      "city": "San Francisco",
      "state": "CA",
      "country": "US"
    }
  },
  "updated_by": "user-789"
}
```

#### Archive Person
**Subject**: `cim.person.v1.command.person.archive`
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "reason": "employee_departure",
  "effective_date": "2025-01-31T23:59:59Z",
  "retention_policy": "7_years",
  "archived_by": "hr-system"
}
```

### Identity Commands

#### Create Identity
**Subject**: `cim.person.v1.command.identity.create`
```json
{
  "id": "identity-uuid",
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "identity_type": "employee_id",
  "identifier": "EMP-12345",
  "issuer": "hr_system",
  "issued_date": "2025-01-15T10:30:00Z",
  "verification_level": "verified"
}
```

#### Verify Identity
**Subject**: `cim.person.v1.command.identity.verify`
```json
{
  "identity_id": "identity-uuid",
  "verification_method": "document_check",
  "verification_data": {
    "document_type": "government_id",
    "document_number": "DL123456789",
    "expiry_date": "2028-06-15"
  },
  "verified_by": "identity-service"
}
```

### Employment Commands

#### Create Employment Record
**Subject**: `cim.person.v1.command.employment.create`
```json
{
  "id": "employment-uuid",
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "organization_id": "org-456",
  "position": "Senior Software Engineer",
  "department": "Engineering",
  "manager_id": "manager-uuid",
  "start_date": "2025-01-15",
  "employment_type": "full_time",
  "salary": {
    "amount": 120000,
    "currency": "USD",
    "frequency": "annually"
  }
}
```

#### Update Employment Status
**Subject**: `cim.person.v1.command.employment.update-status`
```json
{
  "employment_id": "employment-uuid",
  "status": "active",
  "effective_date": "2025-01-15T00:00:00Z",
  "reason": "promotion",
  "updated_by": "hr-system"
}
```

#### Terminate Employment
**Subject**: `cim.person.v1.command.employment.terminate`
```json
{
  "employment_id": "employment-uuid",
  "termination_date": "2025-01-31",
  "reason": "resignation",
  "final_work_day": "2025-01-29",
  "exit_interview_completed": false,
  "terminated_by": "hr-system"
}
```

### Skills Commands

#### Add Skill
**Subject**: `cim.person.v1.command.skills.add`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "skill": {
    "name": "Rust Programming",
    "category": "programming_language",
    "proficiency_level": "expert",
    "years_experience": 5,
    "certifications": ["rust_certified_developer"],
    "verified": true
  },
  "added_by": "skills-assessment-service"
}
```

#### Certify Skill
**Subject**: `cim.person.v1.command.skills.certify`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "skill_name": "Rust Programming",
  "certification": {
    "name": "Rust Certified Developer",
    "issuer": "Rust Foundation",
    "issued_date": "2025-01-15",
    "expiry_date": "2027-01-15",
    "certificate_id": "RUST-2025-12345"
  },
  "certified_by": "certification-service"
}
```

### Network Commands

#### Add Connection
**Subject**: `cim.person.v1.command.network.add-connection`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "connected_person_id": "connected-person-uuid",
  "relationship_type": "colleague",
  "context": "same_department",
  "connection_strength": "strong",
  "established_date": "2025-01-15T10:30:00Z"
}
```

#### Update Relationship
**Subject**: `cim.person.v1.command.network.update-relationship`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "connected_person_id": "connected-person-uuid",
  "relationship_updates": {
    "relationship_type": "mentor",
    "connection_strength": "very_strong",
    "notes": "Became mentor after promotion"
  },
  "updated_by": "network-service"
}
```

## Event Messages

Events are published to NATS subjects following the same envelope pattern as commands.

### Person Events

#### Person Created
**Subject**: `cim.person.v1.event.person.created`
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "given_name": "John",
  "family_name": "Doe",
  "display_name": "John Doe",
  "email": "john.doe@example.com",
  "organization_id": "org-456",
  "created_at": "2025-01-15T10:30:00Z",
  "created_by": "hr-system"
}
```

#### Person Profile Updated
**Subject**: `cim.person.v1.event.person.updated`
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "updated_fields": ["display_name", "phone", "location"],
  "previous_values": {
    "display_name": "John Doe",
    "phone": null
  },
  "current_values": {
    "display_name": "John A. Doe",
    "phone": "+1-555-123-4567"
  },
  "updated_at": "2025-01-15T10:35:00Z",
  "updated_by": "user-789"
}
```

#### Person Archived
**Subject**: `cim.person.v1.event.person.archived`
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "reason": "employee_departure",
  "effective_date": "2025-01-31T23:59:59Z",
  "retention_policy": "7_years",
  "archived_at": "2025-01-15T10:40:00Z",
  "archived_by": "hr-system"
}
```

### Identity Events

#### Identity Created
**Subject**: `cim.person.v1.event.identity.created`
```json
{
  "identity_id": "identity-uuid",
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "identity_type": "employee_id",
  "identifier": "EMP-12345",
  "issuer": "hr_system",
  "verification_level": "pending",
  "created_at": "2025-01-15T10:30:00Z"
}
```

#### Identity Verified
**Subject**: `cim.person.v1.event.identity.verified`
```json
{
  "identity_id": "identity-uuid",
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "verification_method": "document_check",
  "verification_level": "verified",
  "verified_at": "2025-01-15T10:35:00Z",
  "verified_by": "identity-service"
}
```

### Employment Events

#### Employment Created
**Subject**: `cim.person.v1.event.employment.created`
```json
{
  "employment_id": "employment-uuid",
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "organization_id": "org-456",
  "position": "Senior Software Engineer",
  "department": "Engineering",
  "start_date": "2025-01-15",
  "employment_type": "full_time",
  "created_at": "2025-01-15T10:30:00Z"
}
```

#### Employment Status Updated
**Subject**: `cim.person.v1.event.employment.status-updated`
```json
{
  "employment_id": "employment-uuid",
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "previous_status": "probation",
  "current_status": "active",
  "effective_date": "2025-01-15T00:00:00Z",
  "reason": "probation_completed",
  "updated_at": "2025-01-15T10:35:00Z"
}
```

#### Employment Terminated
**Subject**: `cim.person.v1.event.employment.terminated`
```json
{
  "employment_id": "employment-uuid",
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "termination_date": "2025-01-31",
  "reason": "resignation",
  "final_work_day": "2025-01-29",
  "terminated_at": "2025-01-15T10:40:00Z"
}
```

### Skills Events

#### Skill Added
**Subject**: `cim.person.v1.event.skills.added`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "skill": {
    "name": "Rust Programming",
    "category": "programming_language",
    "proficiency_level": "expert",
    "years_experience": 5
  },
  "added_at": "2025-01-15T10:30:00Z",
  "added_by": "skills-assessment-service"
}
```

#### Skill Certified
**Subject**: `cim.person.v1.event.skills.certified`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "skill_name": "Rust Programming",
  "certification": {
    "name": "Rust Certified Developer",
    "issuer": "Rust Foundation",
    "issued_date": "2025-01-15",
    "certificate_id": "RUST-2025-12345"
  },
  "certified_at": "2025-01-15T10:35:00Z"
}
```

### Network Events

#### Connection Established
**Subject**: `cim.person.v1.event.network.connection-established`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "connected_person_id": "connected-person-uuid",
  "relationship_type": "colleague",
  "context": "same_department",
  "connection_strength": "strong",
  "established_at": "2025-01-15T10:30:00Z"
}
```

#### Relationship Updated
**Subject**: `cim.person.v1.event.network.relationship-updated`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "connected_person_id": "connected-person-uuid",
  "previous_relationship": "colleague",
  "current_relationship": "mentor",
  "updated_at": "2025-01-15T10:35:00Z"
}
```

## Query Messages

Query messages use request-reply pattern over NATS.

### Get Person
**Subject**: `cim.person.v1.query.person.get`
**Request**:
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef"
}
```
**Response**:
```json
{
  "id": "01234567-89ab-cdef-0123-456789abcdef",
  "given_name": "John",
  "family_name": "Doe",
  "display_name": "John Doe",
  "email": "john.doe@example.com",
  "phone": "+1-555-123-4567",
  "date_of_birth": "1985-06-15",
  "location": {
    "city": "San Francisco",
    "state": "CA",
    "country": "US"
  },
  "organization_id": "org-456",
  "status": "active",
  "created_at": "2025-01-15T10:30:00Z",
  "updated_at": "2025-01-15T10:35:00Z"
}
```

### Search Persons
**Subject**: `cim.person.v1.query.person.search`
**Request**:
```json
{
  "query": {
    "organization_id": "org-456",
    "department": "Engineering",
    "skills": ["Rust", "Python"],
    "status": "active"
  },
  "limit": 50,
  "offset": 0,
  "sort": "display_name_asc"
}
```

### Get Employment History
**Subject**: `cim.person.v1.query.employment.history`
**Request**:
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "include_terminated": true
}
```

### Get Skills Profile
**Subject**: `cim.person.v1.query.skills.profile`
**Request**:
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "include_certifications": true,
  "skill_categories": ["programming_language", "framework", "tool"]
}
```

### Get Network Connections
**Subject**: `cim.person.v1.query.network.connections`
**Request**:
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "relationship_types": ["colleague", "mentor", "direct_report"],
  "depth": 2
}
```

## Message Patterns

### Request-Reply Pattern
For queries and synchronous operations:

```javascript
// JavaScript example using NATS.js
const msg = await nc.request(
  'cim.person.v1.query.person.get',
  JSON.stringify({
    correlation_id: uuidv4(),
    causation_id: uuidv4(),
    message_id: uuidv4(),
    timestamp: new Date().toISOString(),
    version: '1.0',
    payload: { id: personId }
  }),
  { timeout: 30000 }
);
```

### Publish-Subscribe Pattern
For events and async operations:

```javascript
// Publish a command
await nc.publish(
  'cim.person.v1.command.person.create',
  JSON.stringify({
    correlation_id: uuidv4(),
    causation_id: uuidv4(), 
    message_id: uuidv4(),
    timestamp: new Date().toISOString(),
    version: '1.0',
    payload: {
      id: personId,
      given_name: 'John',
      family_name: 'Doe',
      email: 'john.doe@example.com'
    }
  })
);

// Subscribe to events
const sub = nc.subscribe('cim.person.v1.event.person.created');
for await (const msg of sub) {
  const envelope = JSON.parse(msg.data);
  console.log('Person created:', envelope.payload);
}
```

### Workflow Processing Pattern
For complex person lifecycle workflows:

```javascript
// Subscribe to person lifecycle events with queue groups
const sub = nc.subscribe(
  'cim.person.v1.event.person.*',
  { queue: 'person-processors' }
);

for await (const msg of sub) {
  const envelope = JSON.parse(msg.data);
  
  switch (msg.subject) {
    case 'cim.person.v1.event.person.created':
      await startOnboardingWorkflow(envelope.payload);
      break;
    case 'cim.person.v1.event.employment.terminated':
      await startOffboardingWorkflow(envelope.payload);
      break;
    case 'cim.person.v1.event.skills.certified':
      await updateSkillsProfile(envelope.payload);
      break;
  }
}
```

## Error Handling

### Error Events

Errors are communicated through NATS error events.

#### Identity Verification Failed
**Subject**: `cim.person.v1.event.identity.verification-failed`
```json
{
  "identity_id": "identity-uuid",
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "error": {
    "code": "VERIFICATION_FAILED",
    "message": "Document validation failed: expired government ID",
    "category": "verification",
    "retry_after_seconds": 3600,
    "details": {
      "document_type": "government_id",
      "document_number": "DL123456789",
      "expiry_date": "2023-06-15",
      "current_date": "2025-01-15"
    }
  },
  "failed_at": "2025-01-15T10:35:00Z"
}
```

#### Command Rejected
**Subject**: `cim.person.v1.event.command.rejected`
```json
{
  "original_subject": "cim.person.v1.command.person.create",
  "original_message_id": "22222222-3333-4444-5555-666666666666",
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid email format: 'not-an-email'",
    "category": "validation",
    "field": "email",
    "expected_format": "valid email address"
  },
  "rejected_at": "2025-01-15T10:30:05Z"
}
```

#### Privacy Violation
**Subject**: `cim.person.v1.event.privacy.violation-detected`
```json
{
  "person_id": "01234567-89ab-cdef-0123-456789abcdef",
  "operation": "data_export",
  "error": {
    "code": "PRIVACY_VIOLATION",
    "message": "Attempted export of restricted personal data without consent",
    "category": "privacy",
    "regulation": "GDPR",
    "required_permissions": ["data_export_consent", "admin_override"]
  },
  "detected_at": "2025-01-15T10:40:00Z"
}
```

### Error Codes

Standard error codes used across the Person Domain:

- `VALIDATION_ERROR` - Input validation failed
- `NOT_FOUND` - Requested person/resource not found
- `PERMISSION_DENIED` - Insufficient permissions
- `DUPLICATE_IDENTITY` - Identity already exists
- `VERIFICATION_FAILED` - Identity verification failed
- `PRIVACY_VIOLATION` - Privacy policy violation
- `EMPLOYMENT_CONFLICT` - Employment record conflict
- `SKILLS_VALIDATION_ERROR` - Skills certification validation failed
- `NETWORK_RELATIONSHIP_ERROR` - Invalid relationship configuration
- `GDPR_COMPLIANCE_ERROR` - GDPR compliance violation

### Recovery Patterns

Services implement automatic recovery through NATS:

1. **Retry with Backoff**: Resubmit commands with exponential delay
2. **Manual Review Queue**: Failed identity verifications to manual review
3. **Workflow Compensation**: Rollback complex multi-step operations
4. **Dead Letter Queue**: Failed messages to `cim.person.v1.dlq.*`

## Subject Algebra

The Person Domain implements a formal Subject Algebra that provides mathematical foundations for person management operations. See [Person Subject Algebra](../algebra/README.md) for complete mathematical definitions.

### Subject Structure
```
person.{domain}.{version}.{operation}.{entity}.{aspect}.{qualifier}
```

**Formal Grammar**:
```bnf
<person-subject> ::= "cim.person." <version> "." <operation> "." <entity> "." <aspect> ["." <qualifier>]

<version>    ::= "v1" | "v2" | "v3"
<operation>  ::= "command" | "event" | "query" | "workflow"
<entity>     ::= "person" | "identity" | "employment" | "skills" | "network" | "contact" | "preferences" | "demographics"
<aspect>     ::= <person-aspect> | <identity-aspect> | <employment-aspect> | <skills-aspect> | <network-aspect>

<person-aspect>     ::= "create" | "update" | "archive" | "restore" | "merge" | "split"
<identity-aspect>   ::= "create" | "verify" | "link" | "unlink" | "validate"
<employment-aspect> ::= "create" | "update" | "transfer" | "terminate" | "reactivate"
<skills-aspect>     ::= "add" | "update" | "certify" | "assess" | "endorse" | "expire"
<network-aspect>    ::= "connect" | "disconnect" | "strengthen" | "weaken" | "analyze"
```

### Core Subjects

#### Person Lifecycle Operations
```
# Basic person lifecycle
cim.person.v1.command.person.create
cim.person.v1.command.person.update
cim.person.v1.command.person.archive
cim.person.v1.command.person.restore

# With qualifiers
cim.person.v1.command.person.create.source.hr_system
cim.person.v1.command.person.update.privacy.gdpr_compliant
cim.person.v1.command.person.archive.retention.7_years
```

#### Identity Management
```
# Identity operations
cim.person.v1.command.identity.create.type.employee_id
cim.person.v1.command.identity.verify.method.document_check
cim.person.v1.command.identity.link.provider.sso
cim.person.v1.command.identity.validate.level.high_assurance

# Identity events
cim.person.v1.event.identity.verified.confidence.high
cim.person.v1.event.identity.expired.grace_period.30_days
```

#### Employment Operations
```
# Employment management
cim.person.v1.command.employment.create.type.full_time
cim.person.v1.command.employment.update.status.active
cim.person.v1.command.employment.transfer.department.engineering
cim.person.v1.command.employment.terminate.reason.resignation

# Employment queries
cim.person.v1.query.employment.history.include.terminated
cim.person.v1.query.employment.search.department.engineering
```

#### Skills Management
```
# Skills operations
cim.person.v1.command.skills.add.category.programming_language
cim.person.v1.command.skills.certify.issuer.official
cim.person.v1.command.skills.assess.type.technical_interview
cim.person.v1.command.skills.endorse.source.peer_review

# Skills queries
cim.person.v1.query.skills.search.proficiency.expert
cim.person.v1.query.skills.analyze.gap.team_requirements
```

#### Network Operations
```
# Network management
cim.person.v1.command.network.connect.type.colleague
cim.person.v1.command.network.strengthen.reason.collaboration
cim.person.v1.command.network.analyze.centrality.betweenness
cim.person.v1.command.network.discover.recommendations.mutual_connections

# Network events
cim.person.v1.event.network.connection.established.strength.strong
cim.person.v1.event.network.influence.increased.metric.pagerank
```

### Algebraic Compositions

Sequential processing (⊕):
```
cim.person.v1.event.person.created
  → cim.person.v1.command.identity.create
  → cim.person.v1.event.identity.created
  → cim.person.v1.command.identity.verify
```

Parallel processing (⊗):
```
cim.person.v1.event.person.created
  → cim.person.v1.command.employment.create
  → cim.person.v1.command.skills.initialize
  → cim.person.v1.command.network.discover_connections
```

Conditional transformation (→[P]):
```
cim.person.v1.event.employment.created
  → cim.person.v1.query.person.get_location
  → cim.person.v1.command.network.discover_local_colleagues
      [if same_office_location]
```

### Wildcard Subscriptions

For service implementations:

```javascript
// Process all person commands
nc.subscribe('cim.person.v1.command.*', { queue: 'person-commands' })

// Listen to all identity events
nc.subscribe('cim.person.v1.event.identity.*')

// Handle all skills operations
nc.subscribe('cim.person.v1.*.skills.*', { queue: 'skills-service' })

// Monitor all privacy-related events
nc.subscribe('cim.person.v1.event.*.privacy.*')
```

### Subject Versioning

Version management through subject versioning:

- `v1` - Current stable API
- `v2` - Next version (when available)
- `v1-deprecated` - Deprecated but supported
- `v1-sunset` - Final deprecation notice

Migration example:
```javascript
// Support both versions during transition
nc.subscribe('cim.person.v1.command.person.create', handleV1Create);
nc.subscribe('cim.person.v2.command.person.create', handleV2Create);
```

## Integration Examples

### Basic Publisher (Client)
```javascript
import { connect } from 'nats';

const nc = await connect({ servers: 'nats://localhost:4222' });

// Create person
await nc.publish(
  'cim.person.v1.command.person.create',
  JSON.stringify({
    correlation_id: crypto.randomUUID(),
    causation_id: crypto.randomUUID(),
    message_id: crypto.randomUUID(),
    timestamp: new Date().toISOString(),
    version: '1.0',
    payload: {
      id: crypto.randomUUID(),
      given_name: 'John',
      family_name: 'Doe',
      email: 'john.doe@example.com',
      organization_id: 'org-456'
    }
  })
);
```

### Event Processor (Service)
```javascript
// Subscribe to person creation events
const sub = nc.subscribe('cim.person.v1.event.person.created');
for await (const msg of sub) {
  const envelope = JSON.parse(msg.data);
  
  // Start onboarding workflow
  await nc.publish(
    'cim.person.v1.command.employment.create',
    JSON.stringify({
      correlation_id: envelope.correlation_id, // Chain correlation
      causation_id: envelope.message_id,      // This event caused employment creation
      message_id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      version: '1.0',
      payload: {
        person_id: envelope.payload.id,
        organization_id: envelope.payload.organization_id,
        position: 'New Employee',
        department: 'Onboarding',
        start_date: new Date().toISOString().split('T')[0]
      }
    })
  );
}
```

### Privacy Compliance Handler
```javascript
// Handle GDPR data export requests
const sub = nc.subscribe('cim.person.v1.command.person.export-data');
for await (const msg of sub) {
  const envelope = JSON.parse(msg.data);
  
  try {
    // Validate consent
    const consent = await validateGDPRConsent(envelope.payload.person_id);
    if (!consent.valid) {
      throw new Error('GDPR consent not provided');
    }
    
    // Export person data
    const personData = await exportPersonData(envelope.payload.person_id);
    
    // Publish success event
    await nc.publish(
      'cim.person.v1.event.person.data-exported',
      JSON.stringify({
        correlation_id: envelope.correlation_id,
        causation_id: envelope.message_id,
        message_id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        version: '1.0',
        payload: {
          person_id: envelope.payload.person_id,
          export_size_bytes: JSON.stringify(personData).length,
          export_format: 'json',
          exported_at: new Date().toISOString()
        }
      })
    );
  } catch (error) {
    // Publish privacy violation event
    await nc.publish(
      'cim.person.v1.event.privacy.violation-detected',
      JSON.stringify({
        correlation_id: envelope.correlation_id,
        causation_id: envelope.message_id,
        message_id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        version: '1.0',
        payload: {
          person_id: envelope.payload.person_id,
          operation: 'data_export',
          error: {
            code: 'PRIVACY_VIOLATION',
            message: error.message,
            category: 'privacy',
            regulation: 'GDPR'
          }
        }
      })
    );
  }
}
```