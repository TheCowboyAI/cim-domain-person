# CIM Domain Person

Individual identity management and personal information handling for the Composable Information Machine.

## Overview

The Person domain manages all aspects of individual identity within the CIM ecosystem. It handles personal information, authentication credentials, organizational memberships, preferences, and privacy controls. This domain serves as the foundation for user-centric features while maintaining strict privacy and security standards.

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