//! Channel buffer state independent of the wire protocol.

use std::collections::{HashMap, HashSet, VecDeque};

use super::{ChatLine, User};

#[derive(Debug, Clone)]
pub struct Channel {
    pub name: String,
    pub topic: Option<String>,
    pub nicks: HashSet<String>,
    history: VecDeque<ChatLine>,
    history_limit: usize,
}

impl Channel {
    pub fn new(name: impl Into<String>, history_limit: usize) -> Self {
        Self {
            name: name.into(),
            topic: None,
            nicks: HashSet::new(),
            history: VecDeque::new(),
            history_limit: history_limit.max(64),
        }
    }

    pub fn push(&mut self, line: ChatLine) {
        if self.history.len() >= self.history_limit {
            self.history.pop_front();
        }
        self.history.push_back(line);
    }

    pub fn lines(&self) -> impl Iterator<Item = &ChatLine> {
        self.history.iter()
    }

    pub fn add_nick(&mut self, nick: impl Into<String>) {
        self.nicks.insert(nick.into());
    }

    pub fn remove_nick(&mut self, nick: &str) {
        self.nicks.remove(nick);
    }

    pub fn rename_nick(&mut self, old: &str, new: &str) {
        if self.nicks.remove(old) {
            self.nicks.insert(new.to_string());
        }
    }
}

/// Named collection of channel buffers + a server/status buffer.
#[derive(Debug, Default)]
pub struct BufferSet {
    pub server: VecDeque<ChatLine>,
    channels: HashMap<String, Channel>,
    order: Vec<String>,
    active: Option<String>,
    history_limit: usize,
}

impl BufferSet {
    pub fn new(history_limit: usize) -> Self {
        Self {
            server: VecDeque::new(),
            channels: HashMap::new(),
            order: Vec::new(),
            active: None,
            history_limit: history_limit.max(64),
        }
    }

    pub fn ensure_channel(&mut self, name: &str) -> &mut Channel {
        if !self.channels.contains_key(name) {
            self.channels
                .insert(name.to_string(), Channel::new(name, self.history_limit));
            self.order.push(name.to_string());
            if self.active.is_none() {
                self.active = Some(name.to_string());
            }
        }
        self.channels.get_mut(name).expect("just inserted")
    }

    /// Focus a buffer. `server` is the special server/status buffer (no channel).
    pub fn set_active(&mut self, name: &str) {
        if name.eq_ignore_ascii_case("server") || name == "*" {
            self.active = None;
            return;
        }
        if self.channels.contains_key(name) {
            self.active = Some(name.to_string());
        }
    }

    /// `None` means the server/status buffer is focused.
    pub fn active_name(&self) -> Option<&str> {
        self.active.as_deref()
    }

    pub fn viewing_server(&self) -> bool {
        self.active.is_none()
    }

    pub fn active_channel(&self) -> Option<&Channel> {
        self.active.as_ref().and_then(|n| self.channels.get(n))
    }

    pub fn active_channel_mut(&mut self) -> Option<&mut Channel> {
        let key = self.active.clone()?;
        self.channels.get_mut(&key)
    }

    pub fn channel_names(&self) -> &[String] {
        &self.order
    }

    /// Tab order: server buffer first, then channels.
    pub fn tab_targets(&self) -> Vec<String> {
        let mut v = Vec::with_capacity(self.order.len() + 1);
        v.push("server".to_string());
        v.extend(self.order.iter().cloned());
        v
    }

    pub fn push_server(&mut self, line: ChatLine) {
        if self.server.len() >= self.history_limit {
            self.server.pop_front();
        }
        self.server.push_back(line);
    }

    /// Visible feedback: active channel if any, otherwise server buffer.
    pub fn push_feedback(&mut self, line: ChatLine) {
        if let Some(name) = self.active.clone() {
            self.ensure_channel(&name).push(line);
        } else {
            self.push_server(line);
        }
    }

    pub fn push_to(&mut self, target: Option<&str>, line: ChatLine) {
        match target {
            Some(name) if name.starts_with('#') || name.starts_with('&') => {
                self.ensure_channel(name).push(line);
            }
            Some(name) => {
                // query / nick buffer — treat as named channel-like buffer
                self.ensure_channel(name).push(line);
            }
            None => self.push_server(line),
        }
    }

    pub fn on_join(&mut self, channel: &str, user: &User, is_me: bool) {
        let ch = self.ensure_channel(channel);
        ch.add_nick(user.nick.clone());
        ch.push(ChatLine {
            ts_ms: super_now(),
            kind: super::LineKind::Join,
            target: Some(channel.to_string()),
            from: Some(user.clone()),
            text: format!("{} joined", user.nick),
        });
        if is_me {
            self.active = Some(channel.to_string());
        }
    }

    pub fn on_part(&mut self, channel: &str, user: &User, reason: &str) {
        if let Some(ch) = self.channels.get_mut(channel) {
            ch.remove_nick(&user.nick);
            ch.push(ChatLine {
                ts_ms: super_now(),
                kind: super::LineKind::Part,
                target: Some(channel.to_string()),
                from: Some(user.clone()),
                text: if reason.is_empty() {
                    format!("{} left", user.nick)
                } else {
                    format!("{} left ({reason})", user.nick)
                },
            });
        }
    }
}

fn super_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
