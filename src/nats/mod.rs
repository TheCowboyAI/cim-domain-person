//! NATS integration and subject algebra for the Person domain
//!
//! This module provides comprehensive NATS messaging support with:
//! - Subject algebra for events, commands, and queries
//! - Message identity and correlation
//! - Event publishing and subscription utilities

pub mod subjects;
pub mod message_identity;

pub use subjects::*;
pub use message_identity::*;