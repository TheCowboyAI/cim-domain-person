//! Person-Location cross-domain relationships
//!
//! This module manages the relationship between persons and their addresses
//! without duplicating the Address value object from the location domain.

use crate::PersonId;
use cim_domain::EntityId;
use cim_domain_location::value_objects::Address;
use cim_domain_location::aggregate::LocationMarker;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Type alias for Location ID
pub type LocationId = EntityId<LocationMarker>;

/// Represents the relationship between a person and an address
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonAddress {
    /// The person who has this address
    pub person_id: PersonId,
    
    /// Reference to the location in location domain
    pub location_id: LocationId,
    
    /// Type of address (home, work, etc.)
    pub address_type: PersonAddressType,
    
    /// Whether this is the primary address
    pub is_primary: bool,
    
    /// When this address was associated
    pub associated_at: DateTime<Utc>,
    
    /// When this association ends (if applicable)
    pub valid_until: Option<DateTime<Utc>>,
}

/// Types of addresses a person can have
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonAddressType {
    Home,
    Work,
    Billing,
    Shipping,
    Mailing,
    Other(String),
}

/// Commands for managing person-address relationships
#[derive(Debug, Clone)]
pub enum PersonAddressCommand {
    /// Associate an address with a person
    AssociateAddress {
        person_id: PersonId,
        location_id: LocationId,
        address_type: PersonAddressType,
        is_primary: bool,
    },
    
    /// Remove an address association
    DisassociateAddress {
        person_id: PersonId,
        location_id: LocationId,
    },
    
    /// Change the primary address
    SetPrimaryAddress {
        person_id: PersonId,
        location_id: LocationId,
    },
}

/// Events for person-address relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonAddressEvent {
    /// Address was associated with person
    AddressAssociated {
        person_id: PersonId,
        location_id: LocationId,
        address_type: PersonAddressType,
        is_primary: bool,
        associated_at: DateTime<Utc>,
    },
    
    /// Address association was removed
    AddressDisassociated {
        person_id: PersonId,
        location_id: LocationId,
        disassociated_at: DateTime<Utc>,
    },
    
    /// Primary address was changed
    PrimaryAddressChanged {
        person_id: PersonId,
        old_location_id: Option<LocationId>,
        new_location_id: LocationId,
        changed_at: DateTime<Utc>,
    },
}

/// Service for coordinating person-address operations across domains
#[allow(async_fn_in_trait)]
pub trait PersonAddressService {
    /// Get all addresses for a person
    async fn get_person_addresses(&self, person_id: PersonId) -> Result<Vec<(PersonAddress, Address)>, String>;
    
    /// Associate an address with a person
    async fn associate_address(
        &self,
        person_id: PersonId,
        address: Address,
        address_type: PersonAddressType,
        is_primary: bool,
    ) -> Result<LocationId, String>;
    
    /// Remove an address association
    async fn disassociate_address(
        &self,
        person_id: PersonId,
        location_id: LocationId,
    ) -> Result<(), String>;
} 