//! IRCChannel — modular terminal IRC client.
//!
//! # Layers
//!
//! | Module | Responsibility |
//! |--------|----------------|
//! | [`domain`] | Pure chat model (lines, channels, users) |
//! | [`irc`] | Wire protocol + TCP adapter |
//! | [`app`] | TEA state machine (events → state + commands) |
//! | [`ui`] | ratatui rendering |
//! | [`config`] | defaults, file, env |
//!
//! Dependencies flow **downward only**: `ui`/`irc` → `app` → `domain`.
//! The binary (`main`) is the composition root that wires adapters together.

pub mod app;
pub mod config;
pub mod domain;
pub mod error;
pub mod irc;
pub mod timefmt;
pub mod ui;

pub use config::Config;
pub use error::{Error, Result};
