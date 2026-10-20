//! Visual tokens. Swap this module later for user themes without touching layout.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub border: Style,
    pub header: Style,
    pub nick: Style,
    pub self_nick: Style,
    pub text: Style,
    pub system: Style,
    pub warn: Style,
    pub error: Style,
    pub join: Style,
    pub part: Style,
    pub input: Style,
    pub status_ok: Style,
    pub status_bad: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: Style::default().fg(Color::DarkGray),
            header: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            nick: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            self_nick: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            text: Style::default().fg(Color::White),
            system: Style::default().fg(Color::DarkGray),
            warn: Style::default().fg(Color::Yellow),
            error: Style::default().fg(Color::Red),
            join: Style::default().fg(Color::Green),
            part: Style::default().fg(Color::Yellow),
            input: Style::default().fg(Color::White),
            status_ok: Style::default().fg(Color::Green),
            status_bad: Style::default().fg(Color::Red),
        }
    }
}
