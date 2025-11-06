//! Person domain handlers

mod command_handlers;
mod async_command_processor;

pub use command_handlers::{handle_create_person, handle_person_command};
pub use async_command_processor::{
    AsyncCommandProcessor, PersonCommandProcessor, CommandResult,
    AsyncComponentCommandHandler
};
