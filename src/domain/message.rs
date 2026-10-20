//! Chat lines as the UI/history understands them (already decoded).

use std::time::{SystemTime, UNIX_EPOCH};

use super::User;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Privmsg,
    Notice,
    Action,
    Join,
    Part,
    Quit,
    Nick,
    Topic,
    Mode,
    System(SystemKind),
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemKind {
    Info,
    Warn,
    Error,
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct ChatLine {
    pub ts_ms: u64,
    pub kind: LineKind,
    pub target: Option<String>,
    pub from: Option<User>,
    pub text: String,
}

impl ChatLine {
    pub fn now(kind: LineKind, text: impl Into<String>) -> Self {
        Self {
            ts_ms: now_ms(),
            kind,
            target: None,
            from: None,
            text: text.into(),
        }
    }

    pub fn system(kind: SystemKind, text: impl Into<String>) -> Self {
        Self::now(LineKind::System(kind), text)
    }

    pub fn privmsg(target: impl Into<String>, from: User, text: impl Into<String>) -> Self {
        Self {
            ts_ms: now_ms(),
            kind: LineKind::Privmsg,
            target: Some(target.into()),
            from: Some(from),
            text: text.into(),
        }
    }

    pub fn notice(target: Option<String>, from: Option<User>, text: impl Into<String>) -> Self {
        Self {
            ts_ms: now_ms(),
            kind: LineKind::Notice,
            target,
            from,
            text: text.into(),
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
