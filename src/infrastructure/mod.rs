//! Infrastructure layer for Person domain

pub mod event_store;
pub mod persistence;
pub mod nats_integration;
pub mod component_store;

pub use event_store::*;
pub use persistence::*;
pub use nats_integration::*;
pub use component_store::*; 