//! Projections for the Person domain

use crate::aggregate::PersonId;
use crate::events::*;
use crate::value_objects::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Basic person view
#[derive(Debug, Clone)]
pub struct PersonView {
    pub id: PersonId,
    pub name: PersonName,
    pub primary_email: Option<String>,
    pub primary_phone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contact information view
#[derive(Debug, Clone)]
pub struct ContactView {
    pub person_id: PersonId,
    pub emails: HashMap<String, EmailAddress>,
    pub phones: HashMap<String, PhoneNumber>,
    pub addresses: HashMap<AddressType, PhysicalAddress>,
}

/// Customer view
#[derive(Debug, Clone)]
pub struct CustomerView {
    pub person_id: PersonId,
    pub name: String,
    pub segment: Option<SegmentType>,
    pub value_tier: Option<ValueTier>,
    pub lifetime_value: Option<f32>,
    pub engagement_score: Option<f32>,
}

/// Employee view
#[derive(Debug, Clone)]
pub struct EmployeeView {
    pub person_id: PersonId,
    pub name: String,
    pub current_position: Option<String>,
    pub department: Option<String>,
    pub manager_name: Option<String>,
}

/// Update projections based on events
pub fn update_person_view(view: &mut PersonView, event: &PersonEvent) {
    match event {
        PersonEvent::PersonCreated(e) => {
            view.id = e.person_id;
            view.name = e.name.clone();
            view.created_at = e.created_at;
            view.updated_at = e.created_at;
        }
        PersonEvent::NameUpdated(e) => {
            view.name = e.new_name.clone();
            view.updated_at = e.updated_at;
        }
        PersonEvent::EmailAdded(e) => {
            if e.primary {
                view.primary_email = Some(e.email.address.clone());
            }
            view.updated_at = e.added_at;
        }
        PersonEvent::EmailRemoved(e) => {
            if view.primary_email.as_ref() == Some(&e.email) {
                view.primary_email = None;
            }
            view.updated_at = e.removed_at;
        }
        PersonEvent::PhoneAdded(e) => {
            if e.primary {
                view.primary_phone = Some(e.phone.number.clone());
            }
            view.updated_at = e.added_at;
        }
        PersonEvent::PhoneRemoved(e) => {
            if view.primary_phone.as_ref() == Some(&e.phone) {
                view.primary_phone = None;
            }
            view.updated_at = e.removed_at;
        }
        PersonEvent::PersonDeactivated(e) => {
            view.is_active = false;
            view.updated_at = e.deactivated_at;
        }
        PersonEvent::PersonReactivated(e) => {
            view.is_active = true;
            view.updated_at = e.reactivated_at;
        }
        _ => {}
    }
}

pub fn update_contact_view(view: &mut ContactView, event: &PersonEvent) {
    match event {
        PersonEvent::EmailAdded(e) => {
            view.emails.insert(e.email.address.clone(), e.email.clone());
        }
        PersonEvent::EmailRemoved(e) => {
            view.emails.remove(&e.email);
        }
        PersonEvent::PhoneAdded(e) => {
            view.phones.insert(e.phone.number.clone(), e.phone.clone());
        }
        PersonEvent::PhoneRemoved(e) => {
            view.phones.remove(&e.phone);
        }
        PersonEvent::AddressAdded(e) => {
            view.addresses.insert(e.address_type.clone(), e.address.clone());
        }
        PersonEvent::AddressRemoved(e) => {
            view.addresses.remove(&e.address_type);
        }
        _ => {}
    }
}

pub fn update_customer_view(view: &mut CustomerView, event: &PersonEvent) {
    match event {
        PersonEvent::NameUpdated(e) => {
            view.name = e.new_name.display_name();
        }
        PersonEvent::CustomerSegmentSet(e) => {
            view.segment = Some(e.segment.segment_type.clone());
            view.value_tier = Some(e.segment.value_tier.clone());
        }
        PersonEvent::BehavioralDataUpdated(e) => {
            view.lifetime_value = e.data.lifetime_value;
            view.engagement_score = e.data.engagement_score;
        }
        _ => {}
    }
}
