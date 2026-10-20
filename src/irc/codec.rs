//! Minimal IRC message codec (parse + serialize).
//!
//! Intentionally small: enough for chat MVP, easy to extend without pulling a
//! full protocol crate into the dependency graph.

use crate::error::{Error, Result};

/// One IRC protocol message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMessage {
    pub tags: Vec<(String, Option<String>)>,
    pub prefix: Option<String>,
    pub command: String,
    pub params: Vec<String>,
}

impl RawMessage {
    pub fn new(command: impl Into<String>, params: Vec<String>) -> Self {
        Self {
            tags: Vec::new(),
            prefix: None,
            command: command.into(),
            params,
        }
    }

    pub fn trailing(&self) -> Option<&str> {
        self.params.last().map(String::as_str)
    }

    pub fn param(&self, idx: usize) -> Option<&str> {
        self.params.get(idx).map(String::as_str)
    }

    /// Serialize to a single IRC line **without** CR-LF.
    pub fn encode(&self) -> String {
        let mut out = String::new();
        if let Some(prefix) = &self.prefix {
            out.push(':');
            out.push_str(prefix);
            out.push(' ');
        }
        out.push_str(&self.command);
        let n = self.params.len();
        for (i, p) in self.params.iter().enumerate() {
            out.push(' ');
            if i == n - 1 && (p.contains(' ') || p.is_empty() || p.starts_with(':')) {
                out.push(':');
            }
            out.push_str(p);
        }
        out
    }
}

/// Parse one IRC line (CR/LF already stripped). Supports optional message-tags.
pub fn parse_line(line: &str) -> Result<RawMessage> {
    let mut rest = line.trim_end_matches(['\r', '\n']);
    if rest.is_empty() {
        return Err(Error::Protocol("empty line".into()));
    }

    let mut tags = Vec::new();
    if let Some(stripped) = rest.strip_prefix('@') {
        let (tag_str, after) = stripped
            .split_once(' ')
            .ok_or_else(|| Error::Protocol("tags without body".into()))?;
        for part in tag_str.split(';') {
            if part.is_empty() {
                continue;
            }
            if let Some((k, v)) = part.split_once('=') {
                tags.push((k.to_string(), Some(unescape_tag_value(v))));
            } else {
                tags.push((part.to_string(), None));
            }
        }
        rest = after;
    }

    let mut prefix = None;
    if let Some(stripped) = rest.strip_prefix(':') {
        let (p, after) = stripped
            .split_once(' ')
            .ok_or_else(|| Error::Protocol("prefix without command".into()))?;
        prefix = Some(p.to_string());
        rest = after;
    }

    let mut parts = split_params(rest);
    if parts.is_empty() {
        return Err(Error::Protocol("missing command".into()));
    }
    let command = parts.remove(0);

    Ok(RawMessage {
        tags,
        prefix,
        command,
        params: parts,
    })
}

fn split_params(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = input.trim_start();
    while !rest.is_empty() {
        if let Some(trailing) = rest.strip_prefix(':') {
            out.push(trailing.to_string());
            break;
        }
        if let Some((word, after)) = rest.split_once(' ') {
            out.push(word.to_string());
            rest = after.trim_start();
        } else {
            out.push(rest.to_string());
            break;
        }
    }
    out
}

fn unescape_tag_value(v: &str) -> String {
    // IRC tag escapes: \: \; \s \\ \r \n
    let mut out = String::with_capacity(v.len());
    let mut chars = v.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(':') => out.push(';'),
                Some('s') => out.push(' '),
                Some('\\') => out.push('\\'),
                Some('r') => out.push('\r'),
                Some('n') => out.push('\n'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_privmsg() {
        let m = parse_line(":nick!~u@h PRIVMSG #ditra :hola mundo").unwrap();
        assert_eq!(m.prefix.as_deref(), Some("nick!~u@h"));
        assert_eq!(m.command, "PRIVMSG");
        assert_eq!(m.params, vec!["#ditra", "hola mundo"]);
    }

    #[test]
    fn encode_privmsg() {
        let m = RawMessage::new("PRIVMSG", vec!["#ditra".into(), "hola mundo".into()]);
        assert_eq!(m.encode(), "PRIVMSG #ditra :hola mundo");
    }

    #[test]
    fn parse_ping() {
        let m = parse_line("PING :irc.ditra.local").unwrap();
        assert_eq!(m.command, "PING");
        assert_eq!(m.trailing(), Some("irc.ditra.local"));
    }

    #[test]
    fn roundtrip_join() {
        let m = RawMessage::new("JOIN", vec!["#ditra".into()]);
        let again = parse_line(&m.encode()).unwrap();
        assert_eq!(again.command, "JOIN");
        assert_eq!(again.params, vec!["#ditra"]);
    }
}
