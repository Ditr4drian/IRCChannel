//! IRC wire adapter.
//!
//! Owns TCP IO and RFC1459-ish parsing. Emits [`event::IrcEvent`] upward and
//! accepts [`command::IrcCommand`] downward. No UI knowledge.

pub mod client;
pub mod codec;
pub mod command;
pub mod event;

pub use client::{IrcClient, IrcHandle};
pub use command::IrcCommand;
pub use event::IrcEvent;
