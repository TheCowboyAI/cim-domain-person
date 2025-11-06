//! PersonName value object - Comprehensive name representation
//!
//! This module provides a culturally-aware, structured representation of human names.
//! See `doc/person-names-design.md` for full design rationale and examples.
//!
//! ## Key Principles
//!
//! 1. **Names are structured collections**, not simple strings
//! 2. **Titles/honorifics are temporal** (tracked separately from name)
//! 3. **Cultural conventions matter** (Spanish, Chinese, Icelandic, etc.)
//! 4. **Minimum requirement**: At least one character (what noise gets your attention)
//! 5. **Immutable value objects** (name changes → new Name)

use serde::{Deserialize, Serialize};
use std::fmt;
use chrono::NaiveDate;
use cim_domain::DomainResult;

/// Core name components - the immutable identity of how a person is named
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameComponents {
    /// Given names (first, middle, etc.) - at least one required
    /// Examples: ["Jane"], ["Pablo", "Diego", "José"], ["明"]
    pub given_names: Vec<String>,

    /// Family names (last name, surname) - can be multiple
    /// Examples: ["Smith"], ["Ruiz", "Picasso"], ["李"]
    pub family_names: Vec<String>,

    /// Patronymic (father's name-derived)
    /// Examples: "Guðmundsdóttir" (Icelandic), "Иванович" (Russian)
    pub patronymic: Option<String>,

    /// Matronymic (mother's name-derived)
    /// Used in some cultures
    pub matronymic: Option<String>,

    /// Name prefixes/particles
    /// Examples: "de", "van", "von", "al", "bin"
    pub prefixes: Vec<String>,

    /// Generational suffixes
    /// Examples: "Jr.", "Sr.", "II", "III"
    pub suffixes: Vec<String>,
}

impl NameComponents {
    /// Validate that name components meet minimum requirements
    fn validate(&self) -> DomainResult<()> {
        // Must have at least ONE non-empty component in given OR family names
        let has_given = !self.given_names.is_empty() &&
                       self.given_names.iter().any(|s| !s.trim().is_empty());
        let has_family = !self.family_names.is_empty() &&
                        self.family_names.iter().any(|s| !s.trim().is_empty());

        if !has_given && !has_family {
            return Err(cim_domain::DomainError::generic(
                "Name must have at least one given name or family name"
            ));
        }

        // Validate no empty strings in any component
        for name in &self.given_names {
            if name.trim().is_empty() {
                return Err(cim_domain::DomainError::generic(
                    "Given names cannot be empty strings"
                ));
            }
        }

        for name in &self.family_names {
            if name.trim().is_empty() {
                return Err(cim_domain::DomainError::generic(
                    "Family names cannot be empty strings"
                ));
            }
        }

        Ok(())
    }
}

/// Cultural naming conventions for proper display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamingConvention {
    /// Western: Given Family (Jane Smith)
    Western,

    /// Spanish: Given Middle... Paternal y Maternal
    Spanish,

    /// Chinese/Japanese/Korean: Family Given (李 明)
    EastAsian,

    /// Icelandic: Given Patronymic/Matronymic
    Patronymic,

    /// Indonesian: Often single name (mononym)
    Mononymic,

    /// Custom/Other
    Other,
}

/// Display policies for different contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameDisplayPolicy {
    /// Full formal name with all components
    /// "Jane Elizabeth Smith"
    Formal,

    /// Preferred name or first given name only
    /// "Jane"
    Informal,

    /// Legal format (often surname first for sorting)
    /// "Smith, Jane Elizabeth"
    Legal,

    /// Alphabetical sorting format
    /// "Smith, J."
    Alphabetical,

    /// Respect cultural convention
    /// Spanish: "Pablo Ruiz y Picasso"
    /// Chinese: "李明"
    Cultural,
}

/// PersonName - Immutable value object representing how a person is named
///
/// This does NOT include temporal titles/honorifics (Dr., Sir, etc.)
/// Those are tracked separately as PersonTitle with award/revoke dates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonName {
    /// The structural components of the name
    pub components: NameComponents,

    /// Preferred form for informal use (nickname, etc.)
    pub preferred_form: Option<String>,

    /// Cultural convention for proper display
    pub naming_convention: NamingConvention,
}

impl PersonName {
    /// Create a simple Western-style name (most common case)
    ///
    /// ```
    /// use cim_domain_person::value_objects::PersonName;
    ///
    /// let name = PersonName::simple("Jane", "Smith");
    /// assert_eq!(name.display_name(), "Jane Smith");
    /// ```
    pub fn new(given_name: String, family_name: String) -> Self {
        Self {
            components: NameComponents {
                given_names: vec![given_name],
                family_names: vec![family_name],
                patronymic: None,
                matronymic: None,
                prefixes: Vec::new(),
                suffixes: Vec::new(),
            },
            preferred_form: None,
            naming_convention: NamingConvention::Western,
        }
    }

    /// Create a mononym (single name)
    ///
    /// ```
    /// use cim_domain_person::value_objects::PersonName;
    ///
    /// let name = PersonName::mononym("Suharto");
    /// assert_eq!(name.display_name(), "Suharto");
    /// ```
    pub fn mononym(name: String) -> Self {
        Self {
            components: NameComponents {
                given_names: vec![name],
                family_names: Vec::new(),
                patronymic: None,
                matronymic: None,
                prefixes: Vec::new(),
                suffixes: Vec::new(),
            },
            preferred_form: None,
            naming_convention: NamingConvention::Mononymic,
        }
    }

    /// Parse a full name from a string with automatic convention detection
    ///
    /// ```
    /// use cim_domain_person::value_objects::PersonName;
    ///
    /// // Simple Western name
    /// let name = PersonName::parse("John Doe").unwrap();
    /// assert_eq!(name.components.given_names[0], "John");
    /// assert_eq!(name.components.family_names[0], "Doe");
    ///
    /// // Complex Spanish name
    /// let picasso = PersonName::parse(
    ///     "Pablo Diego José Francisco de Paula Juan Nepomuceno María de los Remedios Cipriano de la Santísima Trinidad Ruiz Picasso"
    /// ).unwrap();
    /// ```
    pub fn parse(full_name: &str) -> DomainResult<Self> {
        Self::parse_with_convention(full_name, None)
    }

    /// Parse a full name with an explicit naming convention hint
    ///
    /// ```
    /// use cim_domain_person::value_objects::{PersonName, NamingConvention};
    ///
    /// let name = PersonName::parse_with_convention("李明", Some(NamingConvention::EastAsian)).unwrap();
    /// ```
    pub fn parse_with_convention(full_name: &str, convention_hint: Option<NamingConvention>) -> DomainResult<Self> {
        let trimmed = full_name.trim();

        if trimmed.is_empty() {
            return Err(cim_domain::DomainError::ValidationError(
                "Name cannot be empty".to_string()
            ));
        }

        // Detect or use provided convention
        let convention = convention_hint.unwrap_or_else(|| Self::detect_convention(trimmed));

        let mut builder = PersonNameBuilder::new()
            .naming_convention(convention);

        match convention {
            NamingConvention::Western => {
                builder = Self::parse_western(trimmed, builder);
            }
            NamingConvention::Spanish => {
                builder = Self::parse_spanish(trimmed, builder);
            }
            NamingConvention::EastAsian => {
                builder = Self::parse_east_asian(trimmed, builder);
            }
            NamingConvention::Patronymic => {
                builder = Self::parse_patronymic(trimmed, builder);
            }
            NamingConvention::Mononymic => {
                builder = builder.given_names(vec![trimmed.to_string()]);
            }
            NamingConvention::Other => {
                builder = Self::parse_western(trimmed, builder);
            }
        }

        builder.build()
    }

    /// Detect likely naming convention from the name string
    fn detect_convention(name: &str) -> NamingConvention {
        // Check for CJK characters FIRST (before splitting by whitespace)
        // because East Asian names often have no spaces
        if name.chars().any(|c| {
            ('\u{4E00}'..='\u{9FFF}').contains(&c) ||  // Chinese
            ('\u{3040}'..='\u{309F}').contains(&c) ||  // Hiragana
            ('\u{30A0}'..='\u{30FF}').contains(&c) ||  // Katakana
            ('\u{AC00}'..='\u{D7AF}').contains(&c)     // Hangul
        }) {
            return NamingConvention::EastAsian;
        }

        let parts: Vec<&str> = name.split_whitespace().collect();

        // Single name (and not CJK) = Mononymic
        if parts.len() == 1 {
            return NamingConvention::Mononymic;
        }

        // Contains " y " (Spanish connector) or "de la/de los/del" patterns
        if name.contains(" y ") ||
           name.contains(" de la ") ||
           name.contains(" de los ") ||
           name.contains(" del ") {
            return NamingConvention::Spanish;
        }

        // Ends with common patronymic suffixes
        if parts.last().map_or(false, |s| {
            s.ends_with("ovich") || s.ends_with("evich") ||
            s.ends_with("ovna") || s.ends_with("evna") ||
            s.ends_with("son") || s.ends_with("sen") ||
            s.ends_with("dóttir") || s.ends_with("dottir")
        }) {
            return NamingConvention::Patronymic;
        }

        // Default to Western
        NamingConvention::Western
    }

    /// Parse Western-style name (Given Middle... Family)
    fn parse_western(name: &str, mut builder: PersonNameBuilder) -> PersonNameBuilder {
        let parts: Vec<&str> = name.split_whitespace().collect();

        if parts.is_empty() {
            return builder;
        }

        // Common particles that belong to family name
        let particles = ["de", "van", "von", "del", "della", "di", "da", "le", "la"];

        // Find where family name starts (particles + last word, or just last word)
        let mut family_start = parts.len() - 1;

        // Look backwards for particles
        for i in (0..parts.len()-1).rev() {
            if particles.contains(&parts[i].to_lowercase().as_str()) {
                family_start = i;
            } else {
                break;
            }
        }

        // Handle suffixes (Jr., Sr., III, etc.)
        let mut actual_family_end = parts.len();
        if let Some(last) = parts.last() {
            let last_lower = last.to_lowercase();
            if last_lower == "jr." || last_lower == "sr." ||
               last_lower == "jr" || last_lower == "sr" ||
               last_lower == "ii" || last_lower == "iii" || last_lower == "iv" {
                builder = builder.suffix(last.to_string());
                actual_family_end = parts.len() - 1;
                if family_start >= actual_family_end {
                    family_start = actual_family_end - 1;
                }
            }
        }

        // Given names (everything before family name start)
        if family_start > 0 {
            builder = builder.given_names(
                parts[0..family_start].iter().map(|s| s.to_string()).collect()
            );
        }

        // Family name (from family_start to actual_family_end)
        if family_start < actual_family_end {
            let family_parts: Vec<String> = parts[family_start..actual_family_end]
                .iter()
                .map(|s| s.to_string())
                .collect();

            // If first part is a particle, add it as prefix
            if !family_parts.is_empty() && particles.contains(&family_parts[0].to_lowercase().as_str()) {
                builder = builder.prefix(family_parts[0].clone());
                if family_parts.len() > 1 {
                    builder = builder.family_names(family_parts[1..].to_vec());
                }
            } else {
                builder = builder.family_names(family_parts);
            }
        }

        builder
    }

    /// Parse Spanish-style name (Given... Paternal Maternal)
    fn parse_spanish(name: &str, mut builder: PersonNameBuilder) -> PersonNameBuilder {
        let parts: Vec<&str> = name.split_whitespace().collect();

        if parts.is_empty() {
            return builder;
        }

        // Spanish names: Usually last 2 words are family names (paternal maternal)
        // But we need to handle particles

        // Find "y" connector if present
        let y_pos = parts.iter().position(|&p| p == "y");

        if let Some(y_idx) = y_pos {
            // Format: "Given... Paternal y Maternal"
            // "y" connects two family names
            // Word before "y" is paternal surname
            // Word(s) after "y" is maternal surname
            // Everything before paternal surname is given names

            if y_idx > 0 {
                // Given names: everything before the word before "y"
                if y_idx > 1 {
                    builder = builder.given_names(
                        parts[0..y_idx-1].iter().map(|s| s.to_string()).collect()
                    );
                }

                // Family names: word before "y" plus everything after "y"
                let mut family_names = vec![parts[y_idx-1].to_string()];
                family_names.extend(parts[y_idx+1..].iter().map(|s| s.to_string()));
                builder = builder.family_names(family_names);
            }
        } else {
            // No "y" - assume last 2 parts are family names (or last 1 if only 2 parts total)
            let family_count = if parts.len() > 2 { 2 } else { 1 };
            let given_end = parts.len() - family_count;

            if given_end > 0 {
                builder = builder.given_names(
                    parts[0..given_end].iter().map(|s| s.to_string()).collect()
                );
            }

            builder = builder.family_names(
                parts[given_end..].iter().map(|s| s.to_string()).collect()
            );
        }

        builder
    }

    /// Parse East Asian name (Family Given)
    fn parse_east_asian(name: &str, mut builder: PersonNameBuilder) -> PersonNameBuilder {
        let trimmed = name.trim();

        // CJK names are often written without spaces
        // Try to split intelligently
        let chars: Vec<char> = trimmed.chars().collect();

        if chars.len() <= 1 {
            return builder.given_names(vec![trimmed.to_string()]);
        }

        // Common pattern:
        // Chinese: 1-2 char family name + 1-2 char given name
        // Japanese: 1-4 char family + 1-3 char given
        // Korean: 1 char family + 2 char given (usually)

        if chars.len() == 2 {
            // Assume 1 char family, 1 char given
            builder = builder.family_names(vec![chars[0].to_string()]);
            builder = builder.given_names(vec![chars[1].to_string()]);
        } else if chars.len() == 3 {
            // Assume 1 char family, 2 char given (most common)
            builder = builder.family_names(vec![chars[0].to_string()]);
            let given: String = chars[1..].iter().collect();
            builder = builder.given_names(vec![given]);
        } else if chars.len() == 4 {
            // Could be 2+2 or 1+3
            // Default to 2+2 for Chinese, 1+3 for others
            let family: String = chars[0..2].iter().collect();
            let given: String = chars[2..].iter().collect();
            builder = builder.family_names(vec![family]);
            builder = builder.given_names(vec![given]);
        } else {
            // Longer names - split roughly in middle
            let mid = chars.len() / 2;
            let family: String = chars[0..mid].iter().collect();
            let given: String = chars[mid..].iter().collect();
            builder = builder.family_names(vec![family]);
            builder = builder.given_names(vec![given]);
        }

        builder
    }

    /// Parse Patronymic name (Given Patronymic)
    fn parse_patronymic(name: &str, mut builder: PersonNameBuilder) -> PersonNameBuilder {
        let parts: Vec<&str> = name.split_whitespace().collect();

        if parts.is_empty() {
            return builder;
        }

        if parts.len() == 1 {
            return builder.given_names(vec![parts[0].to_string()]);
        }

        // First part(s) are given names, last part is patronymic
        if parts.len() == 2 {
            builder = builder.given_names(vec![parts[0].to_string()]);
            builder = builder.patronymic(parts[1].to_string());
        } else {
            // Multiple given names + patronymic
            builder = builder.given_names(
                parts[0..parts.len()-1].iter().map(|s| s.to_string()).collect()
            );
            builder = builder.patronymic(parts[parts.len()-1].to_string());
        }

        builder
    }

    /// Create a name builder for complex cases
    pub fn builder() -> PersonNameBuilder {
        PersonNameBuilder::new()
    }

    /// Validate the name meets requirements
    pub fn validate(&self) -> DomainResult<()> {
        self.components.validate()
    }

    /// Get display name (preferred or constructed)
    pub fn display_name(&self) -> String {
        if let Some(ref preferred) = self.preferred_form {
            return preferred.clone();
        }

        self.display(NameDisplayPolicy::Informal)
    }

    /// Get full formal name
    pub fn full_name(&self) -> String {
        self.display(NameDisplayPolicy::Formal)
    }

    /// Display name according to policy
    pub fn display(&self, policy: NameDisplayPolicy) -> String {
        match policy {
            NameDisplayPolicy::Formal => self.format_formal(),
            NameDisplayPolicy::Informal => self.format_informal(),
            NameDisplayPolicy::Legal => self.format_legal(),
            NameDisplayPolicy::Alphabetical => self.format_alphabetical(),
            NameDisplayPolicy::Cultural => self.format_cultural(),
        }
    }

    fn format_formal(&self) -> String {
        let mut parts = Vec::new();

        // Prefixes
        parts.extend(self.components.prefixes.clone());

        // All given names
        parts.extend(self.components.given_names.clone());

        // Patronymic/Matronymic
        if let Some(ref p) = self.components.patronymic {
            parts.push(p.clone());
        }
        if let Some(ref m) = self.components.matronymic {
            parts.push(m.clone());
        }

        // All family names
        parts.extend(self.components.family_names.clone());

        // Suffixes
        parts.extend(self.components.suffixes.clone());

        parts.join(" ")
    }

    fn format_informal(&self) -> String {
        if let Some(ref preferred) = self.preferred_form {
            return preferred.clone();
        }

        // First given name, or first family name if no given
        if !self.components.given_names.is_empty() {
            self.components.given_names[0].clone()
        } else if !self.components.family_names.is_empty() {
            self.components.family_names[0].clone()
        } else if let Some(ref p) = self.components.patronymic {
            p.clone()
        } else {
            "Unknown".to_string()
        }
    }

    fn format_legal(&self) -> String {
        // Surname, Given Names
        let mut parts = Vec::new();

        if !self.components.family_names.is_empty() {
            parts.push(self.components.family_names.join(" "));
        }

        if !parts.is_empty() && !self.components.given_names.is_empty() {
            parts.push(",".to_string());
        }

        parts.extend(self.components.given_names.clone());

        parts.join(" ")
    }

    fn format_alphabetical(&self) -> String {
        // Surname, Initial(s)
        let mut result = String::new();

        if !self.components.family_names.is_empty() {
            result.push_str(&self.components.family_names.join(" "));
            result.push_str(", ");
        }

        for (i, given) in self.components.given_names.iter().enumerate() {
            if let Some(first_char) = given.chars().next() {
                result.push(first_char);
                result.push('.');
                if i < self.components.given_names.len() - 1 {
                    result.push(' ');
                }
            }
        }

        result
    }

    fn format_cultural(&self) -> String {
        match self.naming_convention {
            NamingConvention::Western => self.format_formal(),

            NamingConvention::Spanish => {
                // Given names, then "Paternal y Maternal"
                let mut parts = Vec::new();
                parts.extend(self.components.given_names.clone());

                if self.components.family_names.len() >= 2 {
                    parts.push(format!(
                        "{} y {}",
                        self.components.family_names[0],
                        self.components.family_names[1]
                    ));
                } else {
                    parts.extend(self.components.family_names.clone());
                }

                parts.join(" ")
            }

            NamingConvention::EastAsian => {
                // Family name first, then given names
                let mut parts = Vec::new();
                parts.extend(self.components.family_names.clone());
                parts.extend(self.components.given_names.clone());
                parts.join("")
            }

            NamingConvention::Patronymic => {
                // Given + Patronymic
                let mut parts = Vec::new();
                parts.extend(self.components.given_names.clone());
                if let Some(ref p) = self.components.patronymic {
                    parts.push(p.clone());
                }
                parts.join(" ")
            }

            NamingConvention::Mononymic => {
                // Just the name
                if !self.components.given_names.is_empty() {
                    self.components.given_names[0].clone()
                } else if !self.components.family_names.is_empty() {
                    self.components.family_names[0].clone()
                } else {
                    "Unknown".to_string()
                }
            }

            NamingConvention::Other => self.format_formal(),
        }
    }
}

impl fmt::Display for PersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Builder for complex PersonName construction
pub struct PersonNameBuilder {
    given_names: Vec<String>,
    family_names: Vec<String>,
    patronymic: Option<String>,
    matronymic: Option<String>,
    prefixes: Vec<String>,
    suffixes: Vec<String>,
    preferred_form: Option<String>,
    naming_convention: NamingConvention,
}

impl PersonNameBuilder {
    pub fn new() -> Self {
        Self {
            given_names: Vec::new(),
            family_names: Vec::new(),
            patronymic: None,
            matronymic: None,
            prefixes: Vec::new(),
            suffixes: Vec::new(),
            preferred_form: None,
            naming_convention: NamingConvention::Western,
        }
    }

    pub fn given_name(mut self, name: impl Into<String>) -> Self {
        self.given_names.push(name.into());
        self
    }

    pub fn given_names(mut self, names: Vec<impl Into<String>>) -> Self {
        self.given_names.extend(names.into_iter().map(|n| n.into()));
        self
    }

    pub fn family_name(mut self, name: impl Into<String>) -> Self {
        self.family_names.push(name.into());
        self
    }

    pub fn family_names(mut self, names: Vec<impl Into<String>>) -> Self {
        self.family_names.extend(names.into_iter().map(|n| n.into()));
        self
    }

    pub fn patronymic(mut self, name: impl Into<String>) -> Self {
        self.patronymic = Some(name.into());
        self
    }

    pub fn matronymic(mut self, name: impl Into<String>) -> Self {
        self.matronymic = Some(name.into());
        self
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefixes.push(prefix.into());
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffixes.push(suffix.into());
        self
    }

    pub fn preferred(mut self, name: impl Into<String>) -> Self {
        self.preferred_form = Some(name.into());
        self
    }

    pub fn naming_convention(mut self, convention: NamingConvention) -> Self {
        self.naming_convention = convention;
        self
    }

    pub fn build(self) -> DomainResult<PersonName> {
        let name = PersonName {
            components: NameComponents {
                given_names: self.given_names,
                family_names: self.family_names,
                patronymic: self.patronymic,
                matronymic: self.matronymic,
                prefixes: self.prefixes,
                suffixes: self.suffixes,
            },
            preferred_form: self.preferred_form,
            naming_convention: self.naming_convention,
        };

        name.validate()?;
        Ok(name)
    }
}

impl Default for PersonNameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Temporal title or honorific (NOT part of the immutable name)
///
/// Examples: Dr., Sir, Lord, MD, PhD
/// These can be awarded, revoked, or expire
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonTitle {
    /// The title itself (e.g., "Dr.", "Sir", "MD")
    pub title: String,

    /// Type classification
    pub title_type: TitleType,

    /// When this title was awarded/earned
    pub awarded_date: Option<NaiveDate>,

    /// When this title was revoked (if applicable)
    pub revoked_date: Option<NaiveDate>,

    /// When this title expires (if time-limited)
    pub expiry_date: Option<NaiveDate>,

    /// Who issued/awarded this title
    pub issuing_authority: Option<String>,

    /// Why it was revoked (if applicable)
    pub revocation_reason: Option<String>,
}

/// Classification of title types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitleType {
    /// Academic (Dr., Prof.)
    Academic,

    /// Professional credentials (MD, PhD, Esq., CPA)
    Professional,

    /// Noble titles (Sir, Lord, Duke, Baron)
    Noble,

    /// Honorary (Hon., Rev.)
    Honorary,

    /// Military rank (Col., Gen., Maj.)
    Military,

    /// Religious (Fr., Rabbi, Imam)
    Religious,

    /// Other
    Other,
}

impl PersonTitle {
    pub fn new(title: impl Into<String>, title_type: TitleType) -> Self {
        Self {
            title: title.into(),
            title_type,
            awarded_date: None,
            revoked_date: None,
            expiry_date: None,
            issuing_authority: None,
            revocation_reason: None,
        }
    }

    /// Check if title is currently valid
    pub fn is_valid_on(&self, date: NaiveDate) -> bool {
        // Not yet awarded
        if let Some(awarded) = self.awarded_date {
            if date < awarded {
                return false;
            }
        }

        // Already revoked
        if let Some(revoked) = self.revoked_date {
            if date >= revoked {
                return false;
            }
        }

        // Expired
        if let Some(expiry) = self.expiry_date {
            if date >= expiry {
                return false;
            }
        }

        true
    }

    /// Check if currently valid
    pub fn is_currently_valid(&self) -> bool {
        let today = chrono::Utc::now().date_naive();
        self.is_valid_on(today)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_western_name() {
        let name = PersonName::new("Jane".to_string(), "Smith".to_string());
        // display_name() uses informal format (first given name only)
        assert_eq!(name.display_name(), "Jane");
        // full_name() uses formal format (all names)
        assert_eq!(name.full_name(), "Jane Smith");
    }

    #[test]
    fn test_spanish_name() {
        let name = PersonName::builder()
            .given_name("Pablo")
            .family_names(vec!["Ruiz", "Picasso"])
            .naming_convention(NamingConvention::Spanish)
            .build()
            .unwrap();

        assert_eq!(name.display(NameDisplayPolicy::Cultural), "Pablo Ruiz y Picasso");
    }

    #[test]
    fn test_mononym() {
        let name = PersonName::mononym("Suharto".to_string());
        assert_eq!(name.display_name(), "Suharto");
    }

    #[test]
    fn test_empty_name_fails() {
        let result = PersonName::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_title_validity() {
        let mut title = PersonTitle::new("MD", TitleType::Professional);
        title.awarded_date = Some(NaiveDate::from_ymd_opt(2020, 5, 15).unwrap());

        assert!(title.is_valid_on(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()));
        assert!(!title.is_valid_on(NaiveDate::from_ymd_opt(2019, 1, 1).unwrap()));

        title.revoked_date = Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
        assert!(!title.is_valid_on(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
    }
}
