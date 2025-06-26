//! Location component data structures

use serde::{Deserialize, Serialize};
use super::ComponentDataTrait;
use crate::aggregate::ComponentType;
use cim_domain::{DomainResult, DomainError};

/// Address component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressData {
    pub address_type: AddressType,
    pub street1: String,
    pub street2: Option<String>,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
    pub country_code: String,
    pub is_verified: bool,
    pub verification_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressType {
    Residential,
    Business,
    Mailing,
    Billing,
    Shipping,
    Other,
}

impl ComponentDataTrait for AddressData {
    fn component_type(&self) -> ComponentType {
        ComponentType::Address
    }
    
    fn validate(&self) -> DomainResult<()> {
        if self.street1.is_empty() {
            return Err(DomainError::ValidationError("Street address is required".to_string()));
        }
        if self.city.is_empty() {
            return Err(DomainError::ValidationError("City is required".to_string()));
        }
        if self.country.is_empty() {
            return Err(DomainError::ValidationError("Country is required".to_string()));
        }
        if self.country_code.len() != 2 {
            return Err(DomainError::ValidationError("Country code must be 2 characters".to_string()));
        }
        Ok(())
    }
    
    fn summary(&self) -> String {
        format!("{}, {}, {}", self.street1, self.city, self.country)
    }
}

/// Location data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationData {
    pub address: Option<AddressData>,
    pub coordinates: Option<Coordinates>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f32>,
}

impl LocationData {
    pub fn location_type(&self) -> &'static str {
        "Location"
    }
}

impl ComponentDataTrait for LocationData {
    fn component_type(&self) -> ComponentType {
        ComponentType::Address
    }
    
    fn validate(&self) -> DomainResult<()> {
        if let Some(address) = &self.address {
            address.validate()?;
        }
        
        if let Some(coords) = &self.coordinates {
            if coords.latitude < -90.0 || coords.latitude > 90.0 {
                return Err(DomainError::ValidationError("Latitude must be between -90 and 90".to_string()));
            }
            if coords.longitude < -180.0 || coords.longitude > 180.0 {
                return Err(DomainError::ValidationError("Longitude must be between -180 and 180".to_string()));
            }
        }
        
        Ok(())
    }
    
    fn summary(&self) -> String {
        if let Some(address) = &self.address {
            address.summary()
        } else if let Some(coords) = &self.coordinates {
            format!("Location at {:.4}, {:.4}", coords.latitude, coords.longitude)
        } else {
            "Empty location".to_string()
        }
    }
} 