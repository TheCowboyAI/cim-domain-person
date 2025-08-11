<!--
Copyright 2025 Cowboy AI, LLC.
SPDX-License-Identifier: MIT
-->

# CIM Person Domain - Cross Reference Index

## Mathematical Foundations → Implementation

### Person Subject Algebra → NATS API

| Algebraic Concept | NATS Subject Pattern | Documentation |
|-------------------|---------------------|---------------|
| Sequential Composition (⊕) | `cim.person.v1.command.person.create` → `cim.person.v1.command.identity.create` | [Algebra](algebra/README.md#sequential-composition-), [API](api/README.md#command-messages) |
| Parallel Composition (⊗) | `cim.person.v1.command.employment.create` ∥ `cim.person.v1.command.skills.initialize` | [Algebra](algebra/README.md#parallel-composition-), [API](api/README.md#message-patterns) |
| Conditional Transformation (→) | `cim.person.v1.query.person.get_location` → `cim.person.v1.command.network.discover_local_colleagues` | [Algebra](algebra/README.md#conditional-transformation-), [API](api/README.md#subject-algebra) |

### Type System → Configuration

| Type Category | Configuration Section | Documentation |
|---------------|----------------------|---------------|
| PersonData → VerifiedPerson | `person.identity.verification_levels` | [Algebra](algebra/README.md#type-system-and-safety), [Config](configuration/README.md#identity-verification-settings) |
| EmploymentRecord | `employment.status_transitions` | [Algebra](algebra/README.md#person-type-hierarchy), [Config](configuration/README.md#employment-configuration) |
| SkillsProfile | `skills.certification_authorities` | [Algebra](algebra/README.md#component-types), [Config](configuration/README.md#skills-management) |

## Processing Workflows → Deployment

### Pipeline Types → Service Architecture

| Workflow | Service Components | Documentation |
|----------|-------------------|---------------|
| Person Onboarding | `person-command-service`, `identity-service`, `employment-service` | [Algebra](algebra/README.md#person-onboarding-pipeline), [Deployment](deployment/README.md#person-services) |
| Identity Verification | `identity-service`, `document-verification-service`, `biometric-service` | [Algebra](algebra/README.md#identity-verification-pipeline), [Deployment](deployment/README.md#identity-verification-deployment) |
| Skills Certification | `skills-service`, `certification-authority-service`, `assessment-service` | [Algebra](algebra/README.md#skills-certification-pipeline), [Deployment](deployment/README.md#skills-management-deployment) |
| Employment Lifecycle | `employment-service`, `hr-integration-service`, `payroll-service` | [Algebra](algebra/README.md#employment-lifecycle-pipeline), [Deployment](deployment/README.md#employment-services) |
| Privacy Compliance | `privacy-service`, `gdpr-compliance-service`, `audit-service` | [Algebra](algebra/README.md#privacy-compliance-pipeline), [Deployment](deployment/README.md#privacy-compliance-deployment) |

## API Patterns → Configuration Settings

### Subject Categories → Environment Variables

| Subject Pattern | Environment Variable | Documentation |
|-----------------|---------------------|---------------|
| `cim.person.v1.command.person.*` | `PERSON_MAX_CONCURRENT_OPERATIONS` | [API](api/README.md#person-commands), [Config](configuration/README.md#person-management-settings) |
| `cim.person.v1.command.identity.verify.*` | `IDENTITY_VERIFICATION_TIMEOUT` | [API](api/README.md#identity-commands), [Config](configuration/README.md#identity-verification-settings) |
| `cim.person.v1.command.employment.*` | `EMPLOYMENT_NOTIFICATION_DELAY` | [API](api/README.md#employment-commands), [Config](configuration/README.md#employment-configuration) |
| `cim.person.v1.command.skills.certify.*` | `SKILLS_CERTIFICATION_VALIDATION_LEVEL` | [API](api/README.md#skills-commands), [Config](configuration/README.md#skills-management) |
| `cim.person.v1.command.network.*` | `NETWORK_ANALYSIS_BATCH_SIZE` | [API](api/README.md#network-commands), [Config](configuration/README.md#network-analysis-settings) |

## Error Handling Cross-Reference

### Algebraic Errors → NATS Error Events

| Algebraic Violation | NATS Error Subject | Error Code | Documentation |
|---------------------|-------------------|------------|---------------|
| Type Safety Violation | `cim.person.v1.event.command.rejected` | `VALIDATION_ERROR` | [Algebra](algebra/README.md#type-safety-rules), [API](api/README.md#error-handling) |
| Identity Verification Failure | `cim.person.v1.event.identity.verification-failed` | `VERIFICATION_FAILED` | [Algebra](algebra/README.md#identity-verification-rules), [API](api/README.md#error-events) |
| Privacy Policy Violation | `cim.person.v1.event.privacy.violation-detected` | `PRIVACY_VIOLATION` | [Algebra](algebra/README.md#privacy-constraints), [API](api/README.md#privacy-violation) |
| Employment Conflict | `cim.person.v1.event.employment.conflict-detected` | `EMPLOYMENT_CONFLICT` | [Algebra](algebra/README.md#employment-rules), [API](api/README.md#employment-errors) |
| Skills Validation Error | `cim.person.v1.event.skills.validation-failed` | `SKILLS_VALIDATION_ERROR` | [Algebra](algebra/README.md#skills-validation-rules), [API](api/README.md#skills-errors) |

## Performance & Scaling Cross-Reference

### Algebraic Optimizations → Deployment Strategies

| Optimization Rule | Deployment Strategy | Documentation |
|------------------|-------------------|---------------|
| Associativity `(A ⊕ B) ⊕ C = A ⊕ (B ⊕ C)` | Workflow Stage Grouping | [Algebra](algebra/README.md#fusion-laws), [Deployment](deployment/README.md#scaling-considerations) |
| Commutativity `A ⊗ B = B ⊗ A` | Load Balancing Strategies | [Algebra](algebra/README.md#parallel-composition-), [Deployment](deployment/README.md#load-balancing-strategies) |
| Distributivity `A ⊕ (B ⊗ C) = (A ⊕ B) ⊗ (A ⊕ C)` | Resource Optimization | [Algebra](algebra/README.md#distributive-laws), [Performance](performance/README.md#optimization-settings) |
| Dead Code Elimination | Service Auto-scaling | [Algebra](algebra/README.md#dead-code-elimination), [Deployment](deployment/README.md#horizontal-pod-autoscaling) |

## Privacy & Compliance Cross-Reference

### GDPR Requirements → Implementation

| GDPR Requirement | Implementation | Documentation |
|-------------------|----------------|---------------|
| Right to Access | `cim.person.v1.command.person.export-data` | [Privacy](privacy/README.md#right-to-access), [API](api/README.md#privacy-compliance-handler) |
| Right to Rectification | `cim.person.v1.command.person.update` with audit trail | [Privacy](privacy/README.md#right-to-rectification), [API](api/README.md#person-commands) |
| Right to Erasure | `cim.person.v1.command.person.delete-personal-data` | [Privacy](privacy/README.md#right-to-erasure), [API](api/README.md#data-deletion) |
| Data Portability | `cim.person.v1.command.person.export-structured-data` | [Privacy](privacy/README.md#data-portability), [API](api/README.md#data-export) |
| Consent Management | `cim.person.v1.command.person.update-consent` | [Privacy](privacy/README.md#consent-management), [API](api/README.md#consent-commands) |

## Documentation Navigation Paths

### For Mathematical Understanding
1. [Person Subject Algebra](algebra/README.md) - Start here for mathematical foundations
2. [API Reference](api/README.md#subject-algebra) - See algebra in practice  
3. [Configuration](configuration/README.md) - Understand type-safe configuration
4. [Deployment](deployment/README.md) - Scale algebraic operations

### For Implementation
1. [API Reference](api/README.md) - NATS message patterns
2. [Configuration](configuration/README.md) - Service setup
3. [Deployment](deployment/README.md) - Production deployment
4. [Person Subject Algebra](algebra/README.md) - Deep mathematical understanding

### For Operations
1. [Deployment Guide](deployment/README.md) - Infrastructure setup
2. [Configuration Reference](configuration/README.md) - Tuning parameters
3. [Troubleshooting](troubleshooting/README.md) - Issue resolution
4. [Performance Guide](performance/README.md) - Optimization strategies

### For Security & Privacy
1. [Privacy Best Practices](privacy/README.md) - GDPR compliance model
2. [Security Best Practices](security/README.md) - Security model
3. [Configuration](configuration/README.md#security-configuration) - Security settings
4. [Deployment](deployment/README.md#network-security) - Network security
5. [API](api/README.md#error-handling) - Secure error handling

### For HR & People Operations
1. [Employment Workflows](workflows/employment/README.md) - Employment lifecycle
2. [Onboarding Guide](workflows/onboarding/README.md) - Employee onboarding
3. [Skills Management](workflows/skills/README.md) - Skills and certification tracking
4. [Privacy Workflows](workflows/privacy/README.md) - Data privacy workflows

## Workflow Cross-Reference

### Business Process → Technical Implementation

| Business Process | Workflow Definition | NATS Subjects | Documentation |
|-------------------|---------------------|---------------|---------------|
| Employee Onboarding | `OnboardingWorkflow` | `cim.person.v1.workflow.onboarding.*` | [Workflows](workflows/onboarding/README.md), [API](api/README.md#workflow-processing-pattern) |
| Employment Status Change | `EmploymentLifecycleWorkflow` | `cim.person.v1.workflow.employment.*` | [Workflows](workflows/employment/README.md), [Algebra](algebra/README.md#employment-lifecycle-pipeline) |
| Skills Certification | `SkillsCertificationWorkflow` | `cim.person.v1.workflow.skills.*` | [Workflows](workflows/skills/README.md), [API](api/README.md#skills-commands) |
| Privacy Data Request | `PrivacyComplianceWorkflow` | `cim.person.v1.workflow.privacy.*` | [Workflows](workflows/privacy/README.md), [Privacy](privacy/README.md) |
| Identity Verification | `IdentityVerificationWorkflow` | `cim.person.v1.workflow.identity.*` | [Workflows](workflows/identity/README.md), [API](api/README.md#identity-commands) |

### Workflow States → Event Subjects

| Workflow State | Event Subject | Next Possible States | Documentation |
|----------------|---------------|---------------------|---------------|
| `onboarding.identity_created` | `cim.person.v1.event.workflow.onboarding.identity-created` | `document_verification`, `biometric_enrollment` | [Workflows](workflows/onboarding/README.md#identity-phase) |
| `employment.status_pending` | `cim.person.v1.event.workflow.employment.status-pending` | `approved`, `rejected`, `needs_review` | [Workflows](workflows/employment/README.md#approval-process) |
| `skills.assessment_started` | `cim.person.v1.event.workflow.skills.assessment-started` | `completed`, `failed`, `expired` | [Workflows](workflows/skills/README.md#assessment-phase) |
| `privacy.request_received` | `cim.person.v1.event.workflow.privacy.request-received` | `validated`, `rejected`, `escalated` | [Workflows](workflows/privacy/README.md#request-processing) |
| `identity.verification_pending` | `cim.person.v1.event.workflow.identity.verification-pending` | `verified`, `failed`, `manual_review` | [Workflows](workflows/identity/README.md#verification-states) |

## Service Dependency Matrix

### Core Services → Dependencies

| Service | Required Services | Optional Services | Documentation |
|---------|-------------------|-------------------|---------------|
| `person-command-service` | `nats-server`, `event-store` | `notification-service` | [Services](services/person-command/README.md) |
| `identity-service` | `person-command-service`, `document-store` | `biometric-service`, `ml-verification` | [Services](services/identity/README.md) |
| `employment-service` | `person-command-service`, `organization-service` | `payroll-service`, `hr-integration` | [Services](services/employment/README.md) |
| `skills-service` | `person-command-service` | `assessment-service`, `certification-authorities` | [Services](services/skills/README.md) |
| `network-service` | `person-command-service`, `graph-database` | `recommendation-engine`, `analytics-service` | [Services](services/network/README.md) |
| `privacy-service` | `person-command-service`, `audit-service` | `legal-review-service`, `compliance-dashboard` | [Services](services/privacy/README.md) |

## Version Compatibility Matrix

| Documentation Version | Algebra Version | API Version | Config Version | Workflow Version |
|-----------------------|-----------------|-------------|----------------|------------------|
| v1.0.0 | v1.0.0 | v1 | v1.0.0 | v1.0.0 |
| v1.1.0 | v1.0.0 | v1 | v1.1.0 | v1.1.0 |
| v2.0.0 | v2.0.0 | v2 | v2.0.0 | v2.0.0 |

## Quick Reference Links

### Essential Concepts
- [7-tuple Algebraic Definition](algebra/README.md#formal-definition)
- [NATS Subject Grammar](api/README.md#subject-structure)  
- [Type Safety Rules](algebra/README.md#type-safety-rules)
- [Person Workflows](algebra/README.md#person-processing-workflows)

### Common Operations
- [Sequential Processing](algebra/README.md#sequential-composition-)
- [Parallel Processing](algebra/README.md#parallel-composition-)
- [Conditional Processing](algebra/README.md#conditional-transformation-)
- [Error Recovery](api/README.md#recovery-patterns)

### Privacy & Compliance
- [GDPR Compliance](privacy/README.md#gdpr-compliance)
- [Data Protection](privacy/README.md#data-protection)
- [Consent Management](privacy/README.md#consent-management)
- [Audit Trail](privacy/README.md#audit-trail)

### Production Checklist
- [ ] [Mathematical Foundation Understood](algebra/README.md)
- [ ] [NATS Infrastructure Deployed](deployment/README.md#nats-configuration)
- [ ] [Services Configured](configuration/README.md)
- [ ] [Identity Verification Setup](deployment/README.md#identity-verification-services)
- [ ] [Privacy Compliance Enabled](deployment/README.md#privacy-compliance)
- [ ] [Monitoring Enabled](deployment/README.md#monitoring--observability)
- [ ] [Security Hardened](security/README.md)
- [ ] [GDPR Compliance Verified](privacy/README.md#compliance-verification)
- [ ] [Backup Strategy Implemented](deployment/README.md#backup--recovery)
- [ ] [Disaster Recovery Tested](deployment/README.md#disaster-recovery)

## Integration Patterns

### Domain Integration → Cross-Domain Events

| Person Domain Event | Cross-Domain Integration | Target Domain | Documentation |
|---------------------|-------------------------|---------------|---------------|
| `person.created` | Document profile creation | `cim-domain-document` | [Integration](integration/document-domain.md) |
| `skills.certified` | Learning record update | `cim-domain-learning` | [Integration](integration/learning-domain.md) |
| `employment.terminated` | Access revocation | `cim-domain-security` | [Integration](integration/security-domain.md) |
| `network.connection.established` | Communication channel setup | `cim-domain-communication` | [Integration](integration/communication-domain.md) |
| `person.location.updated` | Location-based services | `cim-domain-location` | [Integration](integration/location-domain.md) |

### Message Correlation Patterns

| Correlation Type | Implementation | Use Case | Documentation |
|------------------|----------------|----------|---------------|
| Causal Chain | `causation_id` links | Audit trail for complex workflows | [Algebra](algebra/README.md#message-identity-tracking) |
| Workflow Instance | `correlation_id` groups | Multi-step business processes | [Workflows](workflows/README.md#correlation-patterns) |
| Cross-Domain | Domain-specific correlation | Inter-domain data consistency | [Integration](integration/README.md#correlation-strategies) |
| Temporal Correlation | Time-window grouping | Event stream analysis | [Analytics](analytics/README.md#temporal-analysis) |

## Performance Optimization Matrix

### Algebraic Laws → Performance Gains

| Algebraic Law | Performance Optimization | Measurable Benefit | Documentation |
|---------------|-------------------------|-------------------|---------------|
| Associativity | Pipeline stage reordering | 15-30% throughput increase | [Performance](performance/README.md#associativity-optimization) |
| Commutativity | Parallel execution | 2x-4x performance on multi-core | [Performance](performance/README.md#parallel-optimization) |
| Distributivity | Resource sharing | 20-40% memory reduction | [Performance](performance/README.md#resource-optimization) |
| Idempotency | Safe retry mechanisms | 99.9% reliability improvement | [Performance](performance/README.md#reliability-optimization) |