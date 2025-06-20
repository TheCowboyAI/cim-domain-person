//! Name-related components for handling complex personal names
//!
//! This module provides components for representing names in various cultural
//! contexts, including support for titles, honorifics, multiple middle names,
//! and generational suffixes.

use cim_domain::Component;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Comprehensive name component supporting various naming conventions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameComponent {
    /// Title (Dr., Prof., etc.)
    pub title: Option<String>,
    
    /// Honorific prefix (Mr., Mrs., Ms., Mx., etc.)
    pub honorific: Option<String>,
    
    /// Given/first names (can be multiple)
    pub given_names: Vec<String>,
    
    /// Middle names (preserving order for cultural significance)
    pub middle_names: Vec<String>,
    
    /// Family/last names (can be multiple, e.g., Spanish naming)
    pub family_names: Vec<String>,
    
    /// Maternal family name (for cultures that use both)
    pub maternal_family_name: Option<String>,
    
    /// Generational suffix (Jr., Sr., III, etc.)
    pub generational_suffix: Option<String>,
    
    /// Professional suffix (MD, PhD, Esq., etc.)
    pub professional_suffix: Option<String>,
    
    /// Preferred name or nickname
    pub preferred_name: Option<String>,
    
    /// Name order preference (given-first or family-first)
    pub name_order: NameOrder,
    
    /// Cultural context for name interpretation
    pub cultural_context: Option<String>,
}

/// Name ordering preference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NameOrder {
    /// Western style: given name(s) first
    GivenFirst,
    /// Eastern style: family name first
    FamilyFirst,
    /// Custom order with specific formatting rules
    Custom(String),
}

/// Alternative names (aliases, previous names, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternativeNamesComponent {
    /// List of alternative names
    pub names: Vec<AlternativeName>,
}

/// An alternative name with context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternativeName {
    /// The alternative name
    pub name: NameComponent,
    
    /// Type of alternative name
    pub name_type: AlternativeNameType,
    
    /// When this name was/is used
    pub period: Option<NamePeriod>,
    
    /// Context or reason for this name
    pub context: Option<String>,
}

/// Type of alternative name
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlternativeNameType {
    /// Legal name change
    PreviousLegal,
    /// Professional/stage name
    Professional,
    /// Nickname or alias
    Alias,
    /// Maiden name
    Maiden,
    /// Religious name
    Religious,
    /// Translation or transliteration
    Translation,
    /// Other type with description
    Other(String),
}

/// Period when a name was/is used
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamePeriod {
    /// Start date
    pub from: Option<chrono::NaiveDate>,
    
    /// End date (None if current)
    pub to: Option<chrono::NaiveDate>,
}

impl NameComponent {
    /// Create a simple name with just given and family names
    pub fn simple(given_name: String, family_name: String) -> Self {
        Self {
            title: None,
            honorific: None,
            given_names: vec![given_name],
            middle_names: vec![],
            family_names: vec![family_name],
            maternal_family_name: None,
            generational_suffix: None,
            professional_suffix: None,
            preferred_name: None,
            name_order: NameOrder::GivenFirst,
            cultural_context: None,
        }
    }
    
    /// Format the name according to its ordering preference
    pub fn formatted_name(&self) -> String {
        let mut parts = Vec::new();
        
        // Add title if present
        if let Some(title) = &self.title {
            parts.push(title.clone());
        }
        
        // Add honorific if present
        if let Some(honorific) = &self.honorific {
            parts.push(honorific.clone());
        }
        
        // Format based on name order
        match &self.name_order {
            NameOrder::GivenFirst => {
                // Given names
                parts.extend(self.given_names.clone());
                
                // Middle names
                parts.extend(self.middle_names.clone());
                
                // Family names
                parts.extend(self.family_names.clone());
                
                // Maternal family name
                if let Some(maternal) = &self.maternal_family_name {
                    parts.push(maternal.clone());
                }
            }
            NameOrder::FamilyFirst => {
                // Family names first
                parts.extend(self.family_names.clone());
                
                // Maternal family name
                if let Some(maternal) = &self.maternal_family_name {
                    parts.push(maternal.clone());
                }
                
                // Given names
                parts.extend(self.given_names.clone());
                
                // Middle names
                parts.extend(self.middle_names.clone());
            }
            NameOrder::Custom(format) => {
                // TODO: Implement custom formatting rules
                return format.clone();
            }
        }
        
        // Add generational suffix
        if let Some(suffix) = &self.generational_suffix {
            parts.push(suffix.clone());
        }
        
        // Add professional suffix
        if let Some(suffix) = &self.professional_suffix {
            parts.push(suffix.clone());
        }
        
        parts.join(" ")
    }
    
    /// Get display name (preferred name or formatted name)
    pub fn display_name(&self) -> String {
        self.preferred_name
            .clone()
            .unwrap_or_else(|| self.formatted_name())
    }
}

impl Component for NameComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Name"
    }
}

impl Component for AlternativeNamesComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "AlternativeNames"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_name() {
        let name = NameComponent::simple("John".to_string(), "Doe".to_string());
        assert_eq!(name.display_name(), "John Doe");
    }

    #[test]
    fn test_complex_name() {
        let name = NameComponent {
            title: Some("Dr.".to_string()),
            honorific: Some("Mr.".to_string()),
            given_names: vec!["John".to_string()],
            middle_names: vec!["Michael".to_string(), "James".to_string()],
            family_names: vec!["Smith".to_string()],
            maternal_family_name: None,
            generational_suffix: Some("Jr.".to_string()),
            professional_suffix: Some("PhD".to_string()),
            preferred_name: None,
            name_order: NameOrder::GivenFirst,
            cultural_context: None,
        };
        
        assert_eq!(
            name.formatted_name(),
            "Dr. Mr. John Michael James Smith Jr. PhD"
        );
    }

    #[test]
    fn test_spanish_style_name() {
        let name = NameComponent {
            title: None,
            honorific: None,
            given_names: vec!["María".to_string(), "Isabel".to_string()],
            middle_names: vec![],
            family_names: vec!["García".to_string()],
            maternal_family_name: Some("López".to_string()),
            generational_suffix: None,
            professional_suffix: None,
            preferred_name: None,
            name_order: NameOrder::GivenFirst,
            cultural_context: Some("Spanish".to_string()),
        };
        
        assert_eq!(name.formatted_name(), "María Isabel García López");
    }

    #[test]
    fn test_eastern_name_order() {
        let name = NameComponent {
            title: None,
            honorific: None,
            given_names: vec!["Taro".to_string()],
            middle_names: vec![],
            family_names: vec!["Yamada".to_string()],
            maternal_family_name: None,
            generational_suffix: None,
            professional_suffix: None,
            preferred_name: None,
            name_order: NameOrder::FamilyFirst,
            cultural_context: Some("Japanese".to_string()),
        };
        
        assert_eq!(name.formatted_name(), "Yamada Taro");
    }
} 