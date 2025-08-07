//! Infrastructure layer for Person domain

pub mod event_store;
pub mod persistence;
pub mod nats_integration;
pub mod component_store;
pub mod streaming;
pub mod retry;
pub mod subscriptions;

pub use event_store::*;
pub use persistence::*;
pub use nats_integration::*;
pub use component_store::*;
pub use streaming::{StreamingConfig, StreamingClient, EventMetadata};
pub use retry::{RetryHandler, CircuitBreaker};
pub use subscriptions::{SubscriptionManager, StreamingEventHandler}; 