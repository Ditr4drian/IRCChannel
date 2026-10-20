//! State transitions. Keep this free of tokio/ratatui so it stays testable.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::command::AppCommand;
use crate::app::event::AppEvent;
use crate::app::state::{AppState, ConnStatus};
use crate::config::normalize_channel;
use crate::domain::{ChatLine, LineKind, SystemKind, User};
use crate::irc::{IrcCommand, IrcEvent};

pub fn update(state: &mut AppState, event: AppEvent) -> Vec<AppCommand> {
    match event {
        AppEvent::Tick => AppCommand::none(),
        AppEvent::ConnectFailed(reason) => {
            state.status = ConnStatus::Disconnected;
            state.status_text = format!("connect failed: {reason}");
            state.buffers.push_server(ChatLine::system(
                SystemKind::Error,
                format!("connect failed: {reason}"),
            ));
            AppCommand::none()
        }
        AppEvent::Key(key) => handle_key(state, key),
        AppEvent::Irc(ev) => handle_irc(state, ev),
    }
}

fn handle_key(state: &mut AppState, key: KeyEvent) -> Vec<AppCommand> {
    if AppEvent::is_quit_key(&key) {
        state.should_quit = true;
        return AppCommand::one(AppCommand::Batch(vec![
            AppCommand::Irc(IrcCommand::Quit("bye".into())),
            AppCommand::Quit,
        ]));
    }

    match key.code {
        // Some terminals deliver Enter as a control char instead of KeyCode::Enter.
        KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => submit_input(state),
        KeyCode::Backspace => {
            if state.input_cursor > 0 {
                let idx = prev_char_boundary(&state.input, state.input_cursor);
                state.input.drain(idx..state.input_cursor);
                state.input_cursor = idx;
            }
            AppCommand::none()
        }
        KeyCode::Delete => {
            if state.input_cursor < state.input.len() {
                let end = next_char_boundary(&state.input, state.input_cursor);
                state.input.drain(state.input_cursor..end);
            }
            AppCommand::none()
        }
        KeyCode::Left => {
            state.input_cursor = prev_char_boundary(&state.input, state.input_cursor);
            AppCommand::none()
        }
        KeyCode::Right => {
            state.input_cursor = next_char_boundary(&state.input, state.input_cursor);
            AppCommand::none()
        }
        KeyCode::Home => {
            state.input_cursor = 0;
            AppCommand::none()
        }
        KeyCode::End => {
            state.input_cursor = state.input.len();
            AppCommand::none()
        }
        KeyCode::PageUp => {
            state.scroll_offset = state.scroll_offset.saturating_add(5);
            AppCommand::none()
        }
        KeyCode::PageDown => {
            state.scroll_offset = state.scroll_offset.saturating_sub(5);
            AppCommand::none()
        }
        KeyCode::Tab => {
            cycle_buffer(state, key.modifiers.contains(KeyModifiers::SHIFT));
            AppCommand::none()
        }
        KeyCode::Esc => {
            state.input.clear();
            state.input_cursor = 0;
            AppCommand::none()
        }
        // Accept printable chars; SHIFT/ALT-GR ok. Only block CONTROL combos.
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !c.is_control() => {
            state.input.insert(state.input_cursor, c);
            state.input_cursor += c.len_utf8();
            AppCommand::none()
        }
        _ => AppCommand::none(),
    }
}

fn submit_input(state: &mut AppState) -> Vec<AppCommand> {
    let raw = state.input.trim().to_string();
    state.input.clear();
    state.input_cursor = 0;
    if raw.is_empty() {
        return AppCommand::none();
    }

    // Slash commands (and // escapes to send a literal leading slash).
    if let Some(rest) = raw.strip_prefix('/') {
        if let Some(literal) = rest.strip_prefix('/') {
            return send_privmsg(state, format!("/{literal}"));
        }
        return handle_slash(state, rest);
    }

    send_privmsg(state, raw)
}

fn send_privmsg(state: &mut AppState, text: String) -> Vec<AppCommand> {
    let Some(target) = state.buffers.active_name().map(str::to_string) else {
        state.feedback_warn("no channel focused — /join #channel or Tab away from server");
        return AppCommand::none();
    };

    state.buffers.push_to(
        Some(&target),
        ChatLine::privmsg(&target, User::nick(state.nick.clone()), text.clone()),
    );
    state.scroll_offset = 0;

    AppCommand::one(AppCommand::Irc(IrcCommand::Privmsg { target, text }))
}

fn handle_slash(state: &mut AppState, cmd: &str) -> Vec<AppCommand> {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        state.feedback_warn("empty command — try /help");
        return AppCommand::none();
    }

    // Echo so the user always sees that Enter registered.
    state.feedback_info(format!("» /{cmd}"));

    let mut parts = cmd.splitn(2, char::is_whitespace);
    let name = parts.next().unwrap_or("").to_lowercase();
    let rest = parts.next().unwrap_or("").trim();

    match name.as_str() {
        "quit" | "q" | "exit" => {
            state.should_quit = true;
            state.status_text = "quitting…".into();
            AppCommand::one(AppCommand::Batch(vec![
                AppCommand::Irc(IrcCommand::Quit(if rest.is_empty() {
                    "bye".into()
                } else {
                    rest.into()
                })),
                AppCommand::Quit,
            ]))
        }
        "join" | "j" => {
            if rest.is_empty() {
                state.feedback_warn("usage: /join #channel");
                return AppCommand::none();
            }
            let ch = normalize_channel(rest);
            state.buffers.ensure_channel(&ch);
            state.buffers.set_active(&ch);
            state.status_text = format!("joining {ch}…");
            state.feedback_info(format!("joining {ch}…"));
            AppCommand::one(AppCommand::Irc(IrcCommand::Join(ch)))
        }
        "part" | "leave" => {
            let ch = if rest.is_empty() {
                state.buffers.active_name().unwrap_or("").to_string()
            } else {
                normalize_channel(rest)
            };
            if ch.is_empty() || ch == "server" {
                state.feedback_warn("usage: /part [#channel]");
                return AppCommand::none();
            }
            state.status_text = format!("leaving {ch}…");
            state.feedback_info(format!("leaving {ch}…"));
            AppCommand::one(AppCommand::Irc(IrcCommand::Part {
                channel: ch,
                reason: String::new(),
            }))
        }
        "msg" | "query" | "privmsg" => {
            let mut sp = rest.splitn(2, char::is_whitespace);
            let target = sp.next().unwrap_or("").to_string();
            let text = sp.next().unwrap_or("").trim().to_string();
            if target.is_empty() || text.is_empty() {
                state.feedback_warn("usage: /msg <nick|#chan> text");
                return AppCommand::none();
            }
            let target = if target.starts_with('#') || target.starts_with('&') {
                normalize_channel(&target)
            } else {
                target
            };
            state.buffers.ensure_channel(&target);
            state.buffers.set_active(&target);
            state.buffers.push_to(
                Some(&target),
                ChatLine::privmsg(&target, User::nick(state.nick.clone()), text.clone()),
            );
            state.scroll_offset = 0;
            AppCommand::one(AppCommand::Irc(IrcCommand::Privmsg { target, text }))
        }
        "nick" => {
            if rest.is_empty() {
                state.feedback_warn("usage: /nick newnick");
                return AppCommand::none();
            }
            state.status_text = format!("nick → {rest}…");
            state.feedback_info(format!("requesting nick {rest}…"));
            AppCommand::one(AppCommand::Irc(IrcCommand::Nick(rest.to_string())))
        }
        "raw" => {
            if rest.is_empty() {
                state.feedback_warn("usage: /raw PRIVMSG #ditra :hola");
                return AppCommand::none();
            }
            state.feedback_info(format!("raw » {rest}"));
            AppCommand::one(AppCommand::Irc(IrcCommand::Raw(rest.to_string())))
        }
        "buffer" | "win" | "w" | "focus" => {
            if rest.is_empty() {
                state.feedback_info(format!("focus: {}", state.active_title()));
                return AppCommand::none();
            }
            if rest.eq_ignore_ascii_case("server") || rest == "*" {
                state.buffers.set_active("server");
                state.scroll_offset = 0;
                state.feedback_info("focused server buffer");
                return AppCommand::none();
            }
            let name = normalize_channel(rest);
            state.buffers.ensure_channel(&name);
            state.buffers.set_active(&name);
            state.scroll_offset = 0;
            state.status_text = format!("buffer {name}");
            state.feedback_info(format!("focused {name}"));
            AppCommand::none()
        }
        "server" => {
            state.buffers.set_active("server");
            state.scroll_offset = 0;
            state.feedback_info("focused server buffer");
            AppCommand::none()
        }
        "me" | "action" => {
            if rest.is_empty() {
                state.feedback_warn("usage: /me does something");
                return AppCommand::none();
            }
            let Some(target) = state.buffers.active_name().map(str::to_string) else {
                state.feedback_warn("no channel focused");
                return AppCommand::none();
            };
            let ctcp = format!("\x01ACTION {rest}\x01");
            state.buffers.push_to(
                Some(&target),
                ChatLine {
                    ts_ms: line_now(),
                    kind: LineKind::Action,
                    target: Some(target.clone()),
                    from: Some(User::nick(state.nick.clone())),
                    text: format!("* {} {rest}", state.nick),
                },
            );
            state.scroll_offset = 0;
            AppCommand::one(AppCommand::Irc(IrcCommand::Privmsg { target, text: ctcp }))
        }
        "help" | "h" | "?" => {
            for line in [
                "commands:",
                "  /join #chan     join channel",
                "  /part [#chan]   leave channel",
                "  /msg nick text  private message",
                "  /me action      CTCP action",
                "  /nick new       change nick",
                "  /raw LINE       send raw IRC",
                "  /buffer name    focus buffer (or: server)",
                "  /server         focus server log",
                "  /quit           disconnect",
                "  Tab / S-Tab     cycle buffers",
                "  //text          send literal /text",
            ] {
                state.feedback_info(line);
            }
            AppCommand::none()
        }
        other => {
            state.feedback_warn(format!("unknown command /{other} — try /help"));
            AppCommand::none()
        }
    }
}

fn handle_irc(state: &mut AppState, ev: IrcEvent) -> Vec<AppCommand> {
    match ev {
        IrcEvent::Connected => {
            state.status = ConnStatus::Connected;
            state.status_text = "connected — registering…".into();
            state
                .buffers
                .push_server(ChatLine::system(SystemKind::Connected, "TCP connected"));
            AppCommand::none()
        }
        IrcEvent::Disconnected { reason } => {
            state.status = ConnStatus::Disconnected;
            state.status_text = format!("disconnected: {reason}");
            state.buffers.push_server(ChatLine::system(
                SystemKind::Disconnected,
                format!("disconnected: {reason}"),
            ));
            AppCommand::none()
        }
        IrcEvent::Registered { nick } => {
            if !nick.is_empty() {
                state.nick = nick.clone();
            }
            state.status = ConnStatus::Registered;
            state.status_text = format!("registered as {}", state.nick);
            state.buffers.push_server(ChatLine::system(
                SystemKind::Info,
                format!("welcome — you are {}", state.nick),
            ));
            AppCommand::none()
        }
        IrcEvent::Ping { .. } => AppCommand::none(), // adapter already PONGs
        IrcEvent::Privmsg { from, target, text } => {
            // channel msg vs query directed at us
            let buffer = if target.eq_ignore_ascii_case(&state.nick) {
                from.nick.clone()
            } else {
                target.clone()
            };
            state
                .buffers
                .push_to(Some(&buffer), ChatLine::privmsg(&buffer, from, text));
            AppCommand::none()
        }
        IrcEvent::Notice { from, target, text } => {
            let line = ChatLine::notice(target.clone(), from, text);
            if let Some(t) = target {
                if t.starts_with('#') || t.starts_with('&') {
                    state.buffers.push_to(Some(&t), line);
                } else {
                    state.buffers.push_server(line);
                }
            } else {
                state.buffers.push_server(line);
            }
            AppCommand::none()
        }
        IrcEvent::Join { user, channel } => {
            let is_me = user.nick.eq_ignore_ascii_case(&state.nick);
            state.buffers.on_join(&channel, &user, is_me);
            AppCommand::none()
        }
        IrcEvent::Part {
            user,
            channel,
            reason,
        } => {
            state.buffers.on_part(&channel, &user, &reason);
            AppCommand::none()
        }
        IrcEvent::Quit { user, reason } => {
            let text = if reason.is_empty() {
                format!("{} quit", user.nick)
            } else {
                format!("{} quit ({reason})", user.nick)
            };
            let names: Vec<String> = state.buffers.channel_names().to_vec();
            for name in names {
                let ch = state.buffers.ensure_channel(&name);
                if ch.nicks.contains(&user.nick) {
                    ch.remove_nick(&user.nick);
                    ch.push(ChatLine {
                        ts_ms: line_now(),
                        kind: LineKind::Quit,
                        target: Some(name.clone()),
                        from: Some(user.clone()),
                        text: text.clone(),
                    });
                }
            }
            AppCommand::none()
        }
        IrcEvent::Nick { user, new_nick } => {
            if user.nick.eq_ignore_ascii_case(&state.nick) {
                state.nick = new_nick.clone();
                state.status_text = format!("nick → {new_nick}");
            }
            let names: Vec<String> = state.buffers.channel_names().to_vec();
            for name in names {
                let ch = state.buffers.ensure_channel(&name);
                if ch.nicks.contains(&user.nick) {
                    ch.rename_nick(&user.nick, &new_nick);
                    ch.push(ChatLine {
                        ts_ms: line_now(),
                        kind: LineKind::Nick,
                        target: Some(name),
                        from: Some(user.clone()),
                        text: format!("{} → {new_nick}", user.nick),
                    });
                }
            }
            AppCommand::none()
        }
        IrcEvent::Topic { channel, topic } => {
            let ch = state.buffers.ensure_channel(&channel);
            ch.topic = Some(topic.clone());
            ch.push(ChatLine {
                ts_ms: line_now(),
                kind: LineKind::Topic,
                target: Some(channel),
                from: None,
                text: format!("topic: {topic}"),
            });
            AppCommand::none()
        }
        IrcEvent::Names { channel, nicks } => {
            let ch = state.buffers.ensure_channel(&channel);
            for n in nicks {
                ch.add_nick(n);
            }
            AppCommand::none()
        }
        IrcEvent::Raw(msg) => {
            // surface selected numerics as server lines; ignore noisy ones later
            let cmd = msg.command.to_uppercase();
            if matches!(
                cmd.as_str(),
                "002"
                    | "003"
                    | "004"
                    | "005"
                    | "251"
                    | "252"
                    | "253"
                    | "254"
                    | "255"
                    | "265"
                    | "266"
                    | "250"
                    | "375"
                    | "372"
                    | "376"
                    | "422"
                    | "366"
                    | "333"
                    | "321"
                    | "323"
            ) {
                // MOTD / ISUPPORT noise — keep MOTD body only
                if (cmd == "372" || cmd == "375")
                    && let Some(t) = msg.trailing()
                {
                    state
                        .buffers
                        .push_server(ChatLine::now(LineKind::Raw, t.to_string()));
                }
                return AppCommand::none();
            }
            let text = format!("{} {}", msg.command, msg.params.join(" "));
            state
                .buffers
                .push_server(ChatLine::now(LineKind::Raw, text));
            AppCommand::none()
        }
        IrcEvent::Error(e) => {
            state
                .buffers
                .push_server(ChatLine::system(SystemKind::Error, e));
            AppCommand::none()
        }
    }
}

fn cycle_buffer(state: &mut AppState, reverse: bool) {
    let names = state.buffers.tab_targets();
    if names.is_empty() {
        return;
    }
    let current = state.active_title();
    let idx = names
        .iter()
        .position(|n| n.eq_ignore_ascii_case(&current))
        .unwrap_or(0);
    let next = if reverse {
        if idx == 0 { names.len() - 1 } else { idx - 1 }
    } else {
        (idx + 1) % names.len()
    };
    let name = names[next].clone();
    state.buffers.set_active(&name);
    state.scroll_offset = 0;
    state.status_text = format!("buffer {name}");
}

fn prev_char_boundary(s: &str, idx: usize) -> usize {
    if idx == 0 {
        return 0;
    }
    let mut i = idx - 1;
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn next_char_boundary(s: &str, idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    let mut i = idx + 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn line_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn slash_join_emits_irc_command() {
        let mut state = AppState::new(Config::default());
        state.input = "/join #ops".into();
        state.input_cursor = state.input.len();
        let cmds = update(
            &mut state,
            AppEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        );
        assert!(matches!(
            cmds.first(),
            Some(AppCommand::Irc(IrcCommand::Join(c))) if c == "#ops"
        ));
        assert_eq!(state.buffers.active_name(), Some("#ops"));
    }

    #[test]
    fn slash_help_visible_on_active_channel() {
        let mut state = AppState::new(Config::default());
        assert_eq!(state.buffers.active_name(), Some("#ditra"));
        state.input = "/help".into();
        state.input_cursor = state.input.len();
        let cmds = update(
            &mut state,
            AppEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        );
        assert!(cmds.is_empty());
        let ch = state.buffers.active_channel().expect("active");
        let text: Vec<_> = ch.lines().map(|l| l.text.as_str()).collect();
        assert!(
            text.iter().any(|t| t.contains("» /help")),
            "expected command echo on active buffer, got {text:?}"
        );
        assert!(
            text.iter().any(|t| t.contains("/join")),
            "expected help body on active buffer, got {text:?}"
        );
    }

    #[test]
    fn double_slash_sends_literal() {
        let mut state = AppState::new(Config::default());
        state.input = "//not-a-command".into();
        state.input_cursor = state.input.len();
        let cmds = update(
            &mut state,
            AppEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
        );
        assert!(matches!(
            cmds.first(),
            Some(AppCommand::Irc(IrcCommand::Privmsg { text, .. })) if text == "/not-a-command"
        ));
    }
}
