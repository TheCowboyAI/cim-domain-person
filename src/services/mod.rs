//! Services for working with person entities and components
//!
//! This module provides services for composing person entities with
//! different sets of components to create various views and concepts.

pub mod composition;
pub mod views;
pub mod network_analysis;

pub use composition::PersonCompositionService;
pub use views::{
    PersonView, ViewType, EmployeeViewBuilder, CustomerViewBuilder, PartnerViewBuilder,
    EmployeeView, CustomerView, PartnerView,
};

pub use network_analysis::*; 