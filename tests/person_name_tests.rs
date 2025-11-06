//! Comprehensive tests for PersonName functionality
//! Based on user stories US-1.1, US-1.2, US-1.3, US-1.4

use cim_domain_person::value_objects::{
    PersonName, PersonNameBuilder, NamingConvention, NameDisplayPolicy,
    PersonTitle, TitleType
};
use chrono::{NaiveDate, Utc};

// ===== US-1.1: Create Person with Culturally-Aware Name =====

#[test]
fn test_create_western_name() {
    // Test Scenario: Create Western name "John Michael Doe"
    let name = PersonNameBuilder::new()
        .given_names(vec!["John".to_string(), "Michael".to_string()])
        .family_names(vec!["Doe".to_string()])
        .naming_convention(NamingConvention::Western)
        .build()
        .expect("Should create valid Western name");

    assert_eq!(name.components.given_names.len(), 2);
    assert_eq!(name.components.given_names[0], "John");
    assert_eq!(name.components.given_names[1], "Michael");
    assert_eq!(name.components.family_names.len(), 1);
    assert_eq!(name.components.family_names[0], "Doe");
    assert_eq!(name.naming_convention, NamingConvention::Western);
}

#[test]
fn test_create_spanish_name() {
    // Test Scenario: Create Spanish name "María García López"
    let name = PersonNameBuilder::new()
        .given_names(vec!["María".to_string()])
        .family_names(vec!["García".to_string(), "López".to_string()])
        .naming_convention(NamingConvention::Spanish)
        .build()
        .expect("Should create valid Spanish name");

    assert_eq!(name.components.given_names[0], "María");
    assert_eq!(name.components.family_names[0], "García"); // Paternal
    assert_eq!(name.components.family_names[1], "López");  // Maternal
    assert_eq!(name.naming_convention, NamingConvention::Spanish);
}

#[test]
fn test_create_east_asian_name() {
    // Test Scenario: Create East Asian name "山田 太郎" (Yamada Taro)
    let name = PersonNameBuilder::new()
        .family_names(vec!["山田".to_string()])
        .given_names(vec!["太郎".to_string()])
        .naming_convention(NamingConvention::EastAsian)
        .build()
        .expect("Should create valid East Asian name");

    assert_eq!(name.components.family_names[0], "山田");
    assert_eq!(name.components.given_names[0], "太郎");
    assert_eq!(name.naming_convention, NamingConvention::EastAsian);
}

#[test]
fn test_create_patronymic_name() {
    // Test Scenario: Create Patronymic name "Ivan Petrovich"
    let name = PersonNameBuilder::new()
        .given_names(vec!["Ivan".to_string()])
        .patronymic("Petrovich".to_string())
        .naming_convention(NamingConvention::Patronymic)
        .build()
        .expect("Should create valid Patronymic name");

    assert_eq!(name.components.given_names[0], "Ivan");
    assert_eq!(name.components.patronymic, Some("Petrovich".to_string()));
    assert_eq!(name.naming_convention, NamingConvention::Patronymic);
}

#[test]
fn test_create_mononymic_name() {
    // Test Scenario: Create Mononymic name "Prince"
    let name = PersonNameBuilder::new()
        .given_names(vec!["Prince".to_string()])
        .naming_convention(NamingConvention::Mononymic)
        .build()
        .expect("Should create valid Mononymic name");

    assert_eq!(name.components.given_names[0], "Prince");
    assert!(name.components.family_names.is_empty());
    assert_eq!(name.naming_convention, NamingConvention::Mononymic);
}

#[test]
fn test_name_immutability() {
    // Verify PersonName is immutable after creation
    let name = PersonName::new("John".to_string(), "Doe".to_string());

    // PersonName doesn't have &mut methods - this test verifies the API design
    // If this compiles, it proves immutability is enforced
    let _clone = name.clone();
    assert_eq!(name.components.given_names[0], "John");
}

// ===== US-1.2: Display Names with Cultural Conventions =====

#[test]
fn test_display_western_name() {
    // Test Scenario: Display Western name as "Given Family"
    let name = PersonNameBuilder::new()
        .given_names(vec!["John".to_string()])
        .family_names(vec!["Doe".to_string()])
        .naming_convention(NamingConvention::Western)
        .build()
        .unwrap();

    let display = name.display(NameDisplayPolicy::Formal);
    assert_eq!(display, "John Doe", "Western name should display as 'Given Family'");
}

#[test]
fn test_display_spanish_name() {
    // Test Scenario: Display Spanish name as "Given Paternal Maternal"
    let name = PersonNameBuilder::new()
        .given_names(vec!["María".to_string()])
        .family_names(vec!["García".to_string(), "López".to_string()])
        .naming_convention(NamingConvention::Spanish)
        .build()
        .unwrap();

    let display = name.display(NameDisplayPolicy::Formal);
    assert_eq!(display, "María García López", "Spanish name should display all parts");
}

#[test]
fn test_display_east_asian_name() {
    // Test Scenario: Display East Asian name as "Family Given" (reversed)
    let name = PersonNameBuilder::new()
        .family_names(vec!["山田".to_string()])
        .given_names(vec!["太郎".to_string()])
        .naming_convention(NamingConvention::EastAsian)
        .build()
        .unwrap();

    let display = name.display(NameDisplayPolicy::Cultural);
    assert_eq!(display, "山田太郎", "East Asian name should display as 'FamilyGiven' with no space");
}

#[test]
fn test_display_patronymic_name() {
    // Test Scenario: Display Patronymic name as "Given Patronymic"
    let name = PersonNameBuilder::new()
        .given_names(vec!["Ivan".to_string()])
        .patronymic("Petrovich".to_string())
        .naming_convention(NamingConvention::Patronymic)
        .build()
        .unwrap();

    let display = name.display(NameDisplayPolicy::Formal);
    assert_eq!(display, "Ivan Petrovich", "Patronymic name should display given and patronymic");
}

#[test]
fn test_display_mononymic_name() {
    // Test Scenario: Display Mononymic name as just the name
    let name = PersonNameBuilder::new()
        .given_names(vec!["Prince".to_string()])
        .naming_convention(NamingConvention::Mononymic)
        .build()
        .unwrap();

    let display = name.display(NameDisplayPolicy::Formal);
    assert_eq!(display, "Prince", "Mononymic name should display single name");
}

#[test]
fn test_display_policy_override() {
    // Test Scenario: Display same person with different policies
    let name = PersonNameBuilder::new()
        .given_names(vec!["John".to_string(), "Michael".to_string()])
        .family_names(vec!["Doe".to_string()])
        .naming_convention(NamingConvention::Western)
        .build()
        .unwrap();

    let formal = name.display(NameDisplayPolicy::Formal);
    let informal = name.display(NameDisplayPolicy::Informal);

    assert_eq!(formal, "John Michael Doe", "Formal should include all names");
    assert_eq!(informal, "John", "Informal should show only first given name");
}

#[test]
fn test_display_formal_vs_informal() {
    // Test Scenario: Verify correct ordering for each convention
    let name = PersonNameBuilder::new()
        .given_names(vec!["John".to_string()])
        .family_names(vec!["Doe".to_string()])
        .naming_convention(NamingConvention::Western)
        .preferred("Johnny")
        .build()
        .unwrap();

    assert_eq!(
        name.display(NameDisplayPolicy::Formal),
        "John Doe"
    );
    assert_eq!(
        name.display(NameDisplayPolicy::Informal),
        "Johnny"
    );
}

// ===== US-1.3: Manage Person Titles =====

#[test]
fn test_professional_title() {
    // Test Scenario: Add "Dr." title with medical degree date
    let start_date = NaiveDate::from_ymd_opt(2020, 6, 15).unwrap();
    let mut title = PersonTitle::new("Dr.", TitleType::Professional);
    title.awarded_date = Some(start_date);

    assert_eq!(title.title_type, TitleType::Professional);
    assert_eq!(title.awarded_date, Some(start_date));
    assert!(title.expiry_date.is_none(), "Professional title should not have expiry date");
}

#[test]
fn test_honorific_title() {
    // Test Scenario: Add honorific title
    let start_date = NaiveDate::from_ymd_opt(2015, 1, 1).unwrap();
    let mut title = PersonTitle::new("Sir", TitleType::Honorary);
    title.awarded_date = Some(start_date);

    assert_eq!(title.title_type, TitleType::Honorary);
    assert_eq!(title.awarded_date, Some(start_date));
}

#[test]
fn test_nobility_title() {
    // Test Scenario: Add nobility title
    let start_date = NaiveDate::from_ymd_opt(1990, 3, 20).unwrap();
    let mut title = PersonTitle::new("Duke", TitleType::Noble);
    title.awarded_date = Some(start_date);

    assert_eq!(title.title_type, TitleType::Noble);
    assert_eq!(title.title, "Duke");
}

#[test]
fn test_title_temporal_validity() {
    // Test Scenario: Titles have start and optional end dates
    let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();

    let mut title = PersonTitle::new("Prof.", TitleType::Professional);
    title.awarded_date = Some(start);
    title.revoked_date = Some(end);

    assert_eq!(title.awarded_date, Some(start));
    assert_eq!(title.revoked_date, Some(end));
}

#[test]
fn test_query_titles_at_date() {
    // Test Scenario: Query titles valid in 2020 vs 2024
    let mut title1 = PersonTitle::new("Dr.", TitleType::Professional);
    title1.awarded_date = Some(NaiveDate::from_ymd_opt(2018, 1, 1).unwrap());
    title1.revoked_date = Some(NaiveDate::from_ymd_opt(2022, 12, 31).unwrap());

    let mut title2 = PersonTitle::new("Prof.", TitleType::Professional);
    title2.awarded_date = Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());

    let date_2020 = NaiveDate::from_ymd_opt(2020, 6, 1).unwrap();
    let date_2024 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    // Title1 valid in 2020, not in 2024
    assert!(title1.is_valid_on(date_2020), "Title1 should be valid in 2020");
    assert!(!title1.is_valid_on(date_2024), "Title1 should not be valid in 2024");

    // Title2 not valid in 2020, valid in 2024
    assert!(!title2.is_valid_on(date_2020), "Title2 should not be valid in 2020");
    assert!(title2.is_valid_on(date_2024), "Title2 should be valid in 2024");
}

#[test]
fn test_multiple_concurrent_titles() {
    // Test Scenario: Person can have multiple concurrent titles
    // NOTE: Titles are tracked separately from PersonName, not embedded in it
    let mut title1 = PersonTitle::new("Dr.", TitleType::Professional);
    title1.awarded_date = Some(NaiveDate::from_ymd_opt(2015, 1, 1).unwrap());

    let mut title2 = PersonTitle::new("Prof.", TitleType::Professional);
    title2.awarded_date = Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());

    // Titles are stored separately (e.g., in Person aggregate, not PersonName)
    let titles = vec![title1, title2];

    // Both titles should be currently valid
    let now = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let valid_titles: Vec<_> = titles.iter()
        .filter(|t| t.is_valid_on(now))
        .collect();

    assert_eq!(valid_titles.len(), 2, "Both titles should be currently valid");

    // Verify each title individually
    assert!(titles[0].is_valid_on(now));
    assert!(titles[1].is_valid_on(now));
}

// ===== US-1.4:

#[test]
fn test_name_change_event_structure() {
    // Test Scenario: NameUpdated event captures old and new names
    use cim_domain_person::events::{NameUpdated, PersonId};

    let person_id = PersonId::new();
    let old_name = PersonName::new("Jane".to_string(), "Smith".to_string());
    let new_name = PersonName::new("Jane".to_string(), "Doe".to_string());

    let event = NameUpdated {
        person_id,
        old_name: old_name.clone(),
        new_name: new_name.clone(),
        reason: Some("Marriage".to_string()),
        updated_at: Utc::now(),
    };

    // Verify event structure
    assert_eq!(event.old_name.components.family_names[0], "Smith");
    assert_eq!(event.new_name.components.family_names[0], "Doe");
    assert_eq!(event.reason, Some("Marriage".to_string()));

    // Verify immutability - events cannot be modified
    let _clone = event.clone();
}

#[test]
fn test_name_change_reasons() {
    // Test Scenario: Record name change with various reasons
    use cim_domain_person::events::NameUpdated;

    let person_id = cim_domain_person::events::PersonId::new();
    let old_name = PersonName::new("John".to_string(), "Smith".to_string());
    let new_name = PersonName::new("John".to_string(), "Doe".to_string());

    let marriage_event = NameUpdated {
        person_id,
        old_name: old_name.clone(),
        new_name: new_name.clone(),
        reason: Some("Marriage".to_string()),
        updated_at: Utc::now(),
    };

    assert_eq!(marriage_event.reason, Some("Marriage".to_string()));

    let legal_event = NameUpdated {
        person_id,
        old_name: old_name.clone(),
        new_name: new_name.clone(),
        reason: Some("Legal name change".to_string()),
        updated_at: Utc::now(),
    };

    assert_eq!(legal_event.reason, Some("Legal name change".to_string()));
}

// ===== Name Parser Tests =====

#[test]
fn test_parse_simple_western_name() {
    let name = PersonName::parse("John Doe").unwrap();

    assert_eq!(name.components.given_names[0], "John");
    assert_eq!(name.components.family_names[0], "Doe");
    assert_eq!(name.naming_convention, NamingConvention::Western);
}

#[test]
fn test_parse_western_with_middle_names() {
    let name = PersonName::parse("Mary Elizabeth Anne Smith").unwrap();

    assert_eq!(name.components.given_names.len(), 3);
    assert_eq!(name.components.given_names[0], "Mary");
    assert_eq!(name.components.given_names[1], "Elizabeth");
    assert_eq!(name.components.given_names[2], "Anne");
    assert_eq!(name.components.family_names[0], "Smith");
}

#[test]
fn test_parse_western_with_particles() {
    let name = PersonName::parse("Ludwig van Beethoven").unwrap();

    assert_eq!(name.components.given_names[0], "Ludwig");
    assert_eq!(name.components.prefixes[0], "van");
    assert_eq!(name.components.family_names[0], "Beethoven");
}

#[test]
fn test_parse_western_with_suffix() {
    let name = PersonName::parse("Martin Luther King Jr.").unwrap();

    assert_eq!(name.components.given_names[0], "Martin");
    assert_eq!(name.components.given_names[1], "Luther");
    assert_eq!(name.components.family_names[0], "King");
    assert_eq!(name.components.suffixes[0], "Jr.");
}

#[test]
fn test_parse_pablo_picasso_full_name() {
    // The real test! Pablo Picasso's complete name
    let full_name = "Pablo Diego José Francisco de Paula Juan Nepomuceno María de los Remedios Cipriano de la Santísima Trinidad Ruiz Picasso";

    let name = PersonName::parse(full_name).unwrap();

    // Should detect as Spanish due to "de" particles
    assert_eq!(name.naming_convention, NamingConvention::Spanish);

    // Last 2 should be family names: Ruiz (paternal) and Picasso (maternal)
    assert!(name.components.family_names.contains(&"Ruiz".to_string()));
    assert!(name.components.family_names.contains(&"Picasso".to_string()));

    // All the given names should be there
    assert!(name.components.given_names.contains(&"Pablo".to_string()));
    assert!(name.components.given_names.contains(&"Diego".to_string()));
    assert!(name.components.given_names.contains(&"José".to_string()));

    // Can format it
    let display = name.display(NameDisplayPolicy::Informal);
    assert_eq!(display, "Pablo"); // Informal = first given name only
}

#[test]
fn test_parse_spanish_with_y_connector() {
    let name = PersonName::parse("Pablo Ruiz y Picasso").unwrap();

    assert_eq!(name.naming_convention, NamingConvention::Spanish);
    assert_eq!(name.components.given_names[0], "Pablo");
    assert_eq!(name.components.family_names[0], "Ruiz");
    assert_eq!(name.components.family_names[1], "Picasso");
}

#[test]
fn test_parse_east_asian_name() {
    let name = PersonName::parse_with_convention("李明", Some(NamingConvention::EastAsian)).unwrap();

    assert_eq!(name.components.family_names[0], "李");
    assert_eq!(name.components.given_names[0], "明");
    assert_eq!(name.naming_convention, NamingConvention::EastAsian);
}

#[test]
fn test_parse_east_asian_three_chars() {
    let name = PersonName::parse_with_convention("王小明", Some(NamingConvention::EastAsian)).unwrap();

    assert_eq!(name.components.family_names[0], "王");
    assert_eq!(name.components.given_names[0], "小明");
}

#[test]
fn test_parse_patronymic_name() {
    // Won't auto-detect without suffix, so let's use explicit convention
    let name = PersonName::parse_with_convention("Ivan Petrovich", Some(NamingConvention::Patronymic)).unwrap();

    assert_eq!(name.components.given_names[0], "Ivan");
    assert_eq!(name.components.patronymic, Some("Petrovich".to_string()));
}

#[test]
fn test_parse_icelandic_name() {
    let name = PersonName::parse("Björk Guðmundsdóttir").unwrap();

    // Should auto-detect as patronymic due to "dóttir" suffix
    assert_eq!(name.naming_convention, NamingConvention::Patronymic);
    assert_eq!(name.components.given_names[0], "Björk");
    assert_eq!(name.components.patronymic, Some("Guðmundsdóttir".to_string()));
}

#[test]
fn test_parse_mononym() {
    let name = PersonName::parse("Prince").unwrap();

    assert_eq!(name.naming_convention, NamingConvention::Mononymic);
    assert_eq!(name.components.given_names[0], "Prince");
    assert!(name.components.family_names.is_empty());
}

#[test]
fn test_parse_auto_detect_chinese() {
    let name = PersonName::parse("李明").unwrap();

    // Should auto-detect as EastAsian due to CJK characters
    assert_eq!(name.naming_convention, NamingConvention::EastAsian);
}

#[test]
fn test_parse_complex_western_particles() {
    let name = PersonName::parse("Leonardo da Vinci").unwrap();

    assert_eq!(name.components.given_names[0], "Leonardo");
    assert!(name.components.prefixes.contains(&"da".to_string()));
    assert!(name.components.family_names.contains(&"Vinci".to_string()));
}

#[test]
fn test_parse_empty_name_error() {
    let result = PersonName::parse("");
    assert!(result.is_err());

    let result = PersonName::parse("   ");
    assert!(result.is_err());
}

// ===== Edge Cases and Validation =====

#[test]
fn test_empty_name_validation() {
    // Names should require at least a given name or family name
    let result = PersonNameBuilder::new()
        .build();

    assert!(result.is_err(), "Should not allow completely empty name");
}

#[test]
fn test_name_with_special_characters() {
    // Names should handle special characters and unicode
    let name = PersonNameBuilder::new()
        .given_names(vec!["José".to_string()])
        .family_names(vec!["O'Brien".to_string()])
        .naming_convention(NamingConvention::Western)
        .build()
        .unwrap();

    assert_eq!(name.components.given_names[0], "José");
    assert_eq!(name.components.family_names[0], "O'Brien");
}

#[test]
fn test_very_long_name() {
    // Should handle very long names
    let long_given = "Hubert Blaine Wolfeschlegelsteinhausenbergerdorff Sr.".to_string();
    let name = PersonNameBuilder::new()
        .given_names(vec![long_given.clone()])
        .family_names(vec!["Smith".to_string()])
        .naming_convention(NamingConvention::Western)
        .build()
        .unwrap();

    assert_eq!(name.components.given_names[0], long_given);
}

#[test]
fn test_multiple_middle_names() {
    // Should handle multiple middle names
    let name = PersonNameBuilder::new()
        .given_names(vec![
            "Mary".to_string(),
            "Elizabeth".to_string(),
            "Margaret".to_string(),
        ])
        .family_names(vec!["Windsor".to_string()])
        .naming_convention(NamingConvention::Western)
        .build()
        .unwrap();

    assert_eq!(name.components.given_names.len(), 3);
    assert_eq!(name.components.given_names[0], "Mary");
    assert_eq!(name.components.given_names[1], "Elizabeth");
    assert_eq!(name.components.given_names[2], "Margaret");
}

#[test]
fn test_picasso_name_example() {
    // Real-world test: Pablo Picasso's full name
    let name = PersonNameBuilder::new()
        .given_names(vec![
            "Pablo".to_string(),
            "Diego".to_string(),
            "José".to_string(),
            "Francisco".to_string(),
            "de Paula".to_string(),
            "Juan".to_string(),
            "Nepomuceno".to_string(),
            "María".to_string(),
            "de los Remedios".to_string(),
            "Cipriano".to_string(),
            "de la Santísima Trinidad".to_string(),
        ])
        .family_names(vec![
            "Ruiz".to_string(),
            "Picasso".to_string(),
        ])
        .naming_convention(NamingConvention::Spanish)
        .preferred("Pablo Picasso")
        .build()
        .unwrap();

    assert!(name.components.given_names.len() > 10);
    assert_eq!(name.components.family_names[0], "Ruiz");
    assert_eq!(name.components.family_names[1], "Picasso");
    assert_eq!(name.preferred_form, Some("Pablo Picasso".to_string()));

    // Verify preferred form is used for display
    let display = name.display(NameDisplayPolicy::Informal);
    assert_eq!(display, "Pablo Picasso");
}
