use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{AppState, ConnStatus};
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let (dot_style, dot) = match state.status {
        ConnStatus::Registered => (theme.status_ok, "●"),
        ConnStatus::Connected | ConnStatus::Connecting => (theme.warn, "●"),
        ConnStatus::Disconnected => (theme.status_bad, "●"),
    };

    let tabs = state.buffers.tab_targets();
    let active = state.active_title();
    let channels = tabs
        .iter()
        .map(|n| {
            if n.eq_ignore_ascii_case(&active) {
                format!("[{n}]")
            } else {
                n.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let line = Line::from(vec![
        Span::styled(format!(" {dot} "), dot_style),
        Span::styled("IRCChannel ", theme.header),
        Span::styled(format!("{active} "), theme.nick),
        Span::styled(format!("[{}] ", state.nick), theme.self_nick),
        Span::styled(state.status_text.clone(), theme.system),
        Span::raw("  "),
        Span::styled(channels, theme.system),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
