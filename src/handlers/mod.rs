//! Person domain handlers

mod command_handlers;
mod component_handler;

pub use command_handlers::{handle_create_person, handle_person_command};
pub use component_handler::*;
