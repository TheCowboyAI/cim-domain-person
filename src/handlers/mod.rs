//! Command and query handlers for person domain

pub mod command_handlers;
pub mod query_handlers;

pub use command_handlers::*;
pub use query_handlers::{PersonReadModel, PersonQueryResult, PersonQueryHandler};
