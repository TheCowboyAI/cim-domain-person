//! Cross-domain relationship types for Person domain

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Relationship between a person and an organization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonOrganizationRelation {
    /// Organization ID from Organization domain
    pub organization_id: Uuid,
    /// Type of relationship
    pub relation_type: OrganizationRelationType,
    /// Role or position in the organization
    pub role: Option<String>,
    /// Start date of the relationship
    pub start_date: chrono::DateTime<chrono::Utc>,
    /// End date of the relationship (if ended)
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Is this the primary organization
    pub is_primary: bool,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of relationships with organizations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrganizationRelationType {
    /// Employment relationship
    Employment {
        department: Option<String>,
        reports_to: Option<Uuid>,
    },
    /// Contractor/consultant relationship
    Contractor {
        contract_type: String,
        agency: Option<String>,
    },
    /// Board member
    BoardMember {
        position: String,
        committees: Vec<String>,
    },
    /// Volunteer
    Volunteer {
        cause: String,
        hours_per_week: Option<f32>,
    },
    /// Member of organization (club, association, etc.)
    Member {
        membership_type: String,
        membership_number: Option<String>,
    },
    /// Customer/client relationship
    Customer {
        customer_id: Option<String>,
        tier: Option<String>,
    },
    /// Partner relationship
    Partner {
        partnership_type: String,
        equity_percentage: Option<f32>,
    },
    /// Other relationship
    Other(String),
}

/// Relationship between a person and a location
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonLocationRelation {
    /// Location ID from Location domain
    pub location_id: Uuid,
    /// Type of relationship
    pub relation_type: LocationRelationType,
    /// Start date of the relationship
    pub start_date: chrono::DateTime<chrono::Utc>,
    /// End date of the relationship (if ended)
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    /// Is this the primary location
    pub is_primary: bool,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of relationships with locations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LocationRelationType {
    /// Residence
    Residence {
        residence_type: ResidenceType,
        ownership_status: OwnershipStatus,
    },
    /// Work location
    WorkLocation {
        office_number: Option<String>,
        desk_number: Option<String>,
        remote_percentage: Option<f32>,
    },
    /// Mailing address
    MailingAddress,
    /// Billing address
    BillingAddress,
    /// Vacation home
    VacationHome,
    /// Previous residence
    PreviousResidence {
        moved_out_date: chrono::DateTime<chrono::Utc>,
    },
    /// Birth place
    BirthPlace,
    /// Other location relationship
    Other(String),
}

/// Type of residence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResidenceType {
    House,
    Apartment,
    Condo,
    Townhouse,
    MobileHome,
    Dorm,
    Other(String),
}

/// Ownership status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OwnershipStatus {
    Owner,
    Renter,
    Lease,
    FamilyOwned,
    CompanyProvided,
    Other(String),
}

/// Professional network relationship
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfessionalNetworkRelation {
    /// Other person ID
    pub other_person_id: Uuid,
    /// Type of professional relationship
    pub relation_type: ProfessionalRelationType,
    /// Strength of relationship (0.0 to 1.0)
    pub strength: f32,
    /// When the relationship started
    pub established_date: chrono::DateTime<chrono::Utc>,
    /// Last interaction date
    pub last_interaction: Option<chrono::DateTime<chrono::Utc>>,
    /// Interaction count
    pub interaction_count: u32,
    /// Mutual connections count
    pub mutual_connections: u32,
}

/// Types of professional relationships
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProfessionalRelationType {
    /// Direct colleague
    Colleague {
        same_team: bool,
        same_department: bool,
    },
    /// Manager/subordinate
    Manager,
    Subordinate,
    /// Mentor/mentee
    Mentor,
    Mentee,
    /// Business partner
    BusinessPartner,
    /// Client
    Client,
    /// Vendor/supplier
    Vendor,
    /// Professional contact
    ProfessionalContact,
    /// Alumni connection
    Alumni {
        institution: String,
        graduation_year: Option<i32>,
    },
    /// Conference contact
    ConferenceContact {
        event_name: String,
        event_date: chrono::DateTime<chrono::Utc>,
    },
    /// Other professional relationship
    Other(String),
}

impl ProfessionalNetworkRelation {
    /// Calculate influence score based on relationship metrics
    pub fn influence_score(&self) -> f32 {
        let base_score = self.strength;
        let interaction_factor = (self.interaction_count as f32).ln() / 10.0;
        let mutual_factor = (self.mutual_connections as f32).ln() / 20.0;
        
        (base_score + interaction_factor + mutual_factor).min(1.0)
    }
    
    /// Check if relationship is active (interaction within last 6 months)
    pub fn is_active(&self) -> bool {
        if let Some(last_interaction) = self.last_interaction {
            let six_months_ago = chrono::Utc::now() - chrono::Duration::days(180);
            last_interaction > six_months_ago
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_organization_relation_types() {
        let employment = OrganizationRelationType::Employment {
            department: Some("Engineering".to_string()),
            reports_to: Some(Uuid::new_v4()),
        };
        
        let contractor = OrganizationRelationType::Contractor {
            contract_type: "Fixed Term".to_string(),
            agency: Some("Tech Staffing Inc".to_string()),
        };
        
        assert_ne!(employment, contractor);
    }
    
    #[test]
    fn test_location_relation_types() {
        let residence = LocationRelationType::Residence {
            residence_type: ResidenceType::Apartment,
            ownership_status: OwnershipStatus::Renter,
        };
        
        let work = LocationRelationType::WorkLocation {
            office_number: Some("5-123".to_string()),
            desk_number: Some("A-42".to_string()),
            remote_percentage: Some(60.0),
        };
        
        assert_ne!(residence, work);
    }
    
    #[test]
    fn test_professional_network_influence() {
        let relation = ProfessionalNetworkRelation {
            other_person_id: Uuid::new_v4(),
            relation_type: ProfessionalRelationType::Colleague {
                same_team: true,
                same_department: true,
            },
            strength: 0.8,
            established_date: chrono::Utc::now() - chrono::Duration::days(365),
            last_interaction: Some(chrono::Utc::now() - chrono::Duration::days(7)),
            interaction_count: 50,
            mutual_connections: 25,
        };
        
        let influence = relation.influence_score();
        assert!(influence > 0.8);
        assert!(influence <= 1.0);
        assert!(relation.is_active());
    }
} 