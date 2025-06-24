//! Projections for the Person domain - ECS Architecture
//!
//! In ECS architecture, projections focus on core identity.
//! Component-specific views are handled by their respective systems.

use crate::aggregate::{PersonId, ComponentType, PersonLifecycle};
use crate::events::*;
use crate::value_objects::PersonName;
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashSet;

/// Basic person view - core identity only
#[derive(Debug, Clone)]
pub struct PersonView {
    pub id: PersonId,
    pub name: PersonName,
    pub birth_date: Option<NaiveDate>,
    pub death_date: Option<NaiveDate>,
    pub lifecycle: PersonLifecycle,
    pub registered_components: HashSet<ComponentType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PersonView {
    /// Create a new person view from creation event
    pub fn from_created_event(event: &PersonCreated) -> Self {
        Self {
            id: event.person_id,
            name: event.name.clone(),
            birth_date: None,
            death_date: None,
            lifecycle: PersonLifecycle::Active,
            registered_components: HashSet::new(),
            created_at: event.created_at,
            updated_at: event.created_at,
        }
    }

    /// Update the view based on an event
    pub fn apply_event(&mut self, event: &PersonEvent) {
        match event {
            PersonEvent::PersonCreated(e) => {
                self.id = e.person_id;
                self.name = e.name.clone();
                self.created_at = e.created_at;
                self.updated_at = e.created_at;
            }
            PersonEvent::NameUpdated(e) => {
                self.name = e.new_name.clone();
                self.updated_at = e.updated_at;
            }
            PersonEvent::BirthDateSet(e) => {
                self.birth_date = Some(e.birth_date);
                self.updated_at = e.set_at;
            }
            PersonEvent::DeathRecorded(e) => {
                self.death_date = Some(e.date_of_death);
                self.lifecycle = PersonLifecycle::Deceased {
                    date_of_death: e.date_of_death,
                };
                self.updated_at = e.recorded_at;
            }
            PersonEvent::ComponentRegistered(e) => {
                self.registered_components.insert(e.component_type.clone());
                self.updated_at = e.registered_at;
            }
            PersonEvent::ComponentUnregistered(e) => {
                self.registered_components.remove(&e.component_type);
                self.updated_at = e.unregistered_at;
            }
            PersonEvent::PersonDeactivated(e) => {
                self.lifecycle = PersonLifecycle::Deactivated {
                    reason: e.reason.clone(),
                    since: e.deactivated_at,
                };
                self.updated_at = e.deactivated_at;
            }
            PersonEvent::PersonReactivated(e) => {
                self.lifecycle = PersonLifecycle::Active;
                self.updated_at = e.reactivated_at;
            }
            PersonEvent::PersonMergedInto(e) => {
                self.lifecycle = PersonLifecycle::MergedInto {
                    target_id: e.merged_into_id,
                    merged_at: e.merged_at,
                };
                self.updated_at = e.merged_at;
            }
        }
    }
}

/// Person list item for search results
#[derive(Debug, Clone)]
pub struct PersonListItem {
    pub id: PersonId,
    pub display_name: String,
    pub lifecycle: PersonLifecycle,
    pub component_count: usize,
    pub updated_at: DateTime<Utc>,
}

impl From<&PersonView> for PersonListItem {
    fn from(view: &PersonView) -> Self {
        Self {
            id: view.id,
            display_name: view.name.display_name(),
            lifecycle: view.lifecycle.clone(),
            component_count: view.registered_components.len(),
            updated_at: view.updated_at,
        }
    }
}

/// Statistics view for analytics
#[derive(Debug, Clone, Default)]
pub struct PersonStatistics {
    pub total_persons: usize,
    pub active_persons: usize,
    pub deactivated_persons: usize,
    pub deceased_persons: usize,
    pub merged_persons: usize,
    pub persons_with_components: usize,
    pub component_usage: Vec<(ComponentType, usize)>,
}

impl PersonStatistics {
    /// Update statistics based on a person view
    pub fn add_person(&mut self, view: &PersonView) {
        self.total_persons += 1;

        match &view.lifecycle {
            PersonLifecycle::Active => self.active_persons += 1,
            PersonLifecycle::Deactivated { .. } => self.deactivated_persons += 1,
            PersonLifecycle::Deceased { .. } => self.deceased_persons += 1,
            PersonLifecycle::MergedInto { .. } => self.merged_persons += 1,
        }

        if !view.registered_components.is_empty() {
            self.persons_with_components += 1;
        }
    }
}
