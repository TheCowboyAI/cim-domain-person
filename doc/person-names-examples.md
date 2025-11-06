# PersonName: Real-World Usage Examples

This document provides comprehensive examples of using the new PersonName value object with real people's names from various cultures.

## Quick Start: Simple Western Names

```rust
use cim_domain_person::value_objects::PersonName;

// Simple name
let name = PersonName::new("Jane".to_string(), "Smith".to_string());
println!("Display: {}", name.display_name());  // "Jane Smith"
println!("Full: {}", name.full_name());        // "Jane Smith"
```

## Real-World Example 1: Pablo Picasso (Spanish Heritage)

**Full Name**: Pablo Diego José Francisco de Paula Juan Nepomuceno María de los Remedios Cipriano de la Santísima Trinidad Ruiz y Picasso

```rust
use cim_domain_person::value_objects::{PersonName, NamingConvention, NameDisplayPolicy};

let picasso = PersonName::builder()
    .given_name("Pablo")
    .given_names(vec![
        "Diego",
        "José",
        "Francisco",
        "de Paula",
        "Juan",
        "Nepomuceno",
        "María de los Remedios",
        "Cipriano de la Santísima Trinidad"
    ])
    .family_name("Ruiz")        // Paternal surname
    .family_name("Picasso")     // Maternal surname
    .naming_convention(NamingConvention::Spanish)
    .preferred("Pablo Picasso") // What he's commonly called
    .build()
    .unwrap();

// Display in different contexts
println!("{}", picasso.display(NameDisplayPolicy::Cultural));
// Output: "Pablo Ruiz y Picasso"

println!("{}", picasso.display(NameDisplayPolicy::Informal));
// Output: "Pablo Picasso" (preferred name)

println!("{}", picasso.display(NameDisplayPolicy::Formal));
// Output: "Pablo Diego José Francisco de Paula Juan Nepomuceno María de los Remedios Cipriano de la Santísima Trinidad Ruiz Picasso"
```

## Real-World Example 2: English Royalty with Titles

**Name**: Charles Philip Arthur George, Prince of Wales

**Note**: Titles are tracked separately as temporal attributes!

```rust
use cim_domain_person::value_objects::{PersonName, PersonTitle, TitleType};
use chrono::NaiveDate;

// The name itself (immutable)
let charles_name = PersonName::builder()
    .given_name("Charles")
    .given_names(vec!["Philip", "Arthur", "George"])
    .family_name("Windsor")
    .naming_convention(NamingConvention::Western)
    .build()
    .unwrap();

// Titles are temporal (can be awarded/revoked)
let prince_of_wales = PersonTitle {
    title: "Prince of Wales".to_string(),
    title_type: TitleType::Noble,
    awarded_date: Some(NaiveDate::from_ymd_opt(1958, 7, 26).unwrap()),
    revoked_date: None,  // Still holds the title
    expiry_date: None,
    issuing_authority: Some("British Crown".to_string()),
    revocation_reason: None,
};

let hrh = PersonTitle {
    title: "His Royal Highness".to_string(),
    title_type: TitleType::Honorary,
    awarded_date: Some(NaiveDate::from_ymd_opt(1948, 11, 14).unwrap()),
    revoked_date: None,
    expiry_date: None,
    issuing_authority: Some("British Crown".to_string()),
    revocation_reason: None,
};

// In the Person aggregate, these would be stored as:
// person.titles = vec![prince_of_wales, hrh, ...];

// Display the name
println!("{}", charles_name.display_name());  // "Charles Philip Arthur George Windsor"
```

## Real-World Example 3: Academic/Professional Credentials

**Name**: Dr. Jane Elizabeth Smith-Johnson, MD, PhD, FACS

```rust
use cim_domain_person::value_objects::{PersonName, PersonTitle, TitleType};
use chrono::NaiveDate;

// The name (note hyphenated surname)
let name = PersonName::builder()
    .given_name("Jane")
    .given_names(vec!["Elizabeth"])
    .family_names(vec!["Smith-Johnson"])  // Hyphenated - single family name
    .naming_convention(NamingConvention::Western)
    .build()
    .unwrap();

// Titles are temporal (awarded upon degree completion, can be revoked)
let dr_title = PersonTitle {
    title: "Dr.".to_string(),
    title_type: TitleType::Academic,
    awarded_date: Some(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap()),
    revoked_date: None,
    expiry_date: None,
    issuing_authority: Some("Stanford University".to_string()),
    revocation_reason: None,
};

let md_credential = PersonTitle {
    title: "MD".to_string(),
    title_type: TitleType::Professional,
    awarded_date: Some(NaiveDate::from_ymd_opt(2015, 5, 15).unwrap()),
    revoked_date: None,
    expiry_date: None,
    issuing_authority: Some("Medical Board of California".to_string()),
    revocation_reason: None,
};

let phd_credential = PersonTitle {
    title: "PhD".to_string(),
    title_type: TitleType::Professional,
    awarded_date: Some(NaiveDate::from_ymd_opt(2018, 12, 10).unwrap()),
    revoked_date: None,
    expiry_date: None,
    issuing_authority: Some("Stanford University".to_string()),
    revocation_reason: None,
};

let facs = PersonTitle {
    title: "FACS".to_string(),
    title_type: TitleType::Professional,
    awarded_date: Some(NaiveDate::from_ymd_opt(2020, 3, 1).unwrap()),
    revoked_date: None,
    expiry_date: None,
    issuing_authority: Some("American College of Surgeons".to_string()),
    revocation_reason: None,
};

// Example of title revocation (e.g., medical license revoked)
let mut revoked_md = md_credential.clone();
revoked_md.revoked_date = Some(NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
revoked_md.revocation_reason = Some("Medical malpractice".to_string());

// Check if title is still valid
assert!(md_credential.is_currently_valid());
assert!(!revoked_md.is_currently_valid());
```

## Real-World Example 4: Icelandic Patronymic

**Name**: Björk Guðmundsdóttir

```rust
use cim_domain_person::value_objects::{PersonName, NamingConvention, NameDisplayPolicy};

let bjork = PersonName::builder()
    .given_name("Björk")
    .patronymic("Guðmundsdóttir")  // "daughter of Guðmund"
    .naming_convention(NamingConvention::Patronymic)
    .build()
    .unwrap();

println!("{}", bjork.display(NameDisplayPolicy::Cultural));
// Output: "Björk Guðmundsdóttir"

println!("{}", bjork.display(NameDisplayPolicy::Informal));
// Output: "Björk"
```

## Real-World Example 5: Chinese Name

**Name**: 李明 (Lǐ Míng)

```rust
use cim_domain_person::value_objects::{PersonName, NamingConvention, NameDisplayPolicy};

let chinese_name = PersonName::builder()
    .family_name("李")      // Family name comes first
    .given_name("明")
    .naming_convention(NamingConvention::EastAsian)
    .build()
    .unwrap();

println!("{}", chinese_name.display(NameDisplayPolicy::Cultural));
// Output: "李明" (family name first, no space)

println!("{}", chinese_name.display(NameDisplayPolicy::Western));
// Output: "明 李" (given name first for Western audiences)
```

## Real-World Example 6: Indonesian Mononym

**Name**: Suharto

```rust
use cim_domain_person::value_objects::PersonName;

let suharto = PersonName::mononym("Suharto".to_string());

println!("{}", suharto.display_name());  // "Suharto"
```

## Real-World Example 7: Dutch Name with Particle

**Name**: Vincent van Gogh

```rust
use cim_domain_person::value_objects::{PersonName, NameDisplayPolicy};

let van_gogh = PersonName::builder()
    .given_name("Vincent")
    .given_names(vec!["Willem"])
    .prefix("van")          // Name particle
    .family_name("Gogh")
    .build()
    .unwrap();

println!("{}", van_gogh.display(NameDisplayPolicy::Formal));
// Output: "Vincent Willem van Gogh"

println!("{}", van_gogh.display(NameDisplayPolicy::Alphabetical));
// Output: "van Gogh, V."
```

## Real-World Example 8: Junior/Senior Suffixes

**Name**: Martin Luther King Jr.

```rust
use cim_domain_person::value_objects::PersonName;

let mlk = PersonName::builder()
    .given_name("Martin")
    .given_names(vec!["Luther"])
    .family_name("King")
    .suffix("Jr.")
    .build()
    .unwrap();

println!("{}", mlk.full_name());
// Output: "Martin Luther King Jr."
```

## Common Patterns

### Pattern 1: Creating a Person with a Name

```rust
use cim_domain_person::aggregate::Person;
use cim_domain_person::value_objects::PersonName;

let name = PersonName::new("Jane".to_string(), "Smith".to_string());
let person_id = PersonId::new();
let person = Person::new(person_id, name);
```

### Pattern 2: Updating Display Policy in Views

```rust
use cim_domain_person::value_objects::NameDisplayPolicy;

// For a formal letter
let formal = person.core_identity.legal_name.display(NameDisplayPolicy::Formal);

// For a friendly email
let informal = person.core_identity.legal_name.display(NameDisplayPolicy::Informal);

// For alphabetical sorting
let alphabetical = person.core_identity.legal_name.display(NameDisplayPolicy::Alphabetical);
```

### Pattern 3: Name Change Event

When someone legally changes their name, create a NEW PersonName value object:

```rust
// Old name
let old_name = PersonName::new("Jane".to_string(), "Smith".to_string());

// Marriage - new surname
let new_name = PersonName::builder()
    .given_name("Jane")
    .family_name("Johnson")
    .build()
    .unwrap();

// This would trigger a NameChanged event in the aggregate
// person.change_name(new_name)?;
```

## Validation Rules

### Minimum Requirement: At Least One Character

```rust
// This is VALID - single character name
let valid = PersonName::mononym("李".to_string());

// This is INVALID - empty name
let invalid = PersonName::builder().build();
// Error: "Name must have at least one given name or family name"
```

### No Empty Strings in Components

```rust
// This is INVALID - empty string in given_names
let invalid = PersonName::builder()
    .given_name("")  // Empty!
    .family_name("Smith")
    .build();
// Error: "Given names cannot be empty strings"
```

## Integration with Person Aggregate

```rust
use cim_domain_person::aggregate::Person;
use cim_domain_person::value_objects::{PersonName, PersonTitle, TitleType};
use chrono::NaiveDate;

// Create person with complex name
let name = PersonName::builder()
    .given_name("Jane")
    .given_names(vec!["Elizabeth"])
    .family_name("Smith-Johnson")
    .build()
    .unwrap();

let mut person = Person::new(PersonId::new(), name);

// Award a title (this would be a command/event in the actual system)
// person.award_title(PersonTitle {
//     title: "Dr.".to_string(),
//     title_type: TitleType::Academic,
//     awarded_date: Some(NaiveDate::from_ymd_opt(2020, 5, 15).unwrap()),
//     ...
// })?;
```

## Migration Strategy

### For Existing Code

The old simple constructor still works:

```rust
// Old code (still works!)
let name = PersonName::new("Jane".to_string(), "Smith".to_string());

// New code (more expressive)
let name = PersonName::builder()
    .given_name("Jane")
    .family_name("Smith")
    .build()
    .unwrap();
```

### Gradually Adopt Display Policies

```rust
// Old way
let display = format!("{} {}", name.given_name, name.family_name);

// New way (more flexible)
let display = name.display(NameDisplayPolicy::Formal);
```

## References

- See `doc/person-names-design.md` for full design rationale
- See `src/value_objects/person_name.rs` for implementation
- [Falsehoods Programmers Believe About Names](https://www.kalzumeus.com/2010/06/17/falsehoods-programmers-believe-about-names/)
