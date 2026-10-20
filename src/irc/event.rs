//! Inbound events produced by the IRC adapter for the app layer.

use crate::domain::User;
use crate::irc::codec::RawMessage;

#[derive(Debug, Clone)]
pub enum IrcEvent {
    Connected,
    Disconnected {
        reason: String,
    },
    /// Welcome (001) — registration finished.
    Registered {
        nick: String,
    },
    /// Server wants a PONG (adapter may auto-answer; still surfaced).
    Ping {
        token: String,
    },
    Privmsg {
        from: User,
        target: String,
        text: String,
    },
    Notice {
        from: Option<User>,
        target: Option<String>,
        text: String,
    },
    Join {
        user: User,
        channel: String,
    },
    Part {
        user: User,
        channel: String,
        reason: String,
    },
    Quit {
        user: User,
        reason: String,
    },
    Nick {
        user: User,
        new_nick: String,
    },
    Topic {
        channel: String,
        topic: String,
    },
    Names {
        channel: String,
        nicks: Vec<String>,
    },
    /// Numeric or unhandled — for logging / progressive support.
    Raw(RawMessage),
    Error(String),
}

/// Map a parsed wire message into a typed event when possible.
pub fn classify(msg: RawMessage) -> IrcEvent {
    let cmd = msg.command.to_uppercase();
    match cmd.as_str() {
        "PING" => IrcEvent::Ping {
            token: msg.trailing().unwrap_or("").to_string(),
        },
        "PRIVMSG" => {
            let from = msg
                .prefix
                .as_deref()
                .map(User::from_prefix)
                .unwrap_or_else(|| User::nick("unknown"));
            let target = msg.param(0).unwrap_or("*").to_string();
            let text = msg.trailing().unwrap_or("").to_string();
            // CTCP ACTION
            if text.starts_with('\u{1}') && text.ends_with('\u{1}') {
                let inner = &text[1..text.len() - 1];
                if let Some(action) = inner.strip_prefix("ACTION ") {
                    return IrcEvent::Privmsg {
                        from,
                        target,
                        text: format!("* {action}"),
                    };
                }
            }
            IrcEvent::Privmsg { from, target, text }
        }
        "NOTICE" => IrcEvent::Notice {
            from: msg.prefix.as_deref().map(User::from_prefix),
            target: msg.param(0).map(str::to_string),
            text: msg.trailing().unwrap_or("").to_string(),
        },
        "JOIN" => IrcEvent::Join {
            user: msg
                .prefix
                .as_deref()
                .map(User::from_prefix)
                .unwrap_or_else(|| User::nick("unknown")),
            channel: msg
                .trailing()
                .or_else(|| msg.param(0))
                .unwrap_or("#?")
                .to_string(),
        },
        "PART" => IrcEvent::Part {
            user: msg
                .prefix
                .as_deref()
                .map(User::from_prefix)
                .unwrap_or_else(|| User::nick("unknown")),
            channel: msg.param(0).unwrap_or("#?").to_string(),
            reason: msg.trailing().unwrap_or("").to_string(),
        },
        "QUIT" => IrcEvent::Quit {
            user: msg
                .prefix
                .as_deref()
                .map(User::from_prefix)
                .unwrap_or_else(|| User::nick("unknown")),
            reason: msg.trailing().unwrap_or("").to_string(),
        },
        "NICK" => IrcEvent::Nick {
            user: msg
                .prefix
                .as_deref()
                .map(User::from_prefix)
                .unwrap_or_else(|| User::nick("unknown")),
            new_nick: msg
                .trailing()
                .or_else(|| msg.param(0))
                .unwrap_or("?")
                .to_string(),
        },
        "TOPIC" => IrcEvent::Topic {
            channel: msg.param(0).unwrap_or("#?").to_string(),
            topic: msg.trailing().unwrap_or("").to_string(),
        },
        "001" => IrcEvent::Registered {
            nick: msg.param(0).unwrap_or("").to_string(),
        },
        "332" => IrcEvent::Topic {
            channel: msg.param(1).unwrap_or("#?").to_string(),
            topic: msg.trailing().unwrap_or("").to_string(),
        },
        "353" => {
            // :server 353 me = #chan :nick1 nick2
            let channel = msg.param(2).unwrap_or("#?").to_string();
            let nicks = msg
                .trailing()
                .unwrap_or("")
                .split_whitespace()
                .map(|n| n.trim_start_matches(['@', '+', '%', '&', '~']).to_string())
                .collect();
            IrcEvent::Names { channel, nicks }
        }
        "ERROR" => IrcEvent::Error(msg.trailing().unwrap_or("ERROR").to_string()),
        _ => IrcEvent::Raw(msg),
    }
}
