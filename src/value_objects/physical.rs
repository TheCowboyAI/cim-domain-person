//! Physical attribute components for describing a person's appearance
//!
//! These components capture physical characteristics that may be relevant
//! for identification, medical, or other legitimate business purposes.

use cim_domain::Component;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Physical attributes of a person
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalAttributesComponent {
    /// Height in centimeters
    pub height_cm: Option<f32>,
    
    /// Weight in kilograms
    pub weight_kg: Option<f32>,
    
    /// Build/body type
    pub build: Option<Build>,
    
    /// Hair color
    pub hair_color: Option<String>,
    
    /// Hair style/length
    pub hair_style: Option<String>,
    
    /// Eye color
    pub eye_color: Option<String>,
    
    /// Skin tone (descriptive, not racial)
    pub skin_tone: Option<String>,
    
    /// Facial hair description
    pub facial_hair: Option<String>,
    
    /// Whether person wears glasses/contacts
    pub vision_correction: Option<VisionCorrection>,
    
    /// General appearance notes
    pub appearance_notes: Option<String>,
}

/// Build/body type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Build {
    Slim,
    Athletic,
    Average,
    Stocky,
    Heavy,
    Other(String),
}

/// Vision correction type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisionCorrection {
    None,
    Glasses,
    Contacts,
    Both,
    LaserSurgery,
}

/// Distinguishing marks and features
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistinguishingMarksComponent {
    /// List of distinguishing marks
    pub marks: Vec<DistinguishingMark>,
}

/// A specific distinguishing mark
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistinguishingMark {
    /// Type of mark
    pub mark_type: MarkType,
    
    /// Description of the mark
    pub description: String,
    
    /// Location on body
    pub location: String,
    
    /// Visibility (always visible, sometimes, covered by clothing, etc.)
    pub visibility: Option<String>,
    
    /// Additional notes
    pub notes: Option<String>,
}

/// Type of distinguishing mark
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkType {
    Scar,
    Tattoo,
    Birthmark,
    Piercing,
    Prosthetic,
    Other(String),
}

/// Biometric data (for high-security applications)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiometricComponent {
    /// Fingerprint hashes (not raw data)
    pub fingerprint_hashes: Vec<BiometricHash>,
    
    /// Face encoding vector (privacy-preserving)
    pub face_encoding: Option<Vec<f32>>,
    
    /// Voice print hash
    pub voice_print_hash: Option<String>,
    
    /// Iris scan hash
    pub iris_scan_hash: Option<String>,
    
    /// When biometrics were last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// A hashed biometric identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiometricHash {
    /// Which biometric (e.g., "right_thumb", "left_index")
    pub identifier: String,
    
    /// Hashed value (not reversible)
    pub hash: String,
    
    /// Algorithm used for hashing
    pub algorithm: String,
}

/// Medical information relevant to identity/emergency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MedicalIdentityComponent {
    /// Blood type
    pub blood_type: Option<BloodType>,
    
    /// Known allergies (for emergency response)
    pub allergies: Vec<String>,
    
    /// Critical medical conditions
    pub medical_conditions: Vec<String>,
    
    /// Emergency medications
    pub medications: Vec<String>,
    
    /// Emergency contact preference
    pub emergency_contact_id: Option<uuid::Uuid>,
    
    /// Organ donor status
    pub organ_donor: Option<bool>,
    
    /// DNR (Do Not Resuscitate) status
    pub dnr_status: Option<bool>,
}

/// Blood type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BloodType {
    APositive,
    ANegative,
    BPositive,
    BNegative,
    ABPositive,
    ABNegative,
    OPositive,
    ONegative,
}

// Component trait implementations

impl Component for PhysicalAttributesComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "PhysicalAttributes"
    }
}

impl Component for DistinguishingMarksComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "DistinguishingMarks"
    }
}

impl Component for BiometricComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "Biometric"
    }
}

impl Component for MedicalIdentityComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }

    fn type_name(&self) -> &'static str {
        "MedicalIdentity"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physical_attributes() {
        let attrs = PhysicalAttributesComponent {
            height_cm: Some(180.0),
            weight_kg: Some(75.0),
            build: Some(Build::Athletic),
            hair_color: Some("Brown".to_string()),
            hair_style: Some("Short".to_string()),
            eye_color: Some("Blue".to_string()),
            skin_tone: Some("Fair".to_string()),
            facial_hair: Some("Clean shaven".to_string()),
            vision_correction: Some(VisionCorrection::Contacts),
            appearance_notes: None,
        };

        assert_eq!(attrs.height_cm, Some(180.0));
        assert_eq!(attrs.build, Some(Build::Athletic));
    }

    #[test]
    fn test_distinguishing_marks() {
        let marks = DistinguishingMarksComponent {
            marks: vec![
                DistinguishingMark {
                    mark_type: MarkType::Scar,
                    description: "2cm linear scar".to_string(),
                    location: "Left forearm".to_string(),
                    visibility: Some("Visible when wearing short sleeves".to_string()),
                    notes: None,
                },
                DistinguishingMark {
                    mark_type: MarkType::Tattoo,
                    description: "Small anchor design".to_string(),
                    location: "Right shoulder".to_string(),
                    visibility: Some("Usually covered by clothing".to_string()),
                    notes: Some("Navy service tattoo".to_string()),
                },
            ],
        };

        assert_eq!(marks.marks.len(), 2);
        assert_eq!(marks.marks[0].mark_type, MarkType::Scar);
    }

    #[test]
    fn test_blood_type() {
        let medical = MedicalIdentityComponent {
            blood_type: Some(BloodType::OPositive),
            allergies: vec!["Penicillin".to_string()],
            medical_conditions: vec!["Type 2 Diabetes".to_string()],
            medications: vec!["Insulin".to_string()],
            emergency_contact_id: None,
            organ_donor: Some(true),
            dnr_status: Some(false),
        };

        assert_eq!(medical.blood_type, Some(BloodType::OPositive));
        assert!(medical.organ_donor.unwrap());
    }
} 