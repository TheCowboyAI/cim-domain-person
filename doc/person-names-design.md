# Person Names: A Comprehensive Design

## Problem Statement

Human names are culturally complex and cannot be adequately represented as simple "first name" + "last name" strings. Names are:

1. **Structured collections of components** with semantic meaning
2. **Temporally dynamic** (titles/honorifics can be awarded, revoked, or retired)
3. **Culturally variable** (different conventions across cultures)
4. **Context-dependent** (formal vs. informal, legal vs. preferred)
5. **Never empty** (minimum: single character/symbol - "what noise do I make to get your attention")

## Real-World Examples

### Example 1: Pablo Picasso (Spanish Heritage)
**Full Name**: Pablo Diego José Francisco de Paula Juan Nepomuceno María de los Remedios Cipriano de la Santísima Trinidad Ruiz y Picasso

**Structure**:
- **Given Name**: Pablo
- **Middle Names**: Diego, José, Francisco, de Paula, Juan, Nepomuceno, María de los Remedios, Cipriano de la Santísima Trinidad
- **Paternal Surname**: Ruiz
- **Maternal Surname**: Picasso
- **Common Usage**: Pablo Picasso

**Notes**: Spanish naming tradition uses both paternal and maternal surnames, connected with "y". Multiple middle names are common, often including religious references.

### Example 2: English Royalty
**Full Name**: His Royal Highness Prince Charles Philip Arthur George, Prince of Wales, Duke of Cornwall, Duke of Rothesay, Earl of Chester, Earl of Carrick, Baron of Renfrew, Lord of the Isles, Prince and Great Steward of Scotland, KG, KT, GCB, OM, AK, QSO, PC, ADC

**Structure**:
- **Titles** (temporal): His Royal Highness, Prince of Wales, Duke of Cornwall, etc.
- **Given Names**: Charles Philip Arthur George
- **Post-nominal Honors** (temporal): KG, KT, GCB, OM, AK, QSO, PC, ADC

**Notes**: Multiple hereditary titles, earned honors with temporal constraints (awarded, can be revoked).

### Example 3: Academic/Professional
**Name**: Dr. Jane Elizabeth Smith-Johnson, MD, PhD, FACS

**Structure**:
- **Title** (temporal): Dr. (awarded upon degree completion)
- **Given Name**: Jane
- **Middle Name**: Elizabeth
- **Family Names**: Smith-Johnson (hyphenated)
- **Post-nominal Credentials** (temporal): MD, PhD, FACS (awarded, can be revoked by licensing boards)

### Example 4: Cultural Variations

**Icelandic**: Björk Guðmundsdóttir
- **Given Name**: Björk
- **Patronymic**: Guðmundsdóttir (daughter of Guðmund)
- **Note**: No hereditary surname

**Indonesian**: Suharto
- **Single Name**: Suharto
- **Note**: Mononyms are common

**Chinese**: 李明 (Lǐ Míng)
- **Family Name**: 李 (Lǐ) - comes first
- **Given Name**: 明 (Míng)

## Domain Model Design

### Core Principle: Name as Immutable Value Object

```rust
pub struct PersonName {
    // Core components (immutable)
    components: NameComponents,

    // Preferred/display variants
    preferred_form: Option<String>,

    // Cultural context
    naming_convention: NamingConvention,
}
```

### Name Components

```rust
pub struct NameComponents {
    // Primary identity (REQUIRED - minimum one character)
    given_names: Vec<String>,        // First, middle names
    family_names: Vec<String>,       // Last name(s), can be multiple

    // Additional identity
    patronymic: Option<String>,      // Icelandic, Russian
    matronymic: Option<String>,      // Some cultures

    // Particles (cultural)
    prefixes: Vec<String>,           // "de", "van", "von", "al"
    suffixes: Vec<String>,           // "Jr.", "Sr.", "III"
}
```

### Temporal Titles and Honorifics (SEPARATE from Name)

**Important**: Titles and honorifics are NOT part of the immutable name - they are temporal attributes tracked separately.

```rust
pub struct PersonTitle {
    title: String,                   // "Dr.", "Sir", "Lord", "HRH"
    title_type: TitleType,           // Academic, Noble, Professional, Honorary
    awarded_date: Option<NaiveDate>, // When awarded
    revoked_date: Option<NaiveDate>, // If revoked
    expiry_date: Option<NaiveDate>,  // If time-limited
    issuing_authority: Option<String>,
}

pub enum TitleType {
    Academic,        // Dr., Prof.
    Professional,    // MD, PhD, Esq.
    Noble,           // Sir, Lord, Duke
    Honorary,        // Hon., Rev.
    Military,        // Col., Gen.
    Religious,       // Fr., Rabbi
}

// Stored in Person aggregate as temporal collection
pub struct Person {
    // ...
    core_identity: CoreIdentity {
        legal_name: PersonName,      // Immutable components
        // ...
    },
    titles: Vec<PersonTitle>,        // Temporal, can change
}
```

### Display Policies (Interpretation)

Different contexts require different name representations:

```rust
pub enum NameDisplayPolicy {
    Formal,           // Full name with titles
    Informal,         // Preferred or given name only
    Legal,            // Exact legal name components
    Alphabetical,     // Surname first for sorting
    Cultural,         // Respect cultural conventions
}

impl PersonName {
    pub fn display(&self, policy: NameDisplayPolicy) -> String {
        match policy {
            NameDisplayPolicy::Formal => self.format_formal(),
            NameDisplayPolicy::Informal => self.format_informal(),
            // ...
        }
    }
}
```

### Validation Rules

1. **Minimum Requirement**: At least ONE character in either given_names OR family_names
2. **Character Validation**: Support Unicode (全角文字, Ελληνικά, العربية)
3. **No Empty Strings**: Each component must be non-empty if present
4. **Immutability**: Name components don't change (person changes name → new Name value object)

## Implementation Strategy

### Phase 1: Enhanced PersonName Value Object
- [x] Design comprehensive structure
- [ ] Implement `NameComponents` struct
- [ ] Add validation (minimum 1 character rule)
- [ ] Support multiple given/family names
- [ ] Add cultural particles (de, van, etc.)

### Phase 2: Temporal Title Tracking
- [ ] Create `PersonTitle` value object
- [ ] Add `titles: Vec<PersonTitle>` to Person aggregate
- [ ] Implement commands: `AwardTitle`, `RevokeTitle`, `ExpireTitle`
- [ ] Add events: `TitleAwarded`, `TitleRevoked`, `TitleExpired`

### Phase 3: Display Policies
- [ ] Implement `NameDisplayPolicy` enum
- [ ] Create formatting functions for each policy
- [ ] Add cultural convention support
- [ ] Create helper methods for common use cases

### Phase 4: Name Change Events
- [ ] `NameChanged` event (replaces entire PersonName)
- [ ] `PreferredNameSet` event (changes display preference)
- [ ] Track name history (optional, for audit)

## API Examples

### Creating Names

```rust
// Simple Western name
let name = PersonName::simple("Jane", "Smith");

// Spanish heritage (multiple middle names, both surnames)
let picasso = PersonName::builder()
    .given_name("Pablo")
    .middle_names(vec![
        "Diego", "José", "Francisco", "de Paula",
        "Juan", "Nepomuceno", "María de los Remedios",
        "Cipriano de la Santísima Trinidad"
    ])
    .family_names(vec!["Ruiz", "Picasso"])
    .naming_convention(NamingConvention::Spanish)
    .build()?;

// Single name (Indonesian)
let suharto = PersonName::mononym("Suharto");

// Chinese (family name first)
let chinese = PersonName::builder()
    .family_name("李")
    .given_name("明")
    .naming_convention(NamingConvention::Chinese)
    .build()?;
```

### Adding Titles (Temporal)

```rust
// Award academic title
let title = PersonTitle::new("Dr.")
    .title_type(TitleType::Academic)
    .awarded_on(NaiveDate::from_ymd(2020, 5, 15))
    .issuing_authority("Stanford University");

person.award_title(title)?;

// Award professional credential
let md = PersonTitle::new("MD")
    .title_type(TitleType::Professional)
    .awarded_on(NaiveDate::from_ymd(2020, 5, 15))
    .issuing_authority("Medical Board of California");

person.award_title(md)?;

// Revoke title (e.g., medical license)
person.revoke_title("MD", NaiveDate::from_ymd(2025, 1, 1), "License revoked")?;
```

### Display with Policies

```rust
let name = /* ... */;

// Formal: "Dr. Jane Elizabeth Smith, MD, PhD"
println!("{}", name.display(NameDisplayPolicy::Formal));

// Informal: "Jane" or preferred name
println!("{}", name.display(NameDisplayPolicy::Informal));

// Legal: "Smith, Jane Elizabeth"
println!("{}", name.display(NameDisplayPolicy::Legal));

// Cultural (Spanish): "Pablo Ruiz y Picasso"
println!("{}", picasso.display(NameDisplayPolicy::Cultural));
```

## Benefits

1. **Cultural Respect**: Accurately represents names from all cultures
2. **Temporal Accuracy**: Tracks when titles/honors were awarded/revoked
3. **Flexibility**: Multiple display formats for different contexts
4. **Immutability**: Name components are value objects (functional)
5. **Validation**: Enforces minimum requirements (not empty)
6. **Audit Trail**: Name changes are events in event store
7. **Policy-Based**: Interpretation separated from storage

## Migration Path

1. Keep existing simple `PersonName::new(given, family)` constructor
2. Add new builder pattern for complex names
3. Gradually migrate existing code to use display policies
4. Add title tracking as new feature (backward compatible)

## References

- [Falsehoods Programmers Believe About Names](https://www.kalzumeus.com/2010/06/17/falsehoods-programmers-believe-about-names/)
- W3C Personal Names Around the World
- Unicode Common Locale Data Repository (CLDR)
- ISO/IEC 11179 Naming Conventions
