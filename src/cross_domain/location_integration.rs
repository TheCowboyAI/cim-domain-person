//! Integration with Location domain for address management

use crate::aggregate::{PersonId, ComponentType};
use crate::events::PersonEvent;
use crate::infrastructure::PersonRepository;
use cim_domain::{DomainResult, DomainError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};

/// Events from Location domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationDomainEvent {
    AddressCreated {
        address_id: String,
        street_line1: String,
        street_line2: Option<String>,
        city: String,
        state_province: Option<String>,
        postal_code: String,
        country: String,
        latitude: Option<f64>,
        longitude: Option<f64>,
        created_at: DateTime<Utc>,
    },
    AddressValidated {
        address_id: String,
        validation_status: AddressValidationStatus,
        standardized_address: Option<StandardizedAddress>,
        validated_at: DateTime<Utc>,
    },
    AddressAssociatedWithPerson {
        address_id: String,
        person_id: PersonId,
        address_type: AddressUsageType,
        is_primary: bool,
        effective_date: DateTime<Utc>,
    },
    AddressDisassociatedFromPerson {
        address_id: String,
        person_id: PersonId,
        reason: Option<String>,
        end_date: DateTime<Utc>,
    },
    PersonMovedAddress {
        person_id: PersonId,
        from_address_id: String,
        to_address_id: String,
        move_date: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressValidationStatus {
    Valid,
    InvalidFormat,
    NotFound,
    Ambiguous,
    Undeliverable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardizedAddress {
    pub street_line1: String,
    pub street_line2: Option<String>,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country_code: String,
    pub delivery_point_barcode: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressUsageType {
    Residential,
    Business,
    Mailing,
    Billing,
    Shipping,
    Emergency,
    Temporary,
}

/// Handler for Location domain events
pub struct LocationEventHandler {
    person_repository: Arc<PersonRepository>,
}

impl LocationEventHandler {
    pub fn new(person_repository: Arc<PersonRepository>) -> Self {
        Self { person_repository }
    }
    
    /// Process an event from the Location domain
    pub async fn handle_event(&self, event: LocationDomainEvent) -> DomainResult<Vec<PersonEvent>> {
        match event {
            LocationDomainEvent::AddressAssociatedWithPerson {
                person_id,
                address_type,
                ..
            } => {
                self.handle_address_associated(person_id, address_type).await
            }
            LocationDomainEvent::AddressDisassociatedFromPerson {
                person_id,
                ..
            } => {
                self.handle_address_disassociated(person_id).await
            }
            LocationDomainEvent::PersonMovedAddress {
                person_id,
                to_address_id,
                ..
            } => {
                self.handle_person_moved(person_id, to_address_id).await
            }
            _ => Ok(vec![]),
        }
    }
    
    async fn handle_address_associated(
        &self,
        person_id: PersonId,
        _address_type: AddressUsageType,
    ) -> DomainResult<Vec<PersonEvent>> {
        // Load person to verify they exist
        let person = self.person_repository.load(person_id).await?;
        if person.is_none() {
            return Err(DomainError::AggregateNotFound(format!("Person {}", person_id)));
        }
        
        let person = person.unwrap();
        
        // Register that person now has an address component
        if !person.has_component(&ComponentType::Address) {
            let event = PersonEvent::ComponentRegistered(
                crate::events::ComponentRegistered {
                    person_id,
                    component_type: ComponentType::Address,
                    registered_at: Utc::now(),
                }
            );
            
            return Ok(vec![event]);
        }
        
        Ok(vec![])
    }
    
    async fn handle_address_disassociated(
        &self,
        person_id: PersonId,
    ) -> DomainResult<Vec<PersonEvent>> {
        // Check if person has any other addresses
        // If not, unregister the Address component type
        // For now, we'll just log this
        tracing::info!("Address disassociated from person {}", person_id);
        Ok(vec![])
    }
    
    async fn handle_person_moved(
        &self,
        person_id: PersonId,
        to_address_id: String,
    ) -> DomainResult<Vec<PersonEvent>> {
        // This could trigger various updates
        // For example, updating communication preferences based on new location
        tracing::info!("Person {} moved to address {}", person_id, to_address_id);
        Ok(vec![])
    }
}

/// Commands to send to Location domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationDomainCommand {
    /// Create a new address for a person
    CreatePersonAddress {
        person_id: PersonId,
        street_line1: String,
        street_line2: Option<String>,
        city: String,
        state_province: Option<String>,
        postal_code: String,
        country: String,
        address_type: AddressUsageType,
        is_primary: bool,
    },
    
    /// Validate an address
    ValidateAddress {
        address_id: String,
        validation_level: ValidationLevel,
    },
    
    /// Get all addresses for a person
    GetPersonAddresses {
        person_id: PersonId,
        include_historical: bool,
    },
    
    /// Update address association
    UpdateAddressAssociation {
        person_id: PersonId,
        address_id: String,
        address_type: Option<AddressUsageType>,
        is_primary: Option<bool>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationLevel {
    Format,      // Just check format
    Existence,   // Check if address exists
    Deliverable, // Check if mail can be delivered
}

/// Service for location-related operations
pub struct LocationIntegrationService {
    person_repository: Arc<PersonRepository>,
}

impl LocationIntegrationService {
    pub fn new(person_repository: Arc<PersonRepository>) -> Self {
        Self { person_repository }
    }
    
    /// Check if a person has any addresses
    pub async fn person_has_addresses(&self, person_id: PersonId) -> DomainResult<bool> {
        let person = self.person_repository.load(person_id).await?;
        match person {
            Some(p) => Ok(p.has_component(&ComponentType::Address)),
            None => Ok(false),
        }
    }
} 