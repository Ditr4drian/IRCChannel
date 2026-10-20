//! Nickname / identity as seen in chat.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub nick: String,
    pub user: Option<String>,
    pub host: Option<String>,
}

impl User {
    pub fn nick(nick: impl Into<String>) -> Self {
        Self {
            nick: nick.into(),
            user: None,
            host: None,
        }
    }

    /// Parse `nick!user@host` or bare `nick`.
    pub fn from_prefix(prefix: &str) -> Self {
        if let Some((nick, rest)) = prefix.split_once('!') {
            let (user, host) = rest
                .split_once('@')
                .map(|(u, h)| (Some(u.to_string()), Some(h.to_string())))
                .unwrap_or((Some(rest.to_string()), None));
            Self {
                nick: nick.to_string(),
                user,
                host,
            }
        } else {
            Self::nick(prefix)
        }
    }

    pub fn display(&self) -> &str {
        &self.nick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_prefix() {
        let u = User::from_prefix("alice!~a@vpn.ditra");
        assert_eq!(u.nick, "alice");
        assert_eq!(u.user.as_deref(), Some("~a"));
        assert_eq!(u.host.as_deref(), Some("vpn.ditra"));
    }
}
