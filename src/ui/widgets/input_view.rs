use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::app::AppState;
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let target = state.active_title();
    let title = format!(" message → {target}  (/help) ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(title);

    let prefix = format!("[{}] ", state.nick);
    let body = format!("{prefix}{}", state.input);

    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(body, theme.input))),
        inner,
    );

    // cursor: after prefix + char-width of text before cursor
    let before = &state.input[..state.input_cursor.min(state.input.len())];
    let x = inner.x
        + u16::try_from(prefix.width() + before.width())
            .unwrap_or(0)
            .min(inner.width.saturating_sub(1));
    let y = inner.y;
    frame.set_cursor_position((x, y));
}
