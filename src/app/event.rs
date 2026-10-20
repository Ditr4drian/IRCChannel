//! Unified event bus for the app runtime.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::irc::IrcEvent;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Irc(IrcEvent),
    Tick,
    /// Runtime failed to connect before the IRC handle existed.
    ConnectFailed(String),
}

impl AppEvent {
    pub fn is_quit_key(key: &KeyEvent) -> bool {
        matches!(
            (key.modifiers, key.code),
            (KeyModifiers::CONTROL, KeyCode::Char('c'))
                | (KeyModifiers::CONTROL, KeyCode::Char('q'))
        )
    }
}
