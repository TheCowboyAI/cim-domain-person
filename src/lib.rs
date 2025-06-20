//! Person/People domain for the Composable Information Machine
//!
//! This crate provides the person domain implementation, including:
//! - Person aggregate with business logic
//! - Commands for person operations
//! - Events representing person state changes
//! - Command and query handlers
//! - Projections for read models
//! - Value objects specific to people

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod value_objects;
pub mod services;

// Re-export main types
pub use aggregate::{Person, PersonId, PersonMarker};
pub use commands::PersonCommand;
pub use events::PersonEvent;
pub use projections::{PersonProjection, EmployeeView, LdapProjection};
pub use queries::PersonQuery;

// Re-export services
pub use services::{
    PersonCompositionService,
    views::{PersonView, ViewType, EmployeeViewBuilder, CustomerViewBuilder, PartnerViewBuilder},
};

// Re-export legacy components
pub use value_objects::{
    IdentityComponent, ContactComponent, EmailAddress, PhoneNumber,
    EmploymentComponent, PositionComponent, SkillsComponent,
    SkillProficiency, Certification, Education, AccessComponent,
    ExternalIdentifiersComponent
};

// Re-export new comprehensive components
pub use value_objects::{
    // Name components
    NameComponent, NameOrder, AlternativeNamesComponent, AlternativeName,
    AlternativeNameType, NamePeriod,
    
    // Physical components
    PhysicalAttributesComponent, Build, VisionCorrection,
    DistinguishingMarksComponent, DistinguishingMark, MarkType,
    BiometricComponent, BiometricHash,
    MedicalIdentityComponent, BloodType,
    
    // Social components
    RelationshipComponent, Relationship, RelationshipType, RelationshipStatus,
    SocialMediaComponent, SocialMediaProfile, SocialPlatform, PrivacySetting,
    SocialMetrics, InterestsComponent, InterestCategory, Interest, InterestProfile,
    
    // Behavioral components
    PreferencesComponent, CommunicationPreferences, ContactChannel, ChannelSettings,
    ContactTimePreference, FrequencyPreference, ProductPreference, ContentPreferences,
    ContentType, ContentFormat, ComplexityLevel, PrivacyPreferences,
    BehavioralComponent, PurchaseBehavior, EngagementPatterns, InteractionSummary,
    PredictiveScores, SegmentationComponent, CustomerSegment, LifecycleStage, ValueTier,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
