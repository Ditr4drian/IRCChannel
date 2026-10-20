//! Pure domain types — no IO, no TUI, no tokio.
//!
//! Everything the UI renders and the app mutates should bottom out here so
//! protocol and presentation can evolve independently.

mod channel;
mod message;
mod user;

pub use channel::{BufferSet, Channel};
pub use message::{ChatLine, LineKind, SystemKind};
pub use user::User;
