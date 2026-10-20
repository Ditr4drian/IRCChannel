//! Application core (TEA-style).
//!
//! - [`state::AppState`] — pure data the UI renders
//! - [`event::AppEvent`] — inputs from keyboard + IRC + ticks
//! - [`update::update`] — pure-ish transition returning side-effect commands
//! - [`command::AppCommand`] — effects the runtime executes (send IRC, quit, …)

pub mod command;
pub mod event;
pub mod state;
pub mod update;

pub use command::AppCommand;
pub use event::AppEvent;
pub use state::{AppState, ConnStatus, InputMode};
pub use update::update;
