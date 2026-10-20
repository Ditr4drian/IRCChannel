//! Application state — the single source of truth for the UI.

use crate::config::Config;
use crate::domain::{BufferSet, ChatLine, SystemKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnStatus {
    Connecting,
    Connected,
    Registered,
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    /// Future: command palette / nick completion overlays.
    #[allow(dead_code)]
    Command,
}

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub status: ConnStatus,
    pub nick: String,
    pub buffers: BufferSet,
    pub input: String,
    pub input_cursor: usize,
    pub mode: InputMode,
    pub status_text: String,
    pub should_quit: bool,
    pub scroll_offset: u16,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let nick = config.identity.nick.clone();
        let history_limit = config.ui.history_limit;
        let mut buffers = BufferSet::new(history_limit);
        buffers.push_server(ChatLine::system(
            SystemKind::Info,
            format!(
                "IRCChannel → {}:{} as {nick}",
                config.connection.host, config.connection.port
            ),
        ));
        for ch in &config.session.channels {
            buffers.ensure_channel(ch);
        }
        if let Some(first) = config.session.channels.first() {
            buffers.set_active(first);
        }

        Self {
            status: ConnStatus::Connecting,
            nick,
            buffers,
            input: String::new(),
            input_cursor: 0,
            mode: InputMode::Normal,
            status_text: "connecting…".into(),
            should_quit: false,
            scroll_offset: 0,
            config,
        }
    }

    pub fn active_title(&self) -> String {
        self.buffers.active_name().unwrap_or("server").to_string()
    }

    /// User-visible note in the currently focused buffer.
    pub fn feedback(&mut self, kind: SystemKind, text: impl Into<String>) {
        self.buffers.push_feedback(ChatLine::system(kind, text));
        self.scroll_offset = 0;
    }

    pub fn feedback_info(&mut self, text: impl Into<String>) {
        self.feedback(SystemKind::Info, text);
    }

    pub fn feedback_warn(&mut self, text: impl Into<String>) {
        self.feedback(SystemKind::Warn, text);
    }
}
