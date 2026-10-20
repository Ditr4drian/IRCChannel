//! Async TCP IRC client.
//!
//! Lifecycle:
//! 1. `IrcClient::connect` opens the socket and spawns reader/writer tasks
//! 2. App sends [`IrcCommand`] via the returned sink
//! 3. App receives [`IrcEvent`] via the returned stream
//!
//! The adapter auto-answers PING and runs registration (PASS/NICK/USER).

use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::irc::codec::{self, RawMessage};
use crate::irc::command::IrcCommand;
use crate::irc::event::{self, IrcEvent};

pub struct IrcClient;

pub struct IrcHandle {
    pub events: mpsc::UnboundedReceiver<IrcEvent>,
    pub commands: mpsc::UnboundedSender<IrcCommand>,
}

impl IrcClient {
    pub async fn connect(cfg: &Config) -> Result<IrcHandle> {
        let addr = cfg.socket_addr();
        info!(%addr, "connecting");
        let stream = TcpStream::connect(&addr).await?;
        stream.set_nodelay(true)?;

        let (reader, mut writer) = stream.into_split();
        let (ev_tx, ev_rx) = mpsc::unbounded_channel::<IrcEvent>();
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<IrcCommand>();
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();

        let _ = ev_tx.send(IrcEvent::Connected);

        // registration
        if !cfg.connection.server_password.is_empty() {
            let _ = out_tx.send(IrcCommand::Pass(cfg.connection.server_password.clone()).to_wire());
        }
        let _ = out_tx.send(IrcCommand::Nick(cfg.identity.nick.clone()).to_wire());
        let _ = out_tx.send(
            IrcCommand::User {
                username: cfg.identity.username.clone(),
                realname: cfg.identity.realname.clone(),
            }
            .to_wire(),
        );

        let channels = cfg.session.channels.clone();
        let send_delay = Duration::from_millis(cfg.session.send_delay_ms);

        // writer task: serializes outbound lines
        let write_ev = ev_tx.clone();
        tokio::spawn(async move {
            while let Some(line) = out_rx.recv().await {
                debug!(%line, ">>");
                let payload = format!("{line}\r\n");
                if let Err(e) = writer.write_all(payload.as_bytes()).await {
                    let _ = write_ev.send(IrcEvent::Disconnected {
                        reason: format!("write failed: {e}"),
                    });
                    break;
                }
                if let Err(e) = writer.flush().await {
                    let _ = write_ev.send(IrcEvent::Disconnected {
                        reason: format!("flush failed: {e}"),
                    });
                    break;
                }
                if !send_delay.is_zero() {
                    tokio::time::sleep(send_delay).await;
                }
            }
        });

        // command pump: app -> wire
        let out_for_cmd = out_tx.clone();
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                if out_for_cmd.send(cmd.to_wire()).is_err() {
                    break;
                }
            }
        });

        // reader task
        let out_for_ping = out_tx.clone();
        let join_channels = channels;
        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            let mut registered = false;
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        debug!(%line, "<<");
                        match codec::parse_line(&line) {
                            Ok(raw) => {
                                let classified = event::classify(raw);
                                match &classified {
                                    IrcEvent::Ping { token } => {
                                        let _ = out_for_ping
                                            .send(IrcCommand::Pong(token.clone()).to_wire());
                                    }
                                    IrcEvent::Registered { .. } if !registered => {
                                        registered = true;
                                        for ch in &join_channels {
                                            let _ = out_for_ping
                                                .send(IrcCommand::Join(ch.clone()).to_wire());
                                        }
                                    }
                                    _ => {}
                                }
                                if ev_tx.send(classified).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, %line, "failed to parse IRC line");
                            }
                        }
                    }
                    Ok(None) => {
                        let _ = ev_tx.send(IrcEvent::Disconnected {
                            reason: "server closed connection".into(),
                        });
                        break;
                    }
                    Err(e) => {
                        error!(error = %e, "read error");
                        let _ = ev_tx.send(IrcEvent::Disconnected {
                            reason: e.to_string(),
                        });
                        break;
                    }
                }
            }
        });

        Ok(IrcHandle {
            events: ev_rx,
            commands: cmd_tx,
        })
    }
}

/// Helper kept for unit-style checks without opening a socket.
#[allow(dead_code)]
pub fn encode_registration(nick: &str, user: &str, real: &str) -> Vec<RawMessage> {
    vec![
        RawMessage::new("NICK", vec![nick.into()]),
        RawMessage::new(
            "USER",
            vec![user.into(), "0".into(), "*".into(), real.into()],
        ),
    ]
}

impl From<mpsc::error::SendError<IrcCommand>> for Error {
    fn from(_: mpsc::error::SendError<IrcCommand>) -> Self {
        Error::ChannelClosed
    }
}
