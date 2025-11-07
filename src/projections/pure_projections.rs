//! Pure functional projection functions
//!
//! This module contains pure functions for projecting events into read models.
//! Following FRP/CT principles, these functions are:
//! - Pure: same inputs always produce same outputs
//! - Side-effect free: no I/O, no mutation of external state
//! - Composable: can be chained and tested in isolation
//!
//! The pattern is: (CurrentState, Event) → NewState
//!
//! Infrastructure adapters are responsible for:
//! - Loading current state from storage
//! - Calling these pure functions
//! - Persisting the resulting state

use crate::events::PersonEvent;
use crate::projections::{PersonSummary, PersonSearchResult, TimelineEntry};

/// Project a PersonEvent into PersonSummary state
///
/// This is a pure function: given the current summary and an event,
/// it returns the new summary state. No side effects.
///
/// # Category Theory
/// This function is a morphism in the category of read models:
/// PersonSummary × PersonEvent → PersonSummary
pub fn project_person_summary(
    current: Option<PersonSummary>,
    event: &PersonEvent,
) -> Option<PersonSummary> {
    match event {
        PersonEvent::PersonCreated(e) => {
            // Create new summary
            Some(PersonSummary {
                person_id: e.person_id,
                name: e.name.display_name(),
                primary_email: None,
                primary_phone: None,
                current_employer: None,
                current_role: None,
                location: None,
                skills_count: 0,
                component_count: 0,
                last_updated: e.created_at,
            })
        }

        PersonEvent::NameUpdated(e) => {
            // Update name in existing summary
            current.map(|mut summary| {
                summary.name = e.new_name.display_name();
                summary.last_updated = e.updated_at;
                summary
            })
        }

        PersonEvent::PersonDeactivated(_) => {
            // Remove summary when person is deactivated
            None
        }

        PersonEvent::PersonMergedInto(_) => {
            // Remove summary when person is merged
            None
        }

        PersonEvent::PersonReactivated(e) => {
            // Restore or update summary on reactivation
            current.map(|mut summary| {
                summary.last_updated = e.reactivated_at;
                summary
            })
        }

        PersonEvent::AttributeRecorded(e) => {
            // Update last_updated timestamp for attribute changes
            current.map(|mut summary| {
                summary.last_updated = e.recorded_at;
                summary
            })
        }

        PersonEvent::AttributeUpdated(e) => {
            current.map(|mut summary| {
                summary.last_updated = e.updated_at;
                summary
            })
        }

        PersonEvent::AttributeInvalidated(e) => {
            current.map(|mut summary| {
                summary.last_updated = e.invalidated_at;
                summary
            })
        }

        PersonEvent::BirthDateSet(e) => {
            current.map(|mut summary| {
                summary.last_updated = e.set_at;
                summary
            })
        }

        PersonEvent::DeathRecorded(e) => {
            // Update last_updated on death recorded
            current.map(|mut summary| {
                summary.last_updated = e.recorded_at;
                summary
            })
        }

        PersonEvent::PersonUpdated(e) => {
            // Generic update
            current.map(|mut summary| {
                summary.last_updated = e.updated_at;
                summary
            })
        }
    }
}

/// Project a PersonEvent into PersonSearchResult
///
/// This creates searchable index entries from events.
/// Returns Some(result) if the event should be indexed, None otherwise.
pub fn project_person_search(
    current: Option<PersonSearchResult>,
    event: &PersonEvent,
) -> Option<PersonSearchResult> {
    match event {
        PersonEvent::PersonCreated(e) => {
            Some(PersonSearchResult {
                person_id: e.person_id,
                name: e.name.display_name(),
                email: None,
                employer: None,
                role: None,
                relevance_score: 1.0,
            })
        }

        PersonEvent::NameUpdated(e) => {
            current.map(|mut result| {
                result.name = e.new_name.display_name();
                result
            })
        }

        PersonEvent::PersonDeactivated(_) |
        PersonEvent::PersonMergedInto(_) => {
            // Remove from search index
            None
        }

        PersonEvent::PersonReactivated(_) => {
            // Keep in search index if exists
            current
        }

        _ => {
            // Other events don't affect search index
            current
        }
    }
}

/// Project a PersonEvent into timeline entries
///
/// Returns a new timeline entry if the event should appear in the timeline.
/// This is a pure function that creates immutable timeline entries.
pub fn project_timeline_entry(event: &PersonEvent) -> Option<TimelineEntry> {
    match event {
        PersonEvent::PersonCreated(e) => Some(TimelineEntry {
            timestamp: e.created_at,
            event_type: "PersonCreated".to_string(),
            title: "Person Created".to_string(),
            description: format!("Created person record for {}", e.name.display_name()),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map.insert("source".to_string(), serde_json::json!(&e.source));
                map
            },
        }),

        PersonEvent::NameUpdated(e) => Some(TimelineEntry {
            timestamp: e.updated_at,
            event_type: "NameUpdated".to_string(),
            title: "Name Updated".to_string(),
            description: format!(
                "Name changed from {} to {}",
                e.old_name.display_name(),
                e.new_name.display_name()
            ),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map
            },
        }),

        PersonEvent::BirthDateSet(e) => Some(TimelineEntry {
            timestamp: e.set_at,
            event_type: "BirthDateSet".to_string(),
            title: "Birth Date Recorded".to_string(),
            description: format!("Birth date set to {}", e.birth_date),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map.insert("birth_date".to_string(), serde_json::json!(e.birth_date.to_string()));
                map
            },
        }),

        PersonEvent::DeathRecorded(e) => Some(TimelineEntry {
            timestamp: e.recorded_at,
            event_type: "DeathRecorded".to_string(),
            title: "Death Recorded".to_string(),
            description: format!("Death recorded on {}", e.date_of_death),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map.insert("date_of_death".to_string(), serde_json::json!(e.date_of_death.to_string()));
                map
            },
        }),

        PersonEvent::PersonDeactivated(e) => Some(TimelineEntry {
            timestamp: e.deactivated_at,
            event_type: "PersonDeactivated".to_string(),
            title: "Person Deactivated".to_string(),
            description: format!("Deactivated: {}", e.reason),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map.insert("reason".to_string(), serde_json::json!(&e.reason));
                map
            },
        }),

        PersonEvent::PersonReactivated(e) => Some(TimelineEntry {
            timestamp: e.reactivated_at,
            event_type: "PersonReactivated".to_string(),
            title: "Person Reactivated".to_string(),
            description: format!("Reactivated: {}", e.reason),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map.insert("reason".to_string(), serde_json::json!(&e.reason));
                map
            },
        }),

        PersonEvent::PersonMergedInto(e) => Some(TimelineEntry {
            timestamp: e.merged_at,
            event_type: "PersonMergedInto".to_string(),
            title: "Person Merged".to_string(),
            description: format!(
                "Merged into person {} due to {:?}",
                e.merged_into_id, e.merge_reason
            ),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("source_person_id".to_string(), serde_json::json!(e.source_person_id.to_string()));
                map.insert("merged_into_id".to_string(), serde_json::json!(e.merged_into_id.to_string()));
                map.insert("merge_reason".to_string(), serde_json::json!(format!("{:?}", e.merge_reason)));
                map
            },
        }),

        PersonEvent::AttributeRecorded(e) => Some(TimelineEntry {
            timestamp: e.recorded_at,
            event_type: "AttributeRecorded".to_string(),
            title: "Attribute Recorded".to_string(),
            description: format!("Recorded attribute: {:?}", e.attribute.attribute_type),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map.insert("attribute_type".to_string(), serde_json::json!(format!("{:?}", e.attribute.attribute_type)));
                map
            },
        }),

        PersonEvent::AttributeUpdated(e) => Some(TimelineEntry {
            timestamp: e.updated_at,
            event_type: "AttributeUpdated".to_string(),
            title: "Attribute Updated".to_string(),
            description: format!("Updated attribute: {:?}", e.new_attribute.attribute_type),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map.insert("attribute_type".to_string(), serde_json::json!(format!("{:?}", e.new_attribute.attribute_type)));
                map
            },
        }),

        PersonEvent::AttributeInvalidated(e) => Some(TimelineEntry {
            timestamp: e.invalidated_at,
            event_type: "AttributeInvalidated".to_string(),
            title: "Attribute Invalidated".to_string(),
            description: format!(
                "Invalidated attribute: {}",
                e.reason.as_deref().unwrap_or("No reason provided")
            ),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map.insert("reason".to_string(), serde_json::json!(&e.reason));
                map
            },
        }),

        PersonEvent::PersonUpdated(e) => Some(TimelineEntry {
            timestamp: e.updated_at,
            event_type: "PersonUpdated".to_string(),
            title: "Person Updated".to_string(),
            description: "Person record updated".to_string(),
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("person_id".to_string(), serde_json::json!(e.person_id.to_string()));
                map
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::PersonId;
    use crate::events::{PersonCreated, NameUpdated, PersonDeactivated};
    use crate::value_objects::PersonName;
    use chrono::Utc;

    #[test]
    fn test_project_person_summary_create() {
        let person_id = PersonId::new();
        let name = PersonName::new("Alice".to_string(), "Smith".to_string());

        let event = PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: name.clone(),
            source: "test".to_string(),
            created_at: Utc::now(),
        });

        let result = project_person_summary(None, &event);

        assert!(result.is_some());
        let summary = result.unwrap();
        assert_eq!(summary.person_id, person_id);
        assert_eq!(summary.name, name.display_name());
    }

    #[test]
    fn test_project_person_summary_update_name() {
        let person_id = PersonId::new();
        let old_name = PersonName::new("Alice".to_string(), "Smith".to_string());
        let new_name = PersonName::new("Alice".to_string(), "Jones".to_string());

        let current = PersonSummary {
            person_id,
            name: old_name.display_name(),
            primary_email: None,
            primary_phone: None,
            current_employer: None,
            current_role: None,
            location: None,
            skills_count: 0,
            component_count: 0,
            last_updated: Utc::now(),
        };

        let event = PersonEvent::NameUpdated(NameUpdated {
            person_id,
            old_name: old_name.clone(),
            new_name: new_name.clone(),
            reason: None,
            updated_at: Utc::now(),
        });

        let result = project_person_summary(Some(current), &event);

        assert!(result.is_some());
        let summary = result.unwrap();
        assert_eq!(summary.name, new_name.display_name());
    }

    #[test]
    fn test_project_person_summary_deactivate() {
        let person_id = PersonId::new();
        let current = PersonSummary {
            person_id,
            name: "Alice Smith".to_string(),
            primary_email: None,
            primary_phone: None,
            current_employer: None,
            current_role: None,
            location: None,
            skills_count: 0,
            component_count: 0,
            last_updated: Utc::now(),
        };

        let event = PersonEvent::PersonDeactivated(PersonDeactivated {
            person_id,
            reason: "Test deactivation".to_string(),
            deactivated_at: Utc::now(),
        });

        let result = project_person_summary(Some(current), &event);

        // Deactivation should remove the summary
        assert!(result.is_none());
    }

    #[test]
    fn test_project_timeline_entry() {
        let person_id = PersonId::new();
        let name = PersonName::new("Alice".to_string(), "Smith".to_string());

        let event = PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: name.clone(),
            source: "test".to_string(),
            created_at: Utc::now(),
        });

        let entry = project_timeline_entry(&event);

        assert!(entry.is_some());
        let timeline = entry.unwrap();
        assert_eq!(timeline.event_type, "PersonCreated");
        assert_eq!(timeline.title, "Person Created");
    }

    #[test]
    fn test_projection_functions_are_pure() {
        // Pure functions: same inputs produce same outputs
        let person_id = PersonId::new();
        let name = PersonName::new("Alice".to_string(), "Smith".to_string());
        let timestamp = Utc::now();

        let event = PersonEvent::PersonCreated(PersonCreated {
            person_id,
            name: name.clone(),
            source: "test".to_string(),
            created_at: timestamp,
        });

        // Call the function multiple times with same inputs
        let result1 = project_person_summary(None, &event);
        let result2 = project_person_summary(None, &event);

        // Results should be identical (except for system timestamps)
        assert!(result1.is_some());
        assert!(result2.is_some());
        let summary1 = result1.unwrap();
        let summary2 = result2.unwrap();

        assert_eq!(summary1.person_id, summary2.person_id);
        assert_eq!(summary1.name, summary2.name);
    }
}
