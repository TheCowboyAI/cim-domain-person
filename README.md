# CIM Person Domain

The Person domain provides a comprehensive, component-based system for representing people in various contexts, with a special focus on Customer Relationship Management (CRM) functionality.

## Overview

The Person domain uses a flexible component-based architecture that allows you to compose person entities with different sets of components to represent various concepts like Employee, Customer, Partner, or any custom concept your business requires.

## Core Components

### Name Components

- **NameComponent**: Comprehensive name handling supporting:
  - Titles (Dr., Prof., etc.)
  - Honorifics (Mr., Mrs., Ms., Mx., etc.)
  - Multiple given names
  - Multiple middle names (preserving order)
  - Multiple family names (e.g., Spanish naming)
  - Maternal family names
  - Generational suffixes (Jr., Sr., III, etc.)
  - Professional suffixes (MD, PhD, etc.)
  - Preferred names/nicknames
  - Cultural naming conventions (Western, Eastern, etc.)

- **AlternativeNamesComponent**: Track previous names, aliases, professional names, etc.

### Physical Components

- **PhysicalAttributesComponent**: Height, weight, hair color, eye color, etc.
- **DistinguishingMarksComponent**: Scars, tattoos, birthmarks, etc.
- **BiometricComponent**: Privacy-preserving biometric data (hashes only)
- **MedicalIdentityComponent**: Blood type, allergies, emergency medical info

### Social Components

- **RelationshipComponent**: Family, professional, and social relationships
- **SocialMediaComponent**: Social media profiles and metrics
- **InterestsComponent**: Hobbies, interests, and activities

### Behavioral Components (CRM-focused)

- **PreferencesComponent**: Communication, product, content, and privacy preferences
- **BehavioralComponent**: Purchase behavior, engagement patterns, predictive scores
- **SegmentationComponent**: Customer segments, lifecycle stages, value tiers

### Professional Components

- **EmploymentComponent**: Current employment information
- **PositionComponent**: Role and responsibilities
- **SkillsComponent**: Skills, certifications, and education
- **AccessComponent**: Roles, permissions, and access levels

### Contact Components

- **ContactComponent**: Email addresses, phone numbers, physical addresses
- **ExternalIdentifiersComponent**: IDs from external systems (LDAP, OAuth, etc.)

## Usage Examples

### Creating a Customer

```rust
use cim_domain_person::*;

// Create a customer with name, contact, and preferences
let name = NameComponent::simple("Jane".to_string(), "Doe".to_string());

let contact = ContactComponent {
    emails: vec![EmailAddress {
        email: "jane@example.com".to_string(),
        email_type: "personal".to_string(),
        is_primary: true,
        is_verified: true,
    }],
    phones: vec![],
    addresses: vec![],
};

let preferences = PreferencesComponent {
    communication: CommunicationPreferences {
        preferred_channel: ContactChannel::Email,
        frequency_preference: FrequencyPreference::Weekly,
        // ... other preferences
    },
    // ... other preference categories
};

let customer = PersonCompositionService::create_customer(
    name,
    contact,
    preferences,
)?;
```

### Creating an Employee

```rust
let employee = PersonCompositionService::create_employee(
    name,
    employment,
    contact,
)?;

// Add additional components as needed
PersonCompositionService::add_physical_attributes(
    &mut employee,
    physical_attributes,
    "hr_system",
    Some("ID badge photo".to_string()),
)?;
```

### Building Views

Different views can be built from person entities based on their components:

```rust
// Check if person can be viewed as employee
if EmployeeViewBuilder::can_build(&person) {
    let employee_view = EmployeeViewBuilder::build(&person)?;
}

// Build customer view
if CustomerViewBuilder::can_build(&person) {
    let customer_view = CustomerViewBuilder::build(&person)?;
}
```

## Component Composition

The power of this system lies in its flexibility. You can:

1. **Start Simple**: Create a person with just a name
2. **Add Components**: Progressively add components as you learn more
3. **Create Views**: Build different views based on available components
4. **Track Metadata**: Know when and why each component was added

Example of progressive enhancement:

```rust
// Start with basic person
let mut person = PersonCompositionService::create_basic_person(name)?;

// Add components as information becomes available
person.add_component(contact_info, "crm_system", Some("Initial contact"))?;
person.add_component(preferences, "user_portal", Some("User preferences set"))?;
person.add_component(behavioral_data, "analytics", Some("Behavioral analysis"))?;
```

## Design Principles

1. **Component Immutability**: Components are immutable once added. To change, remove and re-add.
2. **Domain Alignment**: Components represent business concepts, not technical constructs.
3. **Progressive Enhancement**: Start simple and add complexity as needed.
4. **Audit Trail**: Track who added what component and when.
5. **Type Safety**: Leverage Rust's type system to prevent component misuse.

## Integration with Other Domains

The Person domain integrates seamlessly with other CIM domains:

- **Organization**: Link people to organizations via employment
- **Location**: Reference physical addresses
- **Workflow**: Assign people to workflow tasks
- **Identity**: Manage authentication and authorization

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

Run the example to see the system in action:

```bash
cargo run --example crm_person_composition
```

## Future Enhancements

- Additional component types based on business needs
- Integration with external CRM systems
- Advanced segmentation algorithms
- Machine learning for predictive scoring
- GDPR compliance tools for data management

## Key Concepts

### Person
An individual identity with:
- **Identity**: Unique identifier and core attributes
- **Profile**: Personal information and preferences
- **Credentials**: Authentication and verification data
- **Memberships**: Organizational affiliations
- **Permissions**: Access rights and capabilities
- **Privacy**: Control over data sharing

### Person Components
- **Basic Information**: Name, contact details
- **Extended Profile**: Biography, skills, interests
- **Authentication**: Credentials and MFA settings
- **Preferences**: System and UI preferences
- **Privacy Settings**: Data visibility controls
- **Activity History**: Audit trail of actions

### Identity Lifecycle
```
Registration → Verification → Active → Updates → Deactivation
      ↓             ↓           ↓         ↓           ↓
   Events       Events      Events    Events      Events
```

## Architecture

### Aggregates

#### Person Aggregate
```rust
pub struct Person {
    pub id: PersonId,
    pub username: Username,
    pub email: Email,
    pub profile: PersonProfile,
    pub status: PersonStatus,
    pub credentials: Credentials,
    pub preferences: UserPreferences,
    pub privacy_settings: PrivacySettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}
```

#### PersonProfile
```rust
pub struct PersonProfile {
    pub display_name: String,
    pub full_name: Option<FullName>,
    pub avatar_url: Option<Url>,
    pub bio: Option<String>,
    pub timezone: String,
    pub locale: String,
    pub contact_info: ContactInfo,
    pub social_links: Vec<SocialLink>,
    pub skills: HashSet<Skill>,
    pub interests: HashSet<Interest>,
}
```

#### Credentials
```rust
pub struct Credentials {
    pub password_hash: Option<PasswordHash>,
    pub mfa_settings: MfaSettings,
    pub api_keys: Vec<ApiKey>,
    pub oauth_tokens: HashMap<Provider, OAuthToken>,
    pub recovery_codes: Vec<RecoveryCode>,
    pub security_questions: Vec<SecurityQuestion>,
}
```

### Commands

#### Identity Management
- `RegisterPerson`: Create new person identity
- `UpdateProfile`: Modify personal information
- `ChangeUsername`: Update username
- `DeactivatePerson`: Disable account
- `ReactivatePerson`: Re-enable account

#### Authentication
- `ChangePassword`: Update password
- `EnableMfa`: Activate multi-factor auth
- `DisableMfa`: Deactivate multi-factor auth
- `GenerateApiKey`: Create API access key
- `RevokeApiKey`: Remove API access key

#### Profile Management
- `UpdateContactInfo`: Modify contact details
- `SetAvatar`: Update profile picture
- `AddSkill`: Add skill to profile
- `RemoveSkill`: Remove skill from profile
- `UpdatePrivacySettings`: Change visibility

#### Membership Operations
- `JoinOrganization`: Add org membership
- `LeaveOrganization`: Remove membership
- `UpdateRole`: Change org role
- `AcceptInvitation`: Join via invite
- `TransferOwnership`: Hand over responsibilities

### Events

All events published to NATS subjects under `person.events.*`:

#### Identity Events
- `PersonRegistered`: New identity created
- `PersonVerified`: Email/phone confirmed
- `PersonUpdated`: Profile modified
- `PersonDeactivated`: Account disabled
- `PersonReactivated`: Account re-enabled

#### Authentication Events
- `PasswordChanged`: Credential updated
- `MfaEnabled`: Two-factor activated
- `MfaDisabled`: Two-factor deactivated
- `LoginSuccessful`: Authentication success
- `LoginFailed`: Authentication failure

#### Profile Events
- `ProfileUpdated`: Information changed
- `AvatarChanged`: Picture updated
- `SkillAdded`: New skill listed
- `SkillRemoved`: Skill delisted
- `PrivacyUpdated`: Settings changed

#### Membership Events
- `OrganizationJoined`: Added to org
- `OrganizationLeft`: Removed from org
- `RoleChanged`: Permissions updated
- `InvitationAccepted`: Joined via invite
- `OwnershipTransferred`: Responsibilities moved

## Usage Examples

### Register a New Person
```rust
use cim_domain_person::{RegisterPerson, PersonProfile, PrivacySettings};

let command = RegisterPerson {
    person_id: PersonId::new(),
    username: Username::try_from("alice.smith")?,
    email: Email::try_from("alice@example.com")?,
    password: PlainPassword::new("secure_password123!"),
    profile: PersonProfile {
        display_name: "Alice Smith".to_string(),
        full_name: Some(FullName {
            first: "Alice".to_string(),
            middle: None,
            last: "Smith".to_string(),
        }),
        timezone: "America/New_York".to_string(),
        locale: "en-US".to_string(),
        ..Default::default()
    },
    privacy_settings: PrivacySettings::default(),
};

nats_client.publish("person.commands.register", &command).await?;
```

### Update Profile Information
```rust
use cim_domain_person::{UpdateProfile, ContactInfo, SocialLink};

let command = UpdateProfile {
    person_id,
    updates: ProfileUpdates {
        display_name: Some("Alice S.".to_string()),
        bio: Some("Software engineer passionate about distributed systems".to_string()),
        contact_info: Some(ContactInfo {
            phone: Some(PhoneNumber::try_from("+1-555-0123")?),
            secondary_email: Some(Email::try_from("alice.work@company.com")?),
            ..Default::default()
        }),
        social_links: Some(vec![
            SocialLink::github("alice-smith"),
            SocialLink::linkedin("alice-smith-dev"),
        ]),
        skills: Some(hashset!["Rust", "Distributed Systems", "Event Sourcing"]),
    },
};

nats_client.publish("person.commands.update_profile", &command).await?;
```

### Enable Multi-Factor Authentication
```rust
use cim_domain_person::{EnableMfa, MfaMethod, TotpSetup};

let command = EnableMfa {
    person_id,
    method: MfaMethod::Totp,
    setup_data: MfaSetupData::Totp(TotpSetup {
        secret: generate_totp_secret(),
        issuer: "CIM Platform".to_string(),
        account_name: username.to_string(),
    }),
    backup_codes: generate_backup_codes(8),
};

nats_client.publish("person.commands.enable_mfa", &command).await?;
```

### Manage Privacy Settings
```rust
use cim_domain_person::{UpdatePrivacySettings, DataVisibility, ConsentSettings};

let command = UpdatePrivacySettings {
    person_id,
    settings: PrivacySettings {
        profile_visibility: DataVisibility::Organization, // Only org members
        email_visibility: DataVisibility::Private,        // Only self
        activity_visibility: DataVisibility::Contacts,    // Friends only
        search_indexing: false,                          // Not searchable
        data_retention: RetentionPeriod::Days(365),      // 1 year
        consent: ConsentSettings {
            marketing_emails: false,
            usage_analytics: true,
            data_sharing: false,
        },
    },
};

nats_client.publish("person.commands.update_privacy", &command).await?;
```

## Integration Points

### NATS Subjects
- Commands: `person.commands.*`
- Events: `person.events.*`
- Queries: `person.queries.*`
- Auth: `person.auth.*`

### Cross-Domain Integration

#### With Identity Domain
- Unified identity management
- Cross-domain authentication
- Permission inheritance
- Session management

#### With Organization Domain
- Membership management
- Role assignments
- Permission delegation
- Organizational profiles

#### With Policy Domain
- Access control enforcement
- Privacy policy compliance
- Consent management
- Audit requirements

#### With Document Domain
- Document ownership
- Access permissions
- Sharing controls
- Activity tracking

### External Integration

#### Authentication Providers
- OAuth2/OIDC providers
- SAML identity providers
- LDAP/Active Directory
- Social login (Google, GitHub, etc.)

#### Communication Services
- Email verification
- SMS verification
- Push notifications
- WebAuthn/FIDO2

## Privacy and Security

### Data Protection
```rust
pub struct PrivacyControls {
    pub encryption: EncryptionSettings,
    pub anonymization: AnonymizationRules,
    pub retention: RetentionPolicies,
    pub audit_trail: AuditSettings,
}
```

### GDPR Compliance
- Right to access (data export)
- Right to rectification (update)
- Right to erasure (delete)
- Right to portability
- Consent management
- Data minimization

### Security Features
- Password complexity rules
- Account lockout policies
- Session management
- API key rotation
- Audit logging
- Anomaly detection

## Authentication Flows

### Password-Based
```
Login Request → Validate Credentials → Check MFA → Create Session → Return Token
       ↓                ↓                 ↓            ↓              ↓
    Events           Events            Events       Events         Events
```

### OAuth2/OIDC
```
Auth Request → Redirect to Provider → Callback → Validate Token → Create Session
      ↓                ↓                 ↓            ↓               ↓
   Events           External          Events       Events          Events
```

### API Key
```
Request with Key → Validate Key → Check Permissions → Process Request
        ↓               ↓                ↓                  ↓
     Events          Events           Events             Events
```

## User Preferences

### System Preferences
```rust
pub struct SystemPreferences {
    pub theme: Theme,
    pub language: LanguageCode,
    pub date_format: DateFormat,
    pub time_format: TimeFormat,
    pub notifications: NotificationSettings,
}
```

### UI Preferences
```rust
pub struct UIPreferences {
    pub dashboard_layout: DashboardConfig,
    pub default_views: HashMap<Context, ViewConfig>,
    pub shortcuts: HashMap<Action, KeyBinding>,
    pub accessibility: AccessibilitySettings,
}
```

## Activity Tracking

### Audit Events
- Login attempts
- Profile changes
- Permission changes
- Data access
- API usage
- Security events

### Analytics
```rust
pub struct UserAnalytics {
    pub login_frequency: TimeSeriesData,
    pub feature_usage: HashMap<Feature, UsageStats>,
    pub api_calls: HashMap<Endpoint, CallStats>,
    pub error_rates: HashMap<ErrorType, Count>,
}
```

## Performance Optimization

### Caching Strategy
- Session cache (Redis)
- Permission cache (LRU)
- Profile cache (CDN)
- Query result cache

### Database Indexes
- Username (unique)
- Email (unique)
- Organization memberships
- Last login time
- Search fields

## Configuration

### Environment Variables
```bash
# Authentication
PERSON_PASSWORD_MIN_LENGTH=12
PERSON_PASSWORD_REQUIRE_SPECIAL=true
PERSON_SESSION_TIMEOUT_MINUTES=30
PERSON_MAX_LOGIN_ATTEMPTS=5

# Privacy
PERSON_DEFAULT_PRIVACY_LEVEL=organization
PERSON_ALLOW_PUBLIC_PROFILES=false
PERSON_GDPR_MODE=true

# Security
PERSON_MFA_REQUIRED=false
PERSON_API_KEY_ROTATION_DAYS=90
PERSON_AUDIT_RETENTION_DAYS=365

# Integration
PERSON_OAUTH_PROVIDERS=google,github
PERSON_LDAP_ENABLED=false
```

## Testing

```bash
# Run all person domain tests
cargo test -p cim-domain-person

# Run specific test categories
cargo test -p cim-domain-person --test authentication
cargo test -p cim-domain-person --test privacy
cargo test -p cim-domain-person --test profile_management
```

### Test Coverage Areas
- Registration flows
- Authentication methods
- Profile updates
- Privacy controls
- Organization membership
- API key management

## Migration Guide

### From Legacy User System
1. Export user data with hashed passwords
2. Map fields to Person aggregate
3. Generate PersonId for each user
4. Import organizational relationships
5. Migrate authentication methods
6. Verify data integrity

### Import Format
```json
{
  "persons": [{
    "external_id": "user-123",
    "username": "alice.smith",
    "email": "alice@example.com",
    "profile": {
      "display_name": "Alice Smith",
      "timezone": "America/New_York",
      "locale": "en-US"
    },
    "memberships": [{
      "organization_id": "org-456",
      "role": "member",
      "joined_at": "2023-01-15T10:00:00Z"
    }]
  }]
}
```

## Best Practices

1. Use strong password policies
2. Implement proper session management
3. Audit all authentication events
4. Respect privacy preferences
5. Minimize data collection
6. Encrypt sensitive data
7. Regular security reviews

## Common Patterns

### User Onboarding
```
Registration → Email Verification → Profile Setup → Organization Join → Tutorial
      ↓               ↓                  ↓               ↓              ↓
   Events          Events            Events          Events         Events
```

### Account Recovery
```
Request Reset → Verify Identity → Send Token → Reset Password → Notify User
      ↓               ↓              ↓              ↓              ↓
   Events          Events         Events         Events         Events
```

## Contributing

1. Follow privacy-by-design principles
2. Implement comprehensive audit trails
3. Test authentication flows thoroughly
4. Document security implications
5. Consider international privacy laws

## License

See the main project LICENSE file. 