use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::AppState;
use crate::domain::{ChatLine, LineKind, SystemKind};
use crate::ui::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let title = format!(" {} ", state.active_title());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<&ChatLine> = if let Some(ch) = state.buffers.active_channel() {
        ch.lines().collect()
    } else {
        state.buffers.server.iter().collect()
    };

    let height = inner.height as usize;
    let total = lines.len();
    let offset = state.scroll_offset as usize;
    let end = total.saturating_sub(offset);
    let start = end.saturating_sub(height);
    let window = &lines[start..end];

    let tz = state.config.ui.timezone.as_str();
    let rendered: Vec<Line> = window
        .iter()
        .map(|l| format_line(l, &state.nick, theme, tz))
        .collect();

    frame.render_widget(
        Paragraph::new(rendered)
            .wrap(Wrap { trim: false })
            .style(theme.text),
        inner,
    );
}

fn format_line<'a>(line: &'a ChatLine, me: &str, theme: &Theme, tz: &str) -> Line<'a> {
    let ts = crate::timefmt::format_hhmmss(line.ts_ms, tz);

    match line.kind {
        LineKind::Privmsg | LineKind::Action => {
            let nick = line.from.as_ref().map(|u| u.nick.as_str()).unwrap_or("?");
            let nick_style = if nick.eq_ignore_ascii_case(me) {
                theme.self_nick
            } else {
                theme.nick
            };
            Line::from(vec![
                Span::styled(format!("{ts} "), theme.system),
                Span::styled(format!("<{nick}> "), nick_style),
                Span::styled(line.text.clone(), theme.text),
            ])
        }
        LineKind::Notice => Line::from(vec![
            Span::styled(format!("{ts} "), theme.system),
            Span::styled("-notice- ", theme.warn),
            Span::styled(line.text.clone(), theme.text),
        ]),
        LineKind::Join => Line::from(vec![
            Span::styled(format!("{ts} "), theme.system),
            Span::styled(format!("→ {}", line.text), theme.join),
        ]),
        LineKind::Part | LineKind::Quit => Line::from(vec![
            Span::styled(format!("{ts} "), theme.system),
            Span::styled(format!("← {}", line.text), theme.part),
        ]),
        LineKind::System(kind) => {
            let style = match kind {
                SystemKind::Error => theme.error,
                SystemKind::Warn => theme.warn,
                SystemKind::Connected => theme.join,
                SystemKind::Disconnected => theme.error,
                SystemKind::Info => theme.system,
            };
            Line::from(vec![
                Span::styled(format!("{ts} "), theme.system),
                Span::styled(format!("* {}", line.text), style),
            ])
        }
        LineKind::Topic | LineKind::Nick | LineKind::Mode | LineKind::Raw => Line::from(vec![
            Span::styled(format!("{ts} "), theme.system),
            Span::styled(format!("* {}", line.text), theme.system),
        ]),
    }
}
