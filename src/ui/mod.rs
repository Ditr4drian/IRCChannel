//! Terminal presentation. Renders [`crate::app::AppState`] only — no business
//! logic beyond layout math.

mod theme;
mod widgets;

use std::io::{self, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::AppState;
use crate::error::Result;
use crate::ui::theme::Theme;
use crate::ui::widgets::{chat_view, input_view, status_view};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    theme: Theme,
}

impl Tui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            theme: Theme::default(),
        })
    }

    pub fn draw(&mut self, state: &AppState) -> Result<()> {
        let theme = self.theme;
        self.terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // header / status
                    Constraint::Min(3),    // chat
                    Constraint::Length(3), // input
                ])
                .split(area);

            status_view::render(frame, chunks[0], state, &theme);
            chat_view::render(frame, chunks[1], state, &theme);
            input_view::render(frame, chunks[2], state, &theme);

            // subtle help on tiny terminals
            if area.height < 6 {
                frame.render_widget(Clear, area);
                frame.render_widget(
                    Paragraph::new("terminal too small")
                        .block(Block::default().borders(Borders::ALL)),
                    centered(area, 40, 3),
                );
            }
        })?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}
