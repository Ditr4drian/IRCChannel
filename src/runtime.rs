//! Composition root helpers shared by `main`.
//!
//! Keeps `main.rs` thin: load config → connect → multiplex terminal + IRC events.

use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEventKind};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tracing::error;

use ircchannel::app::{AppCommand, AppEvent, AppState, update};
use ircchannel::config::Config;
use ircchannel::error::Result;
use ircchannel::irc::{IrcClient, IrcCommand, IrcHandle};
use ircchannel::ui::Tui;

pub async fn run(config: Config) -> Result<()> {
    let mut state = AppState::new(config.clone());
    let mut tui = Tui::new()?;
    tui.draw(&state)?;

    let handle = match IrcClient::connect(&config).await {
        Ok(h) => h,
        Err(e) => {
            let _ = update(&mut state, AppEvent::ConnectFailed(e.to_string()));
            tui.draw(&state)?;
            // brief pause so the user can read the error in the TUI before drop
            tokio::time::sleep(Duration::from_secs(2)).await;
            return Err(e);
        }
    };

    event_loop(&mut state, &mut tui, handle, config.ui.tick_ms).await
}

async fn event_loop(
    state: &mut AppState,
    tui: &mut Tui,
    mut handle: IrcHandle,
    tick_ms: u64,
) -> Result<()> {
    let mut term_events = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(tick_ms.max(50)));
    let cmd_tx = handle.commands.clone();

    loop {
        if state.should_quit {
            let _ = cmd_tx.send(IrcCommand::Quit("client exit".into()));
            break;
        }

        let event = tokio::select! {
            _ = tick.tick() => Some(AppEvent::Tick),
            maybe_term = term_events.next().fuse() => {
                match maybe_term {
                    // Press + Repeat (held keys). Ignore Release so we don't double-fire.
                    Some(Ok(CrosstermEvent::Key(key)))
                        if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                    {
                        Some(AppEvent::Key(key))
                    }
                    Some(Ok(CrosstermEvent::Resize(_, _))) => Some(AppEvent::Tick),
                    Some(Err(e)) => {
                        error!(error = %e, "terminal event error");
                        None
                    }
                    _ => None,
                }
            }
            maybe_irc = handle.events.recv() => {
                match maybe_irc {
                    Some(ev) => Some(AppEvent::Irc(ev)),
                    None => Some(AppEvent::Irc(ircchannel::irc::IrcEvent::Disconnected {
                        reason: "event channel closed".into(),
                    })),
                }
            }
        };

        if let Some(ev) = event {
            let cmds = update(state, ev);
            if execute_commands(&cmd_tx, cmds) {
                break;
            }
            tui.draw(state)?;
        }
    }

    Ok(())
}

/// Returns `true` when the runtime should stop.
fn execute_commands(cmd_tx: &mpsc::UnboundedSender<IrcCommand>, cmds: Vec<AppCommand>) -> bool {
    let mut quit = false;
    for cmd in flatten(cmds) {
        match cmd {
            AppCommand::Quit => quit = true,
            AppCommand::Irc(irc_cmd) => {
                if cmd_tx.send(irc_cmd).is_err() {
                    quit = true;
                }
            }
            AppCommand::Batch(_) => unreachable!("flattened"),
        }
    }
    quit
}

fn flatten(cmds: Vec<AppCommand>) -> Vec<AppCommand> {
    let mut out = Vec::new();
    for cmd in cmds {
        match cmd {
            AppCommand::Batch(inner) => out.extend(flatten(inner)),
            other => out.push(other),
        }
    }
    out
}
