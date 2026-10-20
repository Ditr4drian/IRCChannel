//! Outbound intentions from the app layer toward the IRC adapter.

#[derive(Debug, Clone)]
pub enum IrcCommand {
    /// Low-level escape hatch.
    Raw(String),
    Nick(String),
    User {
        username: String,
        realname: String,
    },
    Pass(String),
    Join(String),
    Part {
        channel: String,
        reason: String,
    },
    Privmsg {
        target: String,
        text: String,
    },
    Notice {
        target: String,
        text: String,
    },
    Ping(String),
    Pong(String),
    Quit(String),
}

impl IrcCommand {
    pub fn to_wire(&self) -> String {
        use crate::irc::codec::RawMessage;

        let msg = match self {
            Self::Raw(line) => return line.clone(),
            Self::Nick(n) => RawMessage::new("NICK", vec![n.clone()]),
            Self::User { username, realname } => RawMessage::new(
                "USER",
                vec![username.clone(), "0".into(), "*".into(), realname.clone()],
            ),
            Self::Pass(p) => RawMessage::new("PASS", vec![p.clone()]),
            Self::Join(c) => RawMessage::new("JOIN", vec![c.clone()]),
            Self::Part { channel, reason } => {
                if reason.is_empty() {
                    RawMessage::new("PART", vec![channel.clone()])
                } else {
                    RawMessage::new("PART", vec![channel.clone(), reason.clone()])
                }
            }
            Self::Privmsg { target, text } => {
                RawMessage::new("PRIVMSG", vec![target.clone(), text.clone()])
            }
            Self::Notice { target, text } => {
                RawMessage::new("NOTICE", vec![target.clone(), text.clone()])
            }
            Self::Ping(t) => RawMessage::new("PING", vec![t.clone()]),
            Self::Pong(t) => RawMessage::new("PONG", vec![t.clone()]),
            Self::Quit(r) => {
                if r.is_empty() {
                    RawMessage::new("QUIT", vec![])
                } else {
                    RawMessage::new("QUIT", vec![r.clone()])
                }
            }
        };
        msg.encode()
    }
}
